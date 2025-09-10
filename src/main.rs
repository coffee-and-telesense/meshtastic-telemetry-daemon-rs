#![feature(stmt_expr_attributes)]
#![warn(missing_docs)]
#![warn(clippy::cargo)]
#![warn(clippy::pedantic)]
// Unfortunately we have duplicate dependencies with different versions
#![allow(clippy::multiple_crate_versions)]

//! Meshtastic to Postgresql/Sqlite database daemon

#[cfg(feature = "syslog")]
extern crate syslog;
#[cfg(feature = "syslog")]
#[macro_use]
extern crate log;

#[cfg(any(feature = "sqlite", feature = "postgres"))]
use crate::db::connection::update_metrics;
use crate::dto::packet_handler;
use crate::util::{
    config::Settings,
    log::set_logger,
    types::{GatewayState, Pkt},
};
use anyhow::{Context, Result};
use chrono::Local;
use db::connection::proactive_ninfo_insert;
#[cfg(feature = "sqlite")]
use db::lite::{self, drop_old_rows, pragma_optimize};
#[cfg(feature = "debug")]
use log::{error, info, warn};
use meshtastic::api::StreamApi;
use meshtastic::utils;
#[cfg(feature = "print-packets")]
use serde_json::to_string_pretty;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
#[cfg(feature = "sqlite")]
use std::time::Instant;
use tokio::sync::mpsc;

/// Database interaction module
#[cfg(any(feature = "sqlite", feature = "postgres"))]
pub(crate) mod db;
/// Handle data transfer objects
pub(crate) mod dto;
/// Utilities module
pub(crate) mod util;

