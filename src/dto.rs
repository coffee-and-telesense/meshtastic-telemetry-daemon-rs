//! Packet handling and database operations

use crate::util::{config::PgPool, state::GatewayState};
use embedded_nano_mesh::PacketDataBytes;
use nano_mesh_telemetry::TelemetryPacket;
use std::sync::{Arc, mpsc::Receiver};

/// Receives packets from the serial thread and inserts them into `PostgreSQL`
pub(crate) fn db_writer(rx: Receiver<PacketDataBytes>, db_pool: PgPool, state: Arc<GatewayState>) {
    for packet in rx {
        if let Some(telemetry) = TelemetryPacket::from_packet_data(&packet) {
            match telemetry {
                TelemetryPacket::Sensor(sensor_packet) => {
                    //TODO: inserts
                }
                TelemetryPacket::NodeStats(node_stats_packet) => {
                    //TODO: inserts
                }
            }
        }
        #[cfg(feature = "debug")]
        if state.any_recvd() {
            tracing::info!("{state}");
        }
    }
    tracing::info!("Database writer channel closed, exiting");
}
