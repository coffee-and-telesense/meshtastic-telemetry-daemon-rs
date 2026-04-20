//! Packet handling and database operations

use crate::schema::{nano_mesh_nodes, node_stats, sensor_readings};
use crate::util::{
    config::{DEPLOYMENT_LOCATION, PgPool},
    state::GatewayState,
    timestamp,
};
use diesel::prelude::*;
use embedded_nano_mesh::PacketDataBytes;
use nano_mesh_telemetry::{MeasurementKind, TelemetryPacket};
use std::sync::{Arc, mpsc::Receiver};

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = nano_mesh_nodes)]
pub struct NanoMeshNode {
    pub node_id: i16,
    pub name: Option<String>,
    pub deployment_location: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = nano_mesh_nodes)]
pub struct NewNanoMeshNode<'a> {
    pub node_id: i16,
    pub name: Option<&'a str>,
    pub deployment_location: &'a str,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = sensor_readings)]
pub struct NewSensorReading<'a> {
    pub node_id: i16,
    pub epoch: i64,
    pub sensor_id: i16,
    pub kind: i16,
    pub value: f32,
    pub deployment_location: &'a str,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = node_stats)]
pub struct NewNodeStats<'a> {
    pub node_id: i16,
    pub epoch: i64,
    pub reboot_count: i16,
    pub tx_fail: i32,
    pub rx_drop: i32,
    pub rx_useful: i32,
    pub rx_overlap: i32,
    pub queue_full: i32,
    pub rx_bad: i32,
    pub num_online_nodes: i16,
    pub num_total_nodes: i16,
    pub channel_util: f32,
    pub air_util_tx: f32,
    pub deployment_location: &'a str,
}

/// Insert a `SensorPacket` into `sensor_readings`.
///
/// Skips `Unknown` measurements and measurements beyond `count`.
/// Uses `ON CONFLICT DO NOTHING` to handle duplicate `(node_id, epoch, sensor_id, kind)`.
fn insert_sensor_packet(
    conn: &mut PgConnection,
    source_node_id: u8,
    packet: &nano_mesh_telemetry::SensorPacket,
    location: &str,
) {
    let node_id = i16::from(source_node_id);
    let epoch = i64::from(packet.epoch);
    let sensor_id = i16::from(u8::from(packet.sensor_id));
    let count = usize::from(packet.count).min(nano_mesh_telemetry::MAX_MEASUREMENTS);

    let readings: Vec<NewSensorReading<'_>> = packet
        .measurements
        .iter()
        .take(count)
        .filter(|m| m.kind != MeasurementKind::Unknown)
        .map(|m| NewSensorReading {
            node_id,
            epoch,
            sensor_id,
            kind: i16::from(u8::from(m.kind)),
            value: m.value,
            deployment_location: location,
        })
        .collect();

    if readings.is_empty() {
        return;
    }

    if let Err(e) = diesel::insert_into(sensor_readings::table)
        .values(&readings)
        .on_conflict_do_nothing()
        .execute(conn)
    {
        tracing::error!("Failed to insert sensor readings: {e}");
    }
}

/// Insert a `NodeStatsPacket` into `node_stats`.
///
/// Uses `ON CONFLICT DO NOTHING` to handle duplicate `(node_id, epoch)`.
fn insert_node_stats(
    conn: &mut PgConnection,
    source_node_id: u8,
    packet: &nano_mesh_telemetry::NodeStatsPacket,
    location: &str,
) {
    let row = NewNodeStats {
        node_id: i16::from(source_node_id),
        epoch: i64::from(packet.epoch),
        reboot_count: i16::from(packet.reboot_count),
        tx_fail: i32::from(packet.tx_fail),
        rx_drop: i32::from(packet.rx_drop),
        rx_useful: i32::from(packet.rx_useful),
        rx_overlap: i32::from(packet.rx_overlap),
        queue_full: i32::from(packet.queue_full),
        rx_bad: i32::from(packet.rx_bad),
        num_online_nodes: i16::from(packet.num_online_nodes),
        num_total_nodes: i16::from(packet.num_total_nodes),
        channel_util: packet.channel_util,
        air_util_tx: packet.air_util_tx,
        deployment_location: location,
    };

    if let Err(e) = diesel::insert_into(node_stats::table)
        .values(&row)
        .on_conflict_do_nothing()
        .execute(conn)
    {
        tracing::error!("Failed to insert node stats: {e}");
    }
}

/// Ensure a node exists in `nano_mesh_nodes`, inserting it if not.
///
/// Uses `ON CONFLICT DO NOTHING` — if the node already exists the insert
/// is silently skipped.
fn ensure_node(conn: &mut PgConnection, node_id: u8, location: &str) {
    let row = NewNanoMeshNode {
        node_id: i16::from(node_id),
        name: None,
        deployment_location: location,
    };

    if let Err(e) = diesel::insert_into(nano_mesh_nodes::table)
        .values(&row)
        .on_conflict_do_nothing()
        .execute(conn)
    {
        tracing::error!("Failed to ensure node {node_id} exists: {e}");
    }
}

/// Receives packets from the serial thread and inserts them into `PostgreSQL`.
pub(crate) fn db_writer(rx: Receiver<PacketDataBytes>, db_pool: PgPool, state: Arc<GatewayState>) {
    let location = DEPLOYMENT_LOCATION.get().map_or("unknown", String::as_str);

    //TODO: source_node_id should come from packet header, not packet data
    // embedded-nano-mesh PacketDataBytes does not carry the sender address
    // This will need the full Packet type to be sent over the channel
    for packet in rx {
        let Some(telemetry) = TelemetryPacket::from_packet_data(&packet) else {
            tracing::warn!("Failed to deserialize packet — skipping");
            continue;
        };

        let Ok(mut conn) = db_pool.get() else {
            tracing::error!("Failed to get DB connection from pool — dropping packet");
            continue;
        };

        match telemetry {
            TelemetryPacket::Sensor(sensor_packet) => {
                let source_node_id: u8 = 0;
                ensure_node(&mut conn, source_node_id, location);
                insert_sensor_packet(&mut conn, source_node_id, &sensor_packet, location);
                state.increment_count(u16::from(source_node_id));
            }
            TelemetryPacket::NodeStats(node_stats_packet) => {
                let source_node_id: u8 = 0;
                ensure_node(&mut conn, source_node_id, location);
                insert_node_stats(&mut conn, source_node_id, &node_stats_packet, location);
                state.increment_count(u16::from(source_node_id));
            }
        }

        #[cfg(feature = "debug")]
        if state.any_recvd() {
            tracing::info!("{state}");
        }
    }

    tracing::info!("Database writer channel closed, exiting");
}
