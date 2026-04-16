//! Packet handling and database operations

use crate::util::{config::PgPool, state::GatewayState};
use embedded_nano_mesh::PacketDataBytes;
use std::sync::{Arc, mpsc::Receiver};

/// Receives packets from the serial thread and inserts them into `PostgreSQL`
pub(crate) fn db_writer(rx: Receiver<PacketDataBytes>, db_pool: PgPool, state: Arc<GatewayState>) {
    for packet in rx {
        //TODO: packet handling and db inserts above this
        #[cfg(feature = "debug")]
        if state.any_recvd() {
            tracing::info!("{state}");
        }
        drop(packet);
    }
    tracing::info!("Database writer channel closed, exiting");
}
