use std::{
    io::{self, BufRead},
    sync::{Arc, Mutex},
};

#[cfg(debug_assertions)]
use std::collections::HashMap;

extern crate syslog;
#[macro_use]
extern crate log;

use log::LevelFilter;
use serde::Deserialize;
use syslog::{BasicLogger, Facility, Formatter3164};

use anyhow::{Context, Result};
use config::Config;
use db_poster::AddData;
use meshtastic::api::StreamApi;
use meshtastic::utils;
use serde_json::to_string_pretty;
use tokio_postgres::NoTls;
use types::GatewayState;

mod db_poster;
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

fn setup_db(cfg: &Config) -> tokio_postgres::Config {
    // Configure postgres connection
    let mut db_config = tokio_postgres::Config::new();
    // Parse config with error handling
    db_config.user(get_cfg_string(cfg, "user").as_str());
    db_config.password(get_cfg_string(cfg, "password").as_str());
    db_config.port(get_cfg::<u16>(cfg, "port"));
    if get_cfg::<bool>(cfg, "use_ssl") {
        //TODO: use ssl
    } else {
        db_config.ssl_mode(tokio_postgres::config::SslMode::Disable);
    }
    db_config.host(get_cfg_string(cfg, "host").as_str());
    db_config.dbname(get_cfg_string(cfg, "dbname").as_str());
    db_config
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get settings from configuration file
    #[cfg(debug_assertions)]
    let settings = read_config("example_config.toml");
    #[cfg(not(debug_assertions))]
    let settings = read_config("/etc/meshtastic_telem.toml");

    // setup logging to systemd or logd
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER, //TODO: this could probably be something else, check libc
        hostname: None,
        process: "mesh_telem".into(),
        pid: 0,
    };
    match syslog::unix(formatter).with_context(|| "Could not connect to syslog posix socket") {
        Ok(logger) => {
            #[cfg(debug_assertions)]
            let _ = log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
                .map(|()| log::set_max_level(LevelFilter::Debug))
                .with_context(|| "Failed to set logger to syslog")
                .inspect_err(|e| {
                    #[cfg(debug_assertions)]
                    eprintln!("{:?}", e);
                });
            //TODO: should be warn on actual release instead of Info
            #[cfg(not(debug_assertions))]
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

    // Create the gateway's state object
    let state = Arc::new(Mutex::new(GatewayState::new()));

    // Connect to postgres db
    let (client, connection) = setup_db(&settings)
        .connect(NoTls)
        .await
        .with_context(|| "Could not initialize database connection from settings")?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection
            .await
            .with_context(|| "Database connection error")
        {
            panic!("{:#?}", e);
        }
    });

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

    // This loop can be broken with ctrl+c, or by disconnecting
    // the attached serial port.
    while let Some(decoded) = decoded_listener.recv().await {
        if let Some(pkt) = packet_handler::process_packet(decoded.clone(), Arc::clone(&state)) {
            match pkt.clone() {
                types::Pkt::Mesh(mp) => {
                    println!("{}", to_string_pretty(&mp).unwrap());
                    let res = client
                        .update_metrics(pkt, None)
                        .await
                        .with_context(|| "Failed to update datatbase with packet from mesh");
                    match res {
                        Ok(v) => info!("inserted {} rows", v),
                        Err(e) => error!("{}", e),
                    }
                }
                types::Pkt::NInfo(ni) => {
                    println!("{}", to_string_pretty(&ni).unwrap());
                    let fake: u32 = state.lock().unwrap().find_fake_id(ni.num).unwrap().into();
                    let res = client
                        .update_metrics(pkt, Some(fake))
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
