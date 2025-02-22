#![warn(missing_docs)]
#![warn(clippy::cargo)]
#![warn(clippy::pedantic)]
// Unfortunately we have duplicate dependencies with different versions
#![allow(clippy::multiple_crate_versions)]

use crate::db::connection::update_metrics;
use crate::dto::packet_handler;
use crate::util::{
    config::{build_db_connection_string, get_cfg_string, get_serial_port, read_config},
    log::set_logger,
    types::{GatewayState, Pkt},
};
use anyhow::{Context, Result};
#[cfg(feature = "debug")]
use log::{error, info, warn};
use meshtastic::api::StreamApi;
use meshtastic::utils;
use sea_orm::{ConnectOptions, Database};
#[cfg(feature = "print-packets")]
use serde_json::to_string_pretty;
use std::sync::{Arc, Mutex};
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    let settings = read_config("example_config.toml");
    #[cfg(not(debug_assertions))]
    let settings = read_config("/etc/meshtastic_telem.toml");

    set_logger();

    // Create the gateway's state object
    let state = Arc::new(Mutex::new(GatewayState::new()));

    // Connect to postgres db
    let mut opt = ConnectOptions::new(build_db_connection_string(&settings));
    opt.sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);
    let db = Database::connect(opt)
        .await
        .with_context(|| "Failed to connect to the database")?;

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

    let (tx, mut rx) = mpsc::channel(32);

    // This loop can be broken with ctrl+c, or by disconnecting
    // the attached serial port.
    while let Some(decoded) = decoded_listener.recv().await {
        let st = Arc::clone(&state);
        let dc = decoded.clone();
        let tx2 = tx.clone();
        tokio::spawn(async move {
            tx2.send(packet_handler::process_packet(&dc, &st))
                .await
                .unwrap();
        });
        if let Some(pkt) = rx.recv().await.unwrap() {
            match pkt.clone() {
                Pkt::Mesh(mp) => {
                    #[cfg(feature = "print-packets")]
                    println!("{}", to_string_pretty(&mp).unwrap());
                    let res = update_metrics(&db, pkt, None, &deployment_loc)
                        .await
                        .with_context(|| "Failed to update datatbase with packet from mesh");
                    match res {
                        Ok(v) => info!("inserted {v} rows"),
                        Err(e) => error!("{e:#}"),
                    }
                }
                Pkt::NInfo(ni) => {
                    #[cfg(feature = "print-packets")]
                    println!("{}", to_string_pretty(&ni).unwrap());
                    let fake = state
                        .lock()
                        .expect("Failed to acquire lock for GatewayState in main()")
                        .find_fake_id(ni.num)
                        .map(|n| Some(n.into()))
                        .expect("No fake_id returned");
                    let res = update_metrics(&db, pkt, fake, &deployment_loc)
                        .await
                        .with_context(|| {
                            "Failed to update database with node info packet from serial"
                        });
                    match res {
                        Ok(v) => info!("inserted {v} rows"),
                        Err(e) => {
                            // This is a lower priority error message since we favor node info data
                            // from the Mesh rather than from the serial connection. Often times it
                            // just means that we did not insert a row
                            info!("{e:#}");
                        }
                    }
                }
                Pkt::MyNodeInfo(mi) => {
                    #[cfg(feature = "print-packets")]
                    println!("{}", to_string_pretty(&mi).unwrap());
                }
            }
        }
    }

    // Note that in this specific example, this will only be called when
    // the radio is disconnected, as the above loop will never exit.
    // Typically you would allow the user to manually kill the loop,
    // for example with tokio::select!.
    let _stream_api = stream_api.disconnect().await?;

    Ok(())
}
