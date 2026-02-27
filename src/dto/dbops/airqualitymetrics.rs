use crate::util::timestamp;
use anyhow::Context;
use meshtastic::protobufs::{AirQualityMetrics, MeshPacket, Telemetry};
use sqlx::postgres::types::Oid;

/// Insert a row into the `AirQualityMetrics` table
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference
/// * `tm` - A `Telemetry` reference
/// * `aqm` - An `AirQualityMetrics` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn insert(
    pkt: &MeshPacket,
    tm: &Telemetry,
    aqm: &AirQualityMetrics,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    sqlx::query!(
        "
INSERT INTO
  AirQualityMetrics (
    msg_id,
    node_id,
    time,
    pm10standard,
    pm25standard,
    pm100standard,
    pm10environmental,
    pm25environmental,
    pm100environmental,
    particles03um,
    particles05um,
    particles10um,
    particles25um,
    particles50um,
    particles100um,
    co2,
    sensor_type
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
        aqm.pm10_standard.map(Oid),
        aqm.pm25_standard.map(Oid),
        aqm.pm100_standard.map(Oid),
        aqm.pm10_environmental.map(Oid),
        aqm.pm25_environmental.map(Oid),
        aqm.pm100_environmental.map(Oid),
        aqm.particles_03um.map(Oid),
        aqm.particles_05um.map(Oid),
        aqm.particles_10um.map(Oid),
        aqm.particles_25um.map(Oid),
        aqm.particles_50um.map(Oid),
        aqm.particles_100um.map(Oid),
        aqm.co2.map(Oid),
        aqm.sensor,
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .context("Failed to insert row into AirQualityMetrics table")
}
