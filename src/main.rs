#![feature(stmt_expr_attributes)]
#![warn(missing_docs)]
#![warn(clippy::cargo)]
#![warn(clippy::pedantic)]
// Unfortunately we have duplicate dependencies with different versions
#![allow(clippy::multiple_crate_versions)]

//! Meshtastic to Postgresql database daemon

#[cfg(feature = "syslog")]
extern crate syslog;
#[cfg(feature = "syslog")]
#[macro_use]
extern crate log;

use crate::dto::packet_handler::process_packet;
use crate::util::config::DEPLOYMENT_LOCATION;
use crate::util::log::{log_msg, log_perf};
use crate::util::{config::Settings, log::set_logger, state::GatewayState};
use anyhow::{Context, Result};
use meshtastic::api::StreamApi;
use meshtastic::protobufs::FromRadio;
use meshtastic::utils;
#[cfg(feature = "print-packets")]
use serde_json::to_string_pretty;
use sqlx::{Pool, Postgres};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::runtime::Builder;
use tokio::sync::mpsc::{self, Receiver};

/// Handle data transfer objects
pub(crate) mod dto;
/// Utilities module
pub(crate) mod util;

/// Version number of the daemon
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> std::result::Result<(), anyhow::Error> {
    #[cfg(debug_assertions)]
    let settings = Settings::new("example_config.toml");
    #[cfg(not(debug_assertions))]
    let settings = Settings::new("/etc/meshtastic_telem.toml");

    set_logger();

    Builder::new_multi_thread()
        .enable_all()
        .thread_name("mesh-telem")
        .worker_threads(settings.async_runtime.worker_threads as usize)
        .max_blocking_threads(settings.async_runtime.max_blocking_threads as usize)
        .thread_stack_size(settings.async_runtime.thread_stack_size as usize)
        .build()
        .with_context(|| "Failed to build tokio multithreaded runtime")?
        .block_on(async { rt_main(settings).await })
}

async fn rt_main(settings: Settings<'static>) -> Result<(), anyhow::Error> {
    // Create the gateway's state object
    let state = Arc::new(Mutex::new(GatewayState::new()));

    // Create postgresql connection
    let postgres_db = Arc::new(
        settings
            .setup_postgres()
            .with_context(|| "Failed to connect to postgresql database")?,
    );

    // Connect to serial meshtastic
    let stream_api = StreamApi::new();
    let entered_port = settings.get_serial_port();
    let serial_stream =
        utils::stream::build_serial_stream(entered_port.to_string(), None, None, None)
            .with_context(|| format!("Failed to build serial stream for {entered_port}"))?;
    let (mut decoded_listener, stream_api) = stream_api.connect(serial_stream).await;

    let config_id = utils::generate_rand_id();
    let stream_api = stream_api
        .configure(config_id)
        .await
        .with_context(|| "Failed to configure serial stream")?;

    // Set the global deployment location string
    DEPLOYMENT_LOCATION
        .set(settings.deployment.location.to_string())
        .unwrap_or_else(|e| {
            panic!(
                "{}:\n\tUnable to initialize global DEPLOYMENT_LOCATION from configuration's value: {}\n ",
                e,
                settings.deployment.location
            )
        });

    // Create a mpsc channel for passing FromRadio data
    let (tx, rx) = mpsc::channel(settings.async_runtime.mpsc_buffer_size.into());

    // Output the version of the daemon to the logger
    log_msg(
        format!("Daemon version: {VERSION}").as_str(),
        log::Level::Info,
    );

    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;

    // Spawn the task for handling packets
    let s = state.clone();
    let pkt_handler = tokio::spawn(async move { packet_handler(rx, &s, &postgres_db).await });

    // This loop can be broken with ctrl+c, or by disconnecting
    // the attached serial port, or by sending a SIGTERM signal
    // through systemctl or other means
    while !term.load(Ordering::Relaxed)
        && let Some(from_radio) = decoded_listener.recv().await
    {
        match tx.send(Box::new(from_radio)).await {
            Ok(()) => (),
            Err(e) => log_msg(
                &format!("Error sending from_radio packet {e}"),
                log::Level::Warn,
            ),
        }

        #[cfg(feature = "debug")]
        {
            // log performance metrics
            log_perf();
            // log state messages
            log_msg(
                state
                    .lock()
                    .expect("Failed to acquire lock for GatewayState in main()")
                    .format_rx_counts()
                    .as_ref(),
                log::Level::Info,
            );
        }
    }

    // Called when either the radio is disconnected or the daemon recieves
    // a SIGTERM or SIGKILL signal from systemctl or by other means
    let _stream_api = stream_api.disconnect().await?;

    // Close the packet_handler worker now that the serial is disconnected
    match pkt_handler.await {
        Ok(()) => (),
        Err(e) => log_msg(&format!("Error joining pkt_handler {e}"), log::Level::Warn),
    }

    Ok(())
}

async fn packet_handler(
    mut rx: Receiver<Box<FromRadio>>,
    state: &Arc<Mutex<GatewayState<'_>>>,
    db: &Arc<Pool<Postgres>>,
) {
    while let Some(from_radio) = rx.recv().await {
        process_packet(&from_radio, state, db).await;
    }
}

#[cfg(all(feature = "debug", feature = "syslog"))]
compile_error!("feature \"debug\" and feature \"syslog\" cannot be enabled at the same time");
