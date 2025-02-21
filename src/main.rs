use std::{
    io::{self, BufRead},
    sync::{Arc, Mutex},
};

#[cfg(feature = "debug")]
use log::{error, info, warn};
#[cfg(debug_assertions)]
use std::collections::HashMap;
#[cfg(not(debug_assertions))]
extern crate syslog;
#[cfg(not(debug_assertions))]
#[macro_use]
extern crate log;
use anyhow::{Context, Result};
use config::Config;
use db_poster::update_metrics;
#[cfg(not(debug_assertions))]
use log::LevelFilter;
use meshtastic::api::StreamApi;
use meshtastic::utils;
use sea_orm::{ConnectOptions, Database};
use serde::Deserialize;
use serde_json::to_string_pretty;
#[cfg(not(debug_assertions))]
use syslog::{BasicLogger, Facility, Formatter3164};
use types::GatewayState;

mod db_poster;
mod entities;
mod packet_handler;
mod types;

fn read_config(p: &str) -> config::Config {
    match config::Config::builder()
        .add_source(config::File::with_name(p))
        .build()
        .with_context(|| format!("Failed to read config file from {}", p))
    {
        Ok(rv) => {
            #[cfg(debug_assertions)]
            println!(
                "{:?}",
                rv.clone()
                    .try_deserialize::<HashMap<String, String>>()
                    .unwrap()
            );
            rv
        }
        Err(e) => {
            panic!("{:#}", e);
        }
    }
}

fn get_cfg<'d, T: Deserialize<'d>>(cfg: &Config, key: &str) -> T {
    match cfg
        .get::<T>(key)
        .with_context(|| format!("Failed to read {} from config", key))
    {
        Ok(rv) => rv,
        Err(e) => {
            panic!("{:?}", e);
        }
    }
}

fn get_cfg_string(cfg: &Config, key: &str) -> String {
    match cfg
        .get_string(key)
        .with_context(|| format!("Failed to read {} from config", key))
    {
        Ok(rv) => rv,
        Err(e) => {
            panic!("{:?}", e);
        }
    }
}

fn db_connection(cfg: &Config) -> String {
    // Parse config with error handling
    format!(
        "postgres://{}:{}@{}:{}/{}",
        get_cfg_string(cfg, "user"),
        get_cfg_string(cfg, "password"),
        get_cfg_string(cfg, "host"),
        get_cfg::<u16>(cfg, "port"),
        get_cfg_string(cfg, "dbname")
    )
}

fn get_serial_port(cfg: &Config) -> String {
    match cfg
        .get_string("serial_port")
        .with_context(|| "Failed to read serial_port from config file")
    {
        Err(e) => {
            #[cfg(debug_assertions)]
            {
                eprintln!("{:#}", e);
                eprintln!("Prompting user for serial port instead");
            }
            #[cfg(not(debug_assertions))]
            {
                error!("{:#}", e);
                warn!("Prompting user for serial port instead");
            }
            match utils::stream::available_serial_ports()
                .with_context(|| "Failed to enumerate list of serial ports")
            {
                Ok(ap) => println!("Available ports: {:?}", ap),
                Err(e) => {
                    error!("{:#}", e);
                    warn!("User will input their own serial port");
                }
            }
            println!("Enter the name of a port to connect to:");

            let stdin = io::stdin();
            match stdin
                .lock()
                .lines()
                .next()
                .expect("Failed to find next line")
                .with_context(|| "Could not read from stdin")
            {
                Ok(sp) => sp,
                Err(e) => {
                    eprintln!("No serial port provided by user");
                    panic!("{:#}", e);
                }
            }
        }
        Ok(sp) => sp,
    }
}

fn set_logger() {
    #[cfg(feature = "debug")]
    {
        colog::init();
    }
    #[cfg(not(debug_assertions))]
    {
        let formatter = Formatter3164 {
            facility: Facility::LOG_USER, //TODO: this could probably be something else, check libc
            hostname: None,
            process: "mesh_telem".into(),
            pid: 0,
        };
        match syslog::unix(formatter).with_context(|| "Could not connect to syslog posix socket") {
            Ok(logger) => {
                //TODO: should be warn on actual release instead of Info
                let _ = log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
                    .map(|()| log::set_max_level(LevelFilter::Info))
                    .with_context(|| "Failed to set logger to syslog")
                    .inspect_err(|e| {
                        error!("{:#}", e);
                        warn!("Continuing execution");
                    });
            }
            Err(e) => {
                error!("{:#}", e);
                warn!("Continuing execution");
            }
        }
    }
}

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
    let mut opt = ConnectOptions::new(db_connection(&settings));
    opt.sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);
    let db = Database::connect(opt)
        .await
        .with_context(|| "Failed to connect to the database")?;

    // Connect to serial meshtastic
    let stream_api = StreamApi::new();
    let entered_port = get_serial_port(&settings);
    let serial_stream = utils::stream::build_serial_stream(entered_port.clone(), None, None, None)
        .with_context(|| format!("Failed to build serial stream for {}", entered_port))?;
    let (mut decoded_listener, stream_api) = stream_api.connect(serial_stream).await;

    let config_id = utils::generate_rand_id();
    let stream_api = stream_api
        .configure(config_id)
        .await
        .with_context(|| "Failed to configure serial stream")?;

    let deployment_loc = get_cfg_string(&settings, "deployment_location");
    // This loop can be broken with ctrl+c, or by disconnecting
    // the attached serial port.
    while let Some(decoded) = decoded_listener.recv().await {
        if let Some(pkt) = packet_handler::process_packet(decoded.clone(), Arc::clone(&state)) {
            match pkt.clone() {
                types::Pkt::Mesh(mp) => {
                    println!("{}", to_string_pretty(&mp).unwrap());
                    let res = update_metrics(&db, pkt, None, &deployment_loc)
                        .await
                        .with_context(|| "Failed to update datatbase with packet from mesh");
                    match res {
                        Ok(v) => info!("inserted {} rows", v),
                        Err(e) => error!("{}", e),
                    }
                }
                types::Pkt::NInfo(ni) => {
                    println!("{}", to_string_pretty(&ni).unwrap());
                    let fake = state
                        .lock()
                        .unwrap()
                        .find_fake_id(ni.num)
                        .map(|n| Some(n.into()))
                        .expect("No fake_id returned");
                    let res = update_metrics(&db, pkt, fake, &deployment_loc)
                        .await
                        .with_context(|| {
                            "Failed to update database with node info packet from serial"
                        });
                    match res {
                        Ok(v) => info!("inserted {} rows", v),
                        Err(e) => {
                            // This is a lower priority error message since we favor node info data
                            // from the Mesh rather than from the serial connection. Often times it
                            // just means that we did not insert a row
                            info!("{:#}", e);
                        }
                    }
                }
                types::Pkt::MyNodeInfo(mi) => {
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
