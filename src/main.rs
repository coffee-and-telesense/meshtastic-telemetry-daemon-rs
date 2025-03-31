#![warn(missing_docs)]
#![warn(clippy::cargo)]
#![warn(clippy::pedantic)]
// Unfortunately we have duplicate dependencies with different versions
#![allow(clippy::multiple_crate_versions)]

//! Meshtastic to Postgresql/Sqlite database daemon

#[cfg(not(debug_assertions))]
extern crate syslog;
#[cfg(not(debug_assertions))]
#[macro_use]
extern crate log;

use crate::db::connection::update_metrics;
use crate::dto::packet_handler;
use crate::util::{
    config::{build_db_connection_string, get_cfg_string, get_serial_port, read_config},
    log::set_logger,
    types::{GatewayState, Pkt},
};
use anyhow::{Context, Result};
use db::lite::{drop_old_rows, pragma_optimize};
use db::{lite, postgres};
#[cfg(feature = "debug")]
use log::{error, info, warn};
use meshtastic::api::StreamApi;
use meshtastic::utils;
#[cfg(feature = "print-packets")]
use serde_json::to_string_pretty;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::mpsc;

/// Database interaction module
pub(crate) mod db;
/// Handle data transfer objects
pub(crate) mod dto;
/// Utilities module
pub(crate) mod util;

/// Main
///
/// # Returns
/// * Result of () or an Error
#[tokio::main(worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    let settings = read_config("example_config.toml");
    #[cfg(not(debug_assertions))]
    let settings = read_config("/etc/meshtastic_telem.toml");

    set_logger();

    // Create the gateway's state object
    let state = Arc::new(Mutex::new(GatewayState::new()));

    // Create postgresql connection
    let postgres_db = postgres::setup(&settings).await?;

    // Create sqlite db
    let sqlite_db = lite::setup()
        .await
        .with_context(|| "Failed to setup sqlite database")?;

    // Timers for optimization of sqlite
    let mut few_hours = Instant::now();
    let mut daily = few_hours.clone();

    // Connect to serial meshtastic
    let stream_api = StreamApi::new();
    let entered_port = get_serial_port(&settings);
    let serial_stream = utils::stream::build_serial_stream(entered_port.clone(), None, None, None)
        .with_context(|| format!("Failed to build serial stream for {entered_port}"))?;
    let (mut decoded_listener, stream_api) = stream_api.connect(serial_stream).await;

    let config_id = utils::generate_rand_id();
    let stream_api = stream_api
        .configure(config_id)
        .await
        .with_context(|| "Failed to configure serial stream")?;

    let deployment_loc = get_cfg_string(&settings, "deployment_location");

    // Decrease the channel size to 4 from 32, in order to prevent OOMs
    let (tx, mut rx) = mpsc::channel(4);

    // This loop can be broken with ctrl+c, or by disconnecting
    // the attached serial port.
    while let Some(decoded) = decoded_listener.recv().await {
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
            tx.send(packet_handler::process_packet(decoded, s))
                .await
                .unwrap();
        });
        if let Some(pkt) = rx.recv().await.unwrap() {
            match pkt {
                Pkt::Mesh(ref mp) => {
                    #[cfg(feature = "print-packets")]
                    println!("{}", to_string_pretty(&mp).unwrap());
                    match update_metrics(&postgres_db, &pkt, None, &deployment_loc)
                        .await
                        .with_context(|| {
                            "Failed to update postgres datatbase with packet from mesh"
                        }) {
                        Ok(v) => info!("inserted {v} rows into postgres db"),
                        Err(e) => error!("{e}"),
                    }
                    match update_metrics(&sqlite_db, &pkt, None, &deployment_loc)
                        .await
                        .with_context(|| "Failed to update sqlite datatbase with packet from mesh")
                    {
                        Ok(v) => info!("inserted {v} rows into sqlite db"),
                        Err(e) => error!("{e}"),
                    }
                }
                Pkt::NInfo(ref ni) => {
                    #[cfg(feature = "print-packets")]
                    println!("{}", to_string_pretty(&ni).unwrap());
                    let fake = state
                        .lock()
                        .expect("Failed to acquire lock for GatewayState in main()")
                        .find_fake_id(ni.num)
                        .map(|n| Some(n.into()))
                        .expect("No fake_id returned");
                    match update_metrics(&postgres_db, &pkt, fake, &deployment_loc)
                        .await
                        .with_context(|| {
                            "Failed to update postgres database with node info packet from serial"
                        }) {
                        Ok(v) => info!("inserted {v} rows into postgres db"),
                        Err(e) => {
                            // This is a lower priority error message since we favor node info data
                            // from the Mesh rather than from the serial connection. Often times it
                            // just means that we did not insert a row
                            info!("{e}");
                        }
                    }
                    match update_metrics(&sqlite_db, &pkt, fake, &deployment_loc)
                        .await
                        .with_context(|| {
                            "Failed to update sqlite database with node info packet from serial"
                        }) {
                        Ok(v) => info!("inserted {v} rows into sqlite db"),
                        Err(e) => {
                            // This is a lower priority error message since we favor node info data
                            // from the Mesh rather than from the serial connection. Often times it
                            // just means that we did not insert a row
                            info!("{e}");
                        }
                    }
                }
                Pkt::MyNodeInfo(ref mi) => {
                    #[cfg(feature = "print-packets")]
                    println!("{}", to_string_pretty(&mi).unwrap());
                }
            }
            // Thread has been used to process and send to DB, kill it
            let _res = join.await?;
            // Dumb sqlite optimizations
            few_hours = pragma_optimize(&sqlite_db, few_hours);
            daily = drop_old_rows(&sqlite_db, daily);
        }
    }

    // Note that in this specific example, this will only be called when
    // the radio is disconnected, as the above loop will never exit.
    // Typically you would allow the user to manually kill the loop,
    // for example with tokio::select!.
    let _stream_api = stream_api.disconnect().await?;

    Ok(())
}
