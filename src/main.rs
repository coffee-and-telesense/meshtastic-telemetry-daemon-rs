#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![expect(
    clippy::multiple_crate_versions,
    reason = "diesel and config pull different version of core deps"
)]

//! `embedded_nano_mesh` to `PostgreSQL` database daemon

use crate::{
    dto::db_writer,
    util::{
        config::{DEPLOYMENT_LOCATION, PgPool, Settings},
        log::set_logger,
        state::GatewayState,
        to_anyhow_err,
    },
};
use anyhow::{Context as _, Error, Result, anyhow};
use embedded_nano_mesh::Packet;
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;
use signal_hook::{
    consts::{SIGINT, SIGTERM},
    flag::register,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::Relaxed},
        mpsc,
    },
    thread,
    time::Instant,
};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Handle data transfer objects
pub(crate) mod dto;
/// Database schema file
pub(crate) mod schema;
/// Utilities module
pub(crate) mod util;

/// Version number of the daemon
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Capacity of the bounded channel between serial and DB threads.
/// At 9600 baud a burst of 32 packets is extremely conservative.
const CHANNEL_BOUND: usize = 32;

fn main() -> Result<(), Error> {
    // Program start time needed for embedded_nano_mesh serial driver
    let program_start_time = Instant::now();

    // Set the logger
    set_logger()?;

    // Output the version of the daemon to the logger
    tracing::info!("Daemon version: {VERSION}");

    // Read settings
    let settings = Settings::new().context("Error initializing Settings")?;

    // Setup serial connection
    let (mut node, mut serial) = settings.setup_serial()?;

    // Create the gateway's state object
    let state = Arc::new(GatewayState::new());

    // Create PostgreSQL connection
    let postgres_db: PgPool = settings
        .setup_postgres()
        .context("Failed to connect to postgresql database")?;

    // Set the global deployment location string
    DEPLOYMENT_LOCATION
        .set(settings.deployment.location)
        .map_err(|e| anyhow!("DEPLOYMENT_LOCATION initialized twice: {e}"))?;

    // Load the already filled in nodeinfo tables to the state
    state.load_from_db(&postgres_db)?;

    // Set the connected node's serial
    state.set_serial_number(node.get_address());

    // Channel for sending packets to database handler from serial
    let (tx, rx) = mpsc::sync_channel::<Packet>(CHANNEL_BOUND);

    // Database writer thread
    let db_state = Arc::clone(&state);
    let db_pool = postgres_db;
    let db_thread = thread::spawn(move || {
        db_writer(rx, &db_pool, &db_state);
    });

    // Handle signals
    let shutdown = Arc::new(AtomicBool::new(false));
    register(SIGINT, Arc::clone(&shutdown)).context("Failed to register SIGINT handler")?;
    register(SIGTERM, Arc::clone(&shutdown)).context("Failed to register SIGTERM handler")?;

    // Receive packets over serial loop
    while !shutdown.load(Relaxed) {
        if let Some(packet) = node.receive()
            && packet.get_spec_state() == embedded_nano_mesh::PacketState::Normal
        {
            //TODO: send Packet types instead so we have access to the headers
            match tx
                .send(packet)
                .context("Failed to send packet over channel")
            {
                Ok(()) => (),
                Err(e) => tracing::error!(%e),
            }
        }

        #[expect(
            clippy::cast_possible_truncation,
            reason = "Truncating is expected behavior here, we want the lower 32 bits"
        )]
        match node
            .update(
                &mut serial,
                Instant::now()
                    .duration_since(program_start_time)
                    .as_millis() as u32,
            )
            .map_err(to_anyhow_err)
        {
            Ok(()) => (),
            Err(e) => {
                tracing::error!("{e}");
                return Err(e);
            }
        }
    }

    // Logging around shutdown
    tracing::warn!("Shutdown signal received");
    drop(tx);
    tracing::info!("Waiting for in-flight database writes to complete...");
    if let Err(e) = db_thread.join() {
        tracing::error!("Database writer thread panicked: {e:?}");
    }
    tracing::info!("Clean shutdown complete");

    Ok(())
}
