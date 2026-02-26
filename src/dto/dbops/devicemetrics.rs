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

/// Upsert (insert or update) a row in the `DeviceMetrics` table with node info data from a mesh packet
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference
/// * `ni` - A `NodeInfo` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn upsert_mp(
    pkt: &MeshPacket,
    ni: &NodeInfo,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    // Destructure
    let (battery, voltage, channel_util, air_util) =
        ni.device_metrics.map_or((None, None, None, None), |d| {
            (
                d.battery_level.map(Oid),
                d.voltage,
                d.channel_utilization,
                d.air_util_tx,
            )
        });
    let (lat, lon) = ni
        .position
        .map_or((None, None), |d| (d.latitude_i, d.longitude_i));

    sqlx::query!(
        "
INSERT INTO DeviceMetrics (
    msg_id, node_id, time,
    battery_levels, voltage, channelutil, airutil,
    latitude, longitude, longname, shortname, hwmodel
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
ON CONFLICT (msg_id) DO UPDATE SET
    node_id        = EXCLUDED.node_id,
    time           = EXCLUDED.time,
    battery_levels = EXCLUDED.battery_levels,
    voltage        = EXCLUDED.voltage,
    channelutil    = EXCLUDED.channelutil,
    airutil        = EXCLUDED.airutil,
    latitude       = EXCLUDED.latitude,
    longitude      = EXCLUDED.longitude,
    longname       = EXCLUDED.longname,
    shortname      = EXCLUDED.shortname,
    hwmodel        = EXCLUDED.hwmodel
        ",
        Oid(pkt.id),
        Oid(pkt.from),
        timestamp(pkt.rx_time),
        battery,
        voltage,
        channel_util,
        air_util,
        lat,
        lon,
        ni.user.as_ref().map(|u| u.long_name.as_str()),
        ni.user.as_ref().map(|u| u.short_name.as_str()),
        ni.user.as_ref().map(|u| u.hw_model),
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .and_then(|result| {
        if result.rows_affected() == 0 {
            Err(anyhow::anyhow!(
                "Upsert from MeshPacket matched 0 rows in DeviceMetrics"
            ))
        } else {
            Ok(result)
        }
    })
    .with_context(|| "Failed to upsert row in DeviceMetrics table from MeshPacket")
}

/// Upsert (insert or update) a row in the `DeviceMetrics` table with node info data from the serial interface
///
/// # Arguments
/// * `pkt` - A `FromRadio` reference
/// * `ni` - A `NodeInfo` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn upsert_fr(
    pkt: &FromRadio,
    ni: &NodeInfo,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    // Destructure
    let (battery, voltage, channel_util, air_util) =
        ni.device_metrics.map_or((None, None, None, None), |d| {
            (
                d.battery_level.map(Oid),
                d.voltage,
                d.channel_utilization,
                d.air_util_tx,
            )
        });
    let (lat, lon) = ni
        .position
        .map_or((None, None), |d| (d.latitude_i, d.longitude_i));

    sqlx::query!(
        "
INSERT INTO DeviceMetrics (
    msg_id, node_id, time,
    battery_levels, voltage, channelutil, airutil,
    latitude, longitude, longname, shortname, hwmodel
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
ON CONFLICT (msg_id) DO UPDATE SET
    node_id        = EXCLUDED.node_id,
    time           = EXCLUDED.time,
    battery_levels = EXCLUDED.battery_levels,
    voltage        = EXCLUDED.voltage,
    channelutil    = EXCLUDED.channelutil,
    airutil        = EXCLUDED.airutil,
    latitude       = EXCLUDED.latitude,
    longitude      = EXCLUDED.longitude,
    longname       = EXCLUDED.longname,
    shortname      = EXCLUDED.shortname,
    hwmodel        = EXCLUDED.hwmodel
        ",
        Oid(pkt.id),
        Oid(ni.num),
        timestamp(0),
        battery,
        voltage,
        channel_util,
        air_util,
        lat,
        lon,
        ni.user.as_ref().map(|u| u.long_name.as_str()),
        ni.user.as_ref().map(|u| u.short_name.as_str()),
        ni.user.as_ref().map(|u| u.hw_model),
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .and_then(|result| {
        if result.rows_affected() == 0 {
            Err(anyhow::anyhow!(
                "Upsert from serial matched 0 rows in DeviceMetrics"
            ))
        } else {
            Ok(result)
        }
    })
    .with_context(|| "Failed to upsert row in DeviceMetrics table from serial")
}
