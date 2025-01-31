use std::{
    collections::HashMap,
    io::{self, BufRead},
    sync::{Arc, Mutex},
};

use db_poster::AddData;
use meshtastic::api::StreamApi;
use meshtastic::utils;
use serde_json::to_string_pretty;
use tokio_postgres::{Config, NoTls};
use types::GatewayState;

mod db_poster;
mod packet_handler;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the gateway's state object
    let state = Arc::new(Mutex::new(GatewayState::new()));

    // Configure postgres connection
    let mut db_config = Config::new();
    // hardcoded, this is BAD but only PoC
    db_config.user("postgres");
    db_config.password("postgres");
    db_config.port(5431);
    db_config.ssl_mode(tokio_postgres::config::SslMode::Disable); // also BAD, need TLS in prod
    db_config.host("localhost");
    db_config.dbname("meshtastic");

    // Connect to postgres db
    let (client, connection) = db_config.connect(NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Connect to serial meshtastic
    let stream_api = StreamApi::new();

    let available_ports = utils::stream::available_serial_ports()?;
    println!("Available ports: {:?}", available_ports);
    println!("Enter the name of a port to connect to:");

    let stdin = io::stdin();
    let entered_port = stdin
        .lock()
        .lines()
        .next()
        .expect("Failed to find next line")
        .expect("Could not read next line");

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
                    println!("inserted {:?} rows", res);
                }
                types::Pkt::NInfo(ni) => {
                    println!("{}", to_string_pretty(&ni).unwrap());
                    let fake: u32 = state.lock().unwrap().find_fake_id(ni.num).unwrap().into();
                    let res = client.update_metrics(pkt, Some(fake)).await;
                    println!("inserted {:?} rows", res);
                }
                types::Pkt::MyNodeInfo(mi) => {
                    println!("{}", to_string_pretty(&mi).unwrap());
                }
            }
        }
        //println!("Received: {:?}", decoded);
    }

    // Note that in this specific example, this will only be called when
    // the radio is disconnected, as the above loop will never exit.
    // Typically you would allow the user to manually kill the loop,
    // for example with tokio::select!.
    let _stream_api = stream_api.disconnect().await?;

    Ok(())
}
