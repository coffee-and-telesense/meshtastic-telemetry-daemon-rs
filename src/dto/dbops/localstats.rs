use crate::util::timestamp;
use anyhow::Context;
use meshtastic::protobufs::{LocalStats, MeshPacket, Telemetry};
use sqlx::postgres::types::Oid;

/// Insert a row into the `LocalStats` table
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference
/// * `tm` - A `Telemetry` reference
/// * `ls` - An `LocalStats` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn insert(
    pkt: &MeshPacket,
    tm: &Telemetry,
    ls: &LocalStats,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    sqlx::query!(
        "
INSERT INTO
  LocalStats (
    msg_id,
    node_id,
    time,
    uptime_seconds,
    channel_util,
    air_util_tx,
    num_packets_tx,
    num_packets_rx,
    num_packets_rx_bad,
    num_online_nodes,
    num_total_nodes,
    num_rx_dupe,
    num_tx_relay,
    num_tx_relay_canceled
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
    $14
  )
            ",
        Oid(pkt.id),
        Oid(pkt.from),
        timestamp(tm.time),
        Some(Oid(ls.uptime_seconds)),
        Some(ls.channel_utilization),
        Some(ls.air_util_tx),
        Some(Oid(ls.num_packets_tx)),
        Some(Oid(ls.num_packets_rx)),
        Some(Oid(ls.num_packets_rx_bad)),
        Some(Oid(ls.num_online_nodes)),
        Some(Oid(ls.num_total_nodes)),
        Some(Oid(ls.num_rx_dupe)),
        Some(Oid(ls.num_tx_relay)),
        Some(Oid(ls.num_tx_relay_canceled)),
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .with_context(|| "Failed to insert row into LocalStats table")
}
