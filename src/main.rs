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
#[cfg(feature = "log_perf")]
use crate::util::log::log_perf;
use crate::util::{config::Settings, log::set_logger, state::GatewayState};
use anyhow::{Context, Result};
use meshtastic::api::StreamApi;
use meshtastic::protobufs::FromRadio;
use meshtastic::utils;
#[cfg(feature = "print-packets")]
use serde_json::to_string_pretty;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, mpsc::UnboundedReceiver};

/// Handle data transfer objects
pub(crate) mod dto;
/// Utilities module
pub(crate) mod util;

/// Version number of the daemon
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), anyhow::Error> {
    use crate::log_msg;

    #[cfg(feature = "trace")]
    console_subscriber::init();

    #[cfg(debug_assertions)]
    let settings = Settings::new("example_config.toml");
    #[cfg(not(debug_assertions))]
    let settings = Settings::new("/etc/meshtastic_telem.toml");

    set_logger();

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
        .set(Box::leak(settings.deployment.location.to_string().into_boxed_str()))
        .unwrap_or_else(|e| {
            panic!(
                "{}:\n\tUnable to initialize global DEPLOYMENT_LOCATION from configuration's value: {}\n ",
                e,
                settings.deployment.location
            )
        });

    // Output the version of the daemon to the logger
    log_msg!(log::Level::Info, "Daemon version: {VERSION}");

    // Spawn the task for handling packets
    let term = Arc::new(AtomicBool::new(false));
    match tokio::spawn(async move {
        packet_handler(&state, &postgres_db, &term, &mut decoded_listener).await;
    })
    .await
    {
        Ok(()) => (),
        Err(e) => log_msg!(
            log::Level::Error,
            "Error joining packet_handler() in main(): {e}"
        ),
    }

    // Called when either the radio is disconnected or the daemon recieves
    // a SIGTERM or SIGKILL signal from systemctl or by other means
    match stream_api.disconnect().await {
        Ok(a) => log_msg!(
            log::Level::Warn,
            "StreamApi disconnected without error: {a:?}",
        ),
        Err(e) => log_msg!(log::Level::Error, "StreamApi disconnected with error: {e}"),
    }

    Ok(())
}

async fn packet_handler(
    state: &Arc<Mutex<GatewayState<'_>>>,
    db: &Arc<Pool<Postgres>>,
    term: &Arc<AtomicBool>,
    decoded_listener: &mut UnboundedReceiver<Box<FromRadio>>,
) {
    match signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(term)) {
        Ok(a) => log_msg!(log::Level::Info, "Successfully registered SIGTERM: {a:?}"),
        Err(e) => log_msg!(log::Level::Warn, "Failed to register SIGTERM: {e}"),
    }

    // This loop can be broken with ctrl+c, or by disconnecting
    // the attached serial port, or by sending a SIGTERM signal
    // through systemctl or other means
    while !term.load(Ordering::Relaxed)
        && let Some(from_radio) = decoded_listener.recv().await
    {
        process_packet(&from_radio, state, db).await;

        #[cfg(feature = "debug")]
        {
            // log performance metrics
            #[cfg(feature = "log_perf")]
            log_perf();
            // log state messages
            let lock = state.lock().await;
            let st = lock.format_rx_counts();
            log_msg!(log::Level::Info, "{st}");
        }
    }
}

#[cfg(all(feature = "colog", feature = "syslog"))]
compile_error!("feature \"colog\" and feature \"syslog\" cannot be enabled at the same time");
