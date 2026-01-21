use crate::util::timestamp;
use anyhow::Context;
use meshtastic::protobufs::{EnvironmentMetrics, MeshPacket, Telemetry};
use sqlx::postgres::types::Oid;

/// Insert a row into the `EnvironmentMetrics` table
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference
/// * `tm` - A `Telemetry` reference
/// * `enm` - An `EnvironmentMetrics` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn insert(
    pkt: &MeshPacket,
    tm: &Telemetry,
    enm: &EnvironmentMetrics,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    sqlx::query!(
        "
INSERT INTO
  EnvironmentMetrics (
    msg_id,
    node_id,
    TIME,
    temperature,
    relative_humidity,
    barometric_pressure,
    gas_resistance,
    iaq,
    wind_direction,
    wind_speed,
    wind_gust,
    wind_lull,
    rainfall_1h,
    rainfall_24h,
    sensor_type,
    voltage,
    current
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
    $12,
    $13,
    $14,
    $15,
    $16,
    $17
  )
            ",
        Oid(pkt.id),
        Oid(pkt.from),
        timestamp(tm.time),
        enm.temperature,
        enm.relative_humidity,
        enm.barometric_pressure,
        enm.gas_resistance,
        enm.iaq.map(Oid),
        enm.wind_direction.map(Oid),
        enm.wind_speed,
        enm.wind_gust,
        enm.wind_lull,
        enm.rainfall_1h,
        enm.rainfall_24h,
        enm.sensor,
        enm.voltage,
        enm.current,
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .with_context(|| "Failed to insert row into EnvironmentMetrics table")
}
