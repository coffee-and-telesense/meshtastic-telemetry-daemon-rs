#![warn(missing_docs)]
#![expect(
    clippy::multiple_crate_versions,
    reason = "meshtastic and sqlx pull different version of core deps"
)]

//! Meshtastic to `PostgreSQL` database daemon

use crate::dto::packet_handler::process_packet;
use crate::util::MAX_INFLIGHT_TASKS;
use crate::util::config::DEPLOYMENT_LOCATION;
#[cfg(feature = "log_perf")]
use crate::util::log::log_perf;
use crate::util::{config::Settings, log::set_logger, state::GatewayState};
use anyhow::{Context, Error, Result, anyhow};
use meshtastic::api::StreamApi;
use meshtastic::utils;
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;
#[cfg(feature = "print-packets")]
use serde_json::to_string_pretty;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::Instrument;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Handle data transfer objects
pub(crate) mod dto;
/// Utilities module
pub(crate) mod util;

/// Version number of the daemon
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Error> {
    // Set the logger
    set_logger();

    // Read settings
    let settings = Settings::new().context("Error initializing Settings")?;

    // Create the gateway's state object
    let state = Arc::new(GatewayState::new());

    // Create PostgreSQL connection
    let postgres_db = settings
        .setup_postgres()
        .await
        .context("Failed to connect to postgresql database")?;

    // Connect to serial Meshtastic
    let stream_api = StreamApi::new();
    let entered_port = settings
        .get_serial_port()
        .context("Failed to get serial port")?;
    let serial_stream =
        utils::stream::build_serial_stream(entered_port.to_string(), None, None, None)
            .with_context(|| format!("Failed to build serial stream for {entered_port}"))?;
    let (mut decoded_listener, stream_api) = stream_api.connect(serial_stream).await;

    let config_id = utils::generate_rand_id();
    let stream_api = stream_api
        .configure(config_id)
        .await
        .context("Failed to configure serial stream")?;

    // Create a semaphore to bound the unbounded channel, maximum value of 32 tasks
    let max_tasks = (settings.get_max_connections() * 2).min(MAX_INFLIGHT_TASKS);
    let semaphore = Arc::new(Semaphore::new(max_tasks));

    // Set the global deployment location string
    DEPLOYMENT_LOCATION
        .set(Box::leak(
            settings.deployment.location.into_owned().into_boxed_str(),
        ))
        .context("DEPLOYMENT_LOCATION initialized twice")?;

    // Output the version of the daemon to the logger
    tracing::info!("Daemon version: {VERSION}");

    // Load the already filled in nodeinfo tables to the state
    state.load_from_db(&postgres_db).await?;

    // This loop can be broken with ctrl+c, or by disconnecting
    // the attached serial port, or by sending a SIGTERM signal
    // through systemctl or other means
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::warn!("Received SIGINT");
                break;
            }
            msg = decoded_listener.recv() => {
                if let Some(from_radio) = msg {
                    let permit = match Arc::clone(&semaphore).acquire_owned().await {
                        Ok(p) => p,
                        Err(e) => {
                            tracing::error!(%e, "Could not acquire an owned clone of the semaphore");
                            return Result::Err(anyhow!(e));
                        },
                    };
                    let s = Arc::clone(&state);
                    let pool = postgres_db.clone();
                    let span = tracing::info_span!("packet", from = from_radio.id);
                    match tokio::spawn(async move {
                        process_packet(&from_radio, &s, &pool).await;

                        // Debug logging in task after receiving/processing/inserting
                        #[cfg(feature = "debug")]
                        {
                            // log performance metrics
                            #[cfg(feature = "log_perf")]
                            log_perf();
                            // log state messages
                            if s.any_recvd() {
                                tracing::info!("{s}");
                            }
                        }

                        // Release semaphore permit to permit spawning a new task
                        drop(permit);
                    }).instrument(span).await {
                        Ok(()) => (),
                        Err(e) => tracing::error!(%e),
                    }
                } else {
                    tracing::error!("Serial connection closed");
                    break;
                }
            }
        }
    }

    // Called when either the radio is disconnected or the daemon receives
    // a SIGTERM or SIGKILL signal from systemctl or by other means
    match stream_api.disconnect().await {
        Ok(_) => tracing::warn!("StreamApi disconnected without error"),
        Err(e) => tracing::error!(%e, "StreamApi disconnected with error"),
    }

    Ok(())
}
