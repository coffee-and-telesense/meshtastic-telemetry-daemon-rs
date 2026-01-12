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
use tokio::sync::mpsc::{self, Receiver};

/// Handle data transfer objects
pub(crate) mod dto;
/// Utilities module
pub(crate) mod util;

/// Version number of the daemon
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), anyhow::Error> {
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
        .set(settings.deployment.location.to_string())
        .unwrap_or_else(|e| {
            panic!(
                "{}:\n\tUnable to initialize global DEPLOYMENT_LOCATION from configuration's value: {}\n ",
                e,
                settings.deployment.location
            )
        });

    // Output the version of the daemon to the logger
    log_msg(
        format!("Daemon version: {VERSION}").as_str(),
        log::Level::Info,
    );

    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;

    // This loop can be broken with ctrl+c, or by disconnecting
    // the attached serial port, or by sending a SIGTERM signal
    // through systemctl or other means
    while !term.load(Ordering::Relaxed)
        && let Some(from_radio) = decoded_listener.recv().await
    {
        let s = state.clone();
        let db = postgres_db.clone();
        let join = tokio::spawn(async move {
            process_packet(&from_radio, &s, &db).await;
        });

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

        match join.await {
            Ok(()) => (),
            Err(e) => log_msg(&format!("Error joining tasks {e}"), log::Level::Warn),
        }
    }

    // Called when either the radio is disconnected or the daemon recieves
    // a SIGTERM or SIGKILL signal from systemctl or by other means
    let _stream_api = stream_api.disconnect().await?;

    Ok(())
}

#[cfg(all(feature = "debug", feature = "syslog"))]
compile_error!("feature \"debug\" and feature \"syslog\" cannot be enabled at the same time");
