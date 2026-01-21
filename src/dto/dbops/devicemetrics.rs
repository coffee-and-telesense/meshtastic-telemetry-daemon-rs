use crate::util::timestamp;
use anyhow::Context;
use meshtastic::protobufs::{DeviceMetrics, FromRadio, MeshPacket, NodeInfo, Position, Telemetry};
use sqlx::postgres::types::Oid;

/// Insert a row into the `DeviceMetrics` table with device metrics data
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference
/// * `tm` - A `Telemetry` reference
/// * `dm` - A `DeviceMetrics` reference
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

/// Insert a row into the `DeviceMetrics` table with position data
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference
/// * `pos` - A `Position` reference
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

/// Insert a row into the `DeviceMetrics` table with node info data from a mesh packet
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference
/// * `ni` - A `NodeInfo` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn insert_mp_ni(
    pkt: &MeshPacket,
    ni: &NodeInfo,
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
    airutil,
    latitude,
    longitude,
    longname,
    shortname,
    hwmodel
)
VALUES
  (
    $1,
    $2,
    $3,
    $4,
    $5,
    $6,
    $7,
    $8,
    $9,
    $10,
    $11,
    $12
  )
            ",
        Oid(pkt.id),
        Oid(pkt.from),
        timestamp(pkt.rx_time),
        ni.device_metrics.and_then(|d| d.battery_level.map(Oid)),
        ni.device_metrics.and_then(|d| d.voltage),
        ni.device_metrics.and_then(|d| d.channel_utilization),
        ni.device_metrics.and_then(|d| d.air_util_tx),
        ni.position.and_then(|l| l.latitude_i),
        ni.position.and_then(|l| l.longitude_i),
        ni.user.as_ref().map(|u| u.long_name.as_str()),
        ni.user.as_ref().map(|u| u.short_name.as_str()),
        ni.user.as_ref().map(|u| u.hw_model),
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .with_context(|| "Failed to insert row into DeviceMetrics table")
}

/// Update a row in the `DeviceMetrics` table with node info data from a mesh packet
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference
/// * `ni` - A `NodeInfo` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn update_mp_ni(
    pkt: &MeshPacket,
    ni: &NodeInfo,
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
  airutil = $7,
  latitude = $8,
  longitude = $9,
  longname = $10,
  shortname = $11,
  hwmodel = $12
WHERE
  msg_id = $1
            ",
        Oid(pkt.id),
        Oid(pkt.from),
        timestamp(pkt.rx_time),
        ni.device_metrics.and_then(|d| d.battery_level.map(Oid)),
        ni.device_metrics.and_then(|d| d.voltage),
        ni.device_metrics.and_then(|d| d.channel_utilization),
        ni.device_metrics.and_then(|d| d.air_util_tx),
        ni.position.and_then(|l| l.latitude_i),
        ni.position.and_then(|l| l.longitude_i),
        ni.user.as_ref().map(|u| u.long_name.as_str()),
        ni.user.as_ref().map(|u| u.short_name.as_str()),
        ni.user.as_ref().map(|u| u.hw_model),
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .with_context(|| "Failed to update row in DeviceMetrics table")
}

/// Insert a row into the `DeviceMetrics` table with node info data from the serial interface
///
/// # Arguments
/// * `pkt` - A `FromRadio` reference
/// * `ni` - A `NodeInfo` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn insert_fr_ni(
    pkt: &FromRadio,
    ni: &NodeInfo,
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
    airutil,
    latitude,
    longitude,
    longname,
    shortname,
    hwmodel
)
VALUES
  (
    $1,
    $2,
    $3,
    $4,
    $5,
    $6,
    $7,
    $8,
    $9,
    $10,
    $11,
    $12
  )
            ",
        Oid(pkt.id),
        Oid(ni.num),
        timestamp(0),
        ni.device_metrics.and_then(|d| d.battery_level.map(Oid)),
        ni.device_metrics.and_then(|d| d.voltage),
        ni.device_metrics.and_then(|d| d.channel_utilization),
        ni.device_metrics.and_then(|d| d.air_util_tx),
        ni.position.and_then(|l| l.latitude_i),
        ni.position.and_then(|l| l.longitude_i),
        ni.user.as_ref().map(|u| u.long_name.as_str()),
        ni.user.as_ref().map(|u| u.short_name.as_str()),
        ni.user.as_ref().map(|u| u.hw_model),
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .with_context(|| "Failed to insert row into DeviceMetrics table")
}

/// Update a row in the `DeviceMetrics` table with node info data from the serial interface
///
/// # Arguments
/// * `pkt` - A `FromRadio` reference
/// * `ni` - A `NodeInfo` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn update_fr_ni(
    pkt: &FromRadio,
    ni: &NodeInfo,
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
  airutil = $7,
  latitude = $8,
  longitude = $9,
  longname = $10,
  shortname = $11,
  hwmodel = $12
WHERE
  msg_id = $1
            ",
        Oid(pkt.id),
        Oid(ni.num),
        timestamp(0),
        ni.device_metrics.and_then(|d| d.battery_level.map(Oid)),
        ni.device_metrics.and_then(|d| d.voltage),
        ni.device_metrics.and_then(|d| d.channel_utilization),
        ni.device_metrics.and_then(|d| d.air_util_tx),
        ni.position.and_then(|l| l.latitude_i),
        ni.position.and_then(|l| l.longitude_i),
        ni.user.as_ref().map(|u| u.long_name.as_str()),
        ni.user.as_ref().map(|u| u.short_name.as_str()),
        ni.user.as_ref().map(|u| u.hw_model),
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .with_context(|| "Failed to update row in DeviceMetrics table")
}