/// Version number of the daemon
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main
///
/// # Returns
/// * Result of () or an Error
#[tokio::main(worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    let settings = Settings::new("example_config.toml");
    #[cfg(not(debug_assertions))]
    let settings = Settings::new("/etc/meshtastic_telem.toml");

    set_logger();

    // Create the gateway's state object
    let state = Arc::new(Mutex::new(GatewayState::new()));

    // Create postgresql connection
    #[cfg(feature = "postgres")]
    let postgres_db = settings
        .setup_postgres()
        .await
        .with_context(|| "Failed to connect to postgresql database")?;

    // Create sqlite db
    #[cfg(feature = "sqlite")]
    let sqlite_db = settings
        .setup_sqlite()
        .await
        .with_context(|| "Failed to setup sqlite database")?;

    // Connect to serial meshtastic
    let stream_api = StreamApi::new();
    let entered_port = settings.get_serial_port();
    let serial_stream = utils::stream::build_serial_stream(entered_port.clone(), None, None, None)
        .with_context(|| format!("Failed to build serial stream for {entered_port}"))?;
    let (mut decoded_listener, stream_api) = stream_api.connect(serial_stream).await;

    let config_id = utils::generate_rand_id();
    let stream_api = stream_api
        .configure(config_id)
        .await
        .with_context(|| "Failed to configure serial stream")?;

    let deployment_loc = settings.deployment.location;

    let (tx, mut rx) = mpsc::channel(settings.async_runtime.mpsc_buffer_size.into());

    // Output the version of the daemon to the logger
    log::info!("Daemon version: {VERSION}");

    // Timers for optimization of sqlite
    #[cfg(feature = "sqlite")]
    let mut few_hours = Instant::now();
    #[cfg(feature = "sqlite")]
    let mut daily = few_hours.clone();

    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;

    // This loop can be broken with ctrl+c, or by disconnecting
    // the attached serial port, or by sending a SIGTERM signal
    // through systemctl or other means
    while !term.load(Ordering::Relaxed)
        && let Some(decoded) = decoded_listener.recv().await
    {
        let tx = tx.clone();
        let s = state.clone();
        let join = tokio::spawn(async move {
            // Process packet will consume decoded on this iteration, but needs to be able to
            // asynchronously pass back results, so in cases where multiple packets arrive
            // simultaneously we can parallel process up to 4 and then send them back to here.
            // Although the more elegant solution will be bridging the types with eval macros rules
            // and eliminating vast swaths of the codebase. This will also let us elimnate this
            // barrier between db inserts and packet receptions. But for now lets do a hacky
            // solution just to test the borrow checker and my async skills as the problem may
            // still exist within the more elegant solution.
            tx.send(packet_handler::process_packet(&decoded, &s))
                .await
                .unwrap();
        });
        if let Some(pkt) = rx.recv().await.unwrap() {
            match pkt {
                Pkt::Mesh(ref mp) => {
                    // Count received packets in debug builds for periodic reporting in logs
                    #[cfg(feature = "debug")]
                    if let Ok(mut lock) = state.clone().lock() {
                        lock.increment_rx_count(mp.from);
                        println!("{}", lock.format_rx_counts());
                    }
                    // Print packets if enabled
                    #[cfg(feature = "print-packets")]
                    println!("{}", to_string_pretty(&mp).unwrap());
                    // Before we insert into postgres or sqlite, we should proactively check that the foreign
                    // key constraint is satisfied and if not we then insert a new nodeinfo row
                    if let Some(p) = &mp.payload {
                        match p {
                            util::types::Payload::NodeinfoApp(_u) => {
                                info!("Received nodeinfo payload");
                            }
                            _ => {
                                #[cfg(feature = "postgres")]
                                match proactive_ninfo_insert(
                                    mp,
                                    &postgres_db,
                                    &deployment_loc,
                                    state.clone(),
                                )
                                .await
                                .with_context(|| {
                                    "Failed to update postgres database with proactive_node_info()"
                                }) {
                                    Ok(v) => {
                                        if v != 0 {
                                            let now = Local::now();
                                            info!(
                                                "{}Inserted {v} rows into NodeInfo table of postgres db proactively",
                                                now.format("%Y-%m-%d %H:%M:%S - ")
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        let now = Local::now();
                                        error!("{}{e:#}", now.format("%Y-%m-%d %H:%M:%S - "));
                                    }
                                }
                                #[cfg(feature = "sqlite")]
                                match proactive_ninfo_insert(
                                    mp,
                                    &sqlite_db,
                                    &deployment_loc,
                                    state.clone(),
                                )
                                .await
                                .with_context(|| {
                                    "Failed to update sqlite database with proactive_node_info()"
                                }) {
                                    Ok(v) => {
                                        if v != 0 {
                                            let now = Local::now();
                                            info!(
                                                "{}Inserted {v} rows into NodeInfo table of sqlite db proactively",
                                                now.format("%Y-%m-%d %H:%M:%S - ")
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        let now = Local::now();
                                        error!("{}{e:#}", now.format("%Y-%m-%d %H:%M:%S - "));
                                    }
                                }
                            }
                        }
                    }
                    // Get tablename to inline into insert messages
                    let tablename = mp.match_tablename();
                    #[cfg(feature = "postgres")]
                    match update_metrics(&postgres_db, &pkt, None, &deployment_loc)
                        .await
                        .with_context(|| {
                            format!("Failed to update {tablename} table in postgres datatbase with packet from mesh")
                        }) {
                        Ok(v) => {
                            let now = Local::now();
                            info!(
                                "{}Inserted {v} rows into {tablename} of postgres db",
                                now.format("%Y-%m-%d %H:%M:%S - ")
                            );
                        }
                        Err(e) => {
                            let now = Local::now();
                            error!("{}{e:#}", now.format("%Y-%m-%d %H:%M:%S - "));
                        }
                    }
                    #[cfg(feature = "sqlite")]
                    match update_metrics(&sqlite_db, &pkt, None, &deployment_loc)
                        .await
                        .with_context(|| format!("Failed to update {} table in sqlite datatbase with packet from mesh", tablename))
                    {
                        Ok(v) => {
                            let now = Local::now();
                            info!(
                                "{}Inserted {v} rows into {tablename} of sqlite db",
                                now.format("%Y-%m-%d %H:%M:%S - ")
                            );
                        }
                        Err(e) => {
                            let now = Local::now();
                            error!("{}{e:#}", now.format("%Y-%m-%d %H:%M:%S - "));
                        }
                    }
                }
                Pkt::NInfo(ref ni) => {
                    #[cfg(feature = "print-packets")]
                    println!("{}", to_string_pretty(&ni).unwrap());
                    let fake = state
                        .lock()
                        .expect("Failed to acquire lock for GatewayState in main()")
                        .find_fake_id(ni.num)
                        .expect("No fake_id returned");
                    #[cfg(feature = "postgres")]
                    match update_metrics(&postgres_db, &pkt, Some(fake.into()), &deployment_loc)
                        .await
                        .with_context(|| {
                            "Failed to update postgres database with node info packet from serial"
                        }) {
                        Ok(v) => {
                            let now = Local::now();
                            info!(
                                "{}Inserted {v} rows into NodeInfo table of postgres db",
                                now.format("%Y-%m-%d %H:%M:%S - ")
                            );
                        }
                        Err(e) => {
                            // This is a lower priority error message since we favor node info data
                            // from the Mesh rather than from the serial connection. Often times it
                            // just means that we did not insert a row
                            let now = Local::now();
                            info!("{}{e:#}", now.format("%Y-%m-%d %H:%M:%S - "));
                        }
                    }
                    #[cfg(feature = "sqlite")]
                    match update_metrics(&sqlite_db, &pkt, Some(fake.into()), &deployment_loc)
                        .await
                        .with_context(
                            || "Failed to update sqlite database with node info packet from serial",
                        ) {
                        Ok(v) => {
                            let now = Local::now();
                            info!(
                                "{}Inserted {v} rows into NodeInfo table of sqlite db",
                                now.format("%Y-%m-%d %H:%M:%S - ")
                            );
                        }
                        Err(e) => {
                            // This is a lower priority error message since we favor node info data
                            // from the Mesh rather than from the serial connection. Often times it
                            // just means that we did not insert a row
                            let now = Local::now();
                            info!("{}{e:#}", now.format("%Y-%m-%d %H:%M:%S - "));
                        }
                    }
                }
                Pkt::MyNodeInfo(ref mi) => {
                    #[cfg(feature = "print-packets")]
                    println!("{}", to_string_pretty(&mi).unwrap());
                    #[cfg(feature = "debug")]
                    state
                        .clone()
                        .lock()
                        .expect("Failed to acquire lock for GatewayState")
                        .set_serial_number(mi.my_node_num);
                }
            }
            // Thread has been used to process and send to DB, kill it
            join.await?;
            // Dumb sqlite optimizations
            #[cfg(feature = "sqlite")]
            {
                few_hours = pragma_optimize(&sqlite_db, few_hours).await;
                daily = drop_old_rows(&sqlite_db, daily).await;
            }
        }
    }

    // Called when either the radio is disconnected or the daemon recieves
    // a SIGTERM or SIGKILL signal from systemctl or by other means
    let _stream_api = stream_api.disconnect().await?;

    Ok(())
}

#[cfg(all(feature = "debug", feature = "syslog"))]
compile_error!("feature \"debug\" and feature \"syslog\" cannot be enabled at the same time");
