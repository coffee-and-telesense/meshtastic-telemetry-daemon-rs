use crate::util::timestamp;
use anyhow::Context;
use meshtastic::protobufs::{MeshPacket, PowerMetrics, Telemetry};
use sqlx::postgres::types::Oid;

/// Insert a row into the `PowerMetrics` table
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference
/// * `tm` - A `Telemetry` reference
/// * `pwr` - An `PowerMetrics` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn insert(
    pkt: &MeshPacket,
    tm: &Telemetry,
    pwr: &PowerMetrics,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    sqlx::query!(
        "
INSERT INTO
  PowerMetrics (
    msg_id,
    node_id,
    time,
    ch1_voltage,
    ch1_current,
    ch2_voltage,
    ch2_current,
    ch3_voltage,
    ch3_current
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
    $9
  )
            ",
        Oid(pkt.id),
        Oid(pkt.from),
        timestamp(tm.time),
        pwr.ch1_voltage,
        pwr.ch1_current,
        pwr.ch2_voltage,
        pwr.ch2_current,
        pwr.ch3_voltage,
        pwr.ch3_current,
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .context("Failed to insert row into PowerMetrics table")
}
