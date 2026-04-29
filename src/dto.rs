//! Packet handling and database operations

use crate::schema::{nano_mesh_nodes, node_stats, sensor_readings};
use crate::util::{
    config::{DEPLOYMENT_LOCATION, PgPool},
    state::GatewayState,
    timestamp,
};
use diesel::prelude::*;
use embedded_nano_mesh::Packet;
use nano_mesh_telemetry::{MeasurementKind, TelemetryPacket};
use std::sync::{Arc, mpsc::Receiver};

#[derive(Insertable, Debug)]
#[diesel(table_name = nano_mesh_nodes)]
struct NewNanoMeshNode<'a> {
    node_id: i16,
    name: Option<&'a str>,
    deployment_location: &'a str,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = sensor_readings)]
struct NewSensorReading<'a> {
    node_id: i16,
    epoch: i64,
    sensor_id: i16,
    kind: i16,
    value: f32,
    deployment_location: &'a str,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = node_stats)]
struct NewNodeStats<'a> {
    node_id: i16,
    epoch: i64,
    reboot_count: i16,
    tx_fail: i32,
    rx_drop: i32,
    rx_useful: i32,
    rx_overlap: i32,
    queue_full: i32,
    rx_bad: i32,
    num_online_nodes: i16,
    num_total_nodes: i16,
    channel_util: f32,
    air_util_tx: f32,
    deployment_location: &'a str,
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
    let epoch = timestamp(packet.epoch).and_utc().timestamp();
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
        epoch: timestamp(packet.epoch).and_utc().timestamp(),
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
pub(crate) fn db_writer(rx: Receiver<Packet>, db_pool: &PgPool, state: &Arc<GatewayState>) {
    let location = DEPLOYMENT_LOCATION.get().map_or("unknown", String::as_str);

    //TODO: source_node_id should come from packet header, not packet data
    // embedded-nano-mesh PacketDataBytes does not carry the sender address
    // This will need the full Packet type to be sent over the channel
    for packet in rx {
        let Some(telemetry) = TelemetryPacket::from_packet_data(&packet.data) else {
            tracing::warn!("Failed to deserialize packet — skipping");
            continue;
        };

        let Ok(mut conn) = db_pool.get() else {
            tracing::error!("Failed to get DB connection from pool — dropping packet");
            continue;
        };

        match telemetry {
            TelemetryPacket::Sensor(sensor_packet) => {
                ensure_node(&mut conn, packet.source_device_identifier, location);
                insert_sensor_packet(
                    &mut conn,
                    packet.source_device_identifier,
                    &sensor_packet,
                    location,
                );
                state.increment_count(packet.source_device_identifier);
            }
            TelemetryPacket::NodeStats(node_stats_packet) => {
                ensure_node(&mut conn, packet.source_device_identifier, location);
                insert_node_stats(
                    &mut conn,
                    packet.source_device_identifier,
                    &node_stats_packet,
                    location,
                );
                state.increment_count(packet.source_device_identifier);
            }
        }

        #[cfg(feature = "debug")]
        if state.any_recvd() {
            tracing::info!("{state}");
        }
    }

    tracing::info!("Database writer channel closed, exiting");
}
