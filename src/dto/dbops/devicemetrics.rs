use crate::dto::types::timestamp;
use anyhow::Context;
use meshtastic::protobufs::{DeviceMetrics, MeshPacket, Position, Telemetry};
use sqlx::postgres::types::Oid;

/// Insert a row into the `DeviceMetrics` table
///
/// # Arguments
/// * `dm` - A `Devicemetric` struct
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn insert_dm(
    pkt: &MeshPacket,
    tm: &Telemetry,
    dm: &DeviceMetrics,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    sqlx::query!(
        "
INSERT INTO
  DeviceMetrics (
    msg_id,
    node_id,
    time,
    battery_levels,
    voltage,
    channelutil,
    airutil
)
VALUES
  (
    $1,
    $2,
    $3,
    $4,
    $5,
    $6,
    $7
  )
            ",
        Oid(pkt.id),
        Oid(pkt.from),
        timestamp(tm.time),
        dm.battery_level.map(Oid),
        dm.voltage,
        dm.channel_utilization,
        dm.air_util_tx,
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .with_context(|| "Failed to insert row into DeviceMetrics table")
}

/// Insert a row into the `DeviceMetrics` table
///
/// # Arguments
/// * `dm` - A `Devicemetric` struct
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn insert_pos(
    pkt: &MeshPacket,
    pos: &Position,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    sqlx::query!(
        "
INSERT INTO
  DeviceMetrics (
    msg_id,
    node_id,
    time,
    latitude,
    longitude
)
VALUES
  (
    $1,
    $2,
    $3,
    $4,
    $5
  )
            ",
        Oid(pkt.id),
        Oid(pkt.from),
        timestamp(pos.timestamp), //TODO: do I need ms adjustments?
        pos.latitude_i,
        pos.longitude_i,
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .with_context(|| "Failed to insert row into DeviceMetrics table")
}

/// Update a row in the `DeviceMetrics` table
///
/// # Arguments
/// * `dm` - A `Devicemetric` struct
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
async fn update(
    pkt: &MeshPacket,
    tm: &Telemetry,
    dm: &DeviceMetrics,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    sqlx::query!(
        "
UPDATE DeviceMetrics
SET
  node_id = $2,
  time = $3,
  battery_levels = $4,
  voltage = $5,
  channelutil = $6,
  airutil = $7
WHERE
  msg_id = $1
            ",
        Oid(pkt.id),
        Oid(pkt.from),
        timestamp(tm.time),
        dm.battery_level.map(Oid),
        dm.voltage,
        dm.channel_utilization,
        dm.air_util_tx,
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .with_context(|| "Failed to update row in DeviceMetrics table")
}
