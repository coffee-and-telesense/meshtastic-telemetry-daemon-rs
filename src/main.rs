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
use syslog::{BasicLogger, Facility, Formatter3164};

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
    let rv = config::Config::builder()
        .add_source(config::File::with_name(p))
        .build()
        .unwrap();
    #[cfg(debug_assertions)]
    println!(
        "{:?}",
        rv.clone()
            .try_deserialize::<HashMap<String, String>>()
            .unwrap()
    );
    rv
}

fn setup_db(cfg: &Config) -> tokio_postgres::Config {
    // Configure postgres connection
    let mut db_config = tokio_postgres::Config::new();
    // hardcoded, this is BAD but only PoC
    db_config.user(cfg.get_string("user").unwrap().as_str());
    db_config.password(cfg.get_string("password").unwrap().as_str());
    db_config.port(cfg.get::<u16>("port").unwrap());
    if cfg.get::<bool>("use_ssl").unwrap() {
        //TODO: use ssl
    } else {
        db_config.ssl_mode(tokio_postgres::config::SslMode::Disable);
    }
    db_config.host(cfg.get_string("host").unwrap().as_str());
    //db_config.host("10.57.247.124"); // on AREDN
    db_config.dbname(cfg.get_string("dbname").unwrap().as_str());
    db_config
}

fn get_serial_port(cfg: &Config) -> Result<String, Box<dyn std::error::Error>> {
    if cfg.get_string("serial_port").is_err() {
        let available_ports = utils::stream::available_serial_ports()?;
        println!("Available ports: {:?}", available_ports);
        println!("Enter the name of a port to connect to:");

        let stdin = io::stdin();
        let rv = stdin
            .lock()
            .lines()
            .next()
            .expect("Failed to find next line")?;
        Ok(rv)
    } else {
        let rv = cfg.get_string("serial_port")?;
        Ok(rv)
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
    let logger = syslog::unix(formatter).expect("could not connect to syslog");
    let _ = log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(LevelFilter::Debug));

    // Create the gateway's state object
    let state = Arc::new(Mutex::new(GatewayState::new()));

    // Connect to postgres db
    let (client, connection) = setup_db(&settings).connect(NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Connect to serial meshtastic
    let stream_api = StreamApi::new();
    let entered_port = get_serial_port(&settings).expect("Could not read next line");
    let serial_stream = utils::stream::build_serial_stream(entered_port, None, None, None)?;
    let (mut decoded_listener, stream_api) = stream_api.connect(serial_stream).await;

    let config_id = utils::generate_rand_id();
    let stream_api = stream_api.configure(config_id).await?;

    // This loop can be broken with ctrl+c, or by disconnecting
    // the attached serial port.
    while let Some(decoded) = decoded_listener.recv().await {
        if let Some(pkt) = packet_handler::process_packet(decoded.clone(), Arc::clone(&state)) {
            match pkt.clone() {
                types::Pkt::Mesh(mp) => {
                    println!("{}", to_string_pretty(&mp).unwrap());
                    let res = client.update_metrics(pkt, None).await;
                    info!("inserted {:?} rows", res);
                }
                types::Pkt::NInfo(ni) => {
                    println!("{}", to_string_pretty(&ni).unwrap());
                    let fake: u32 = state.lock().unwrap().find_fake_id(ni.num).unwrap().into();
                    let res = client.update_metrics(pkt, Some(fake)).await;
                    info!("inserted {:?} rows", res);
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
