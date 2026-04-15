#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![expect(
    clippy::multiple_crate_versions,
    reason = "diesel and config pull different version of core deps"
)]

//! `embedded_nano_mesh` to `PostgreSQL` database daemon

use crate::util::{
    config::{DEPLOYMENT_LOCATION, PgPool, Settings},
    log::set_logger,
    state::GatewayState,
    to_anyhow_err,
};
use anyhow::{Context as _, Error, Result, anyhow};
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;
use std::{sync::Arc, time::Instant};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Handle data transfer objects
pub(crate) mod dto;
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

    // Output the version of the daemon to the logger
    tracing::info!("Daemon version: {VERSION}");

    // Load the already filled in nodeinfo tables to the state
    // state.load_from_db(&postgres_db)?;

    //TODO: handle Ctrl+C and other interrupts of serial connection
    loop {
        if let Some(packet) = node.receive() {
            //TODO: Dispatch to INSERT function thread pool
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

    Ok(())
}
