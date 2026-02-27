use crate::util::timestamp;
use anyhow::Context;
use meshtastic::protobufs::{ErrorMetrics, MeshPacket, Telemetry};
use serde::Serialize;
use serde_json::json;
use sqlx::postgres::types::Oid;

#[derive(Serialize)]
struct Errors {
    no_routes: Option<Oid>,
    naks: Option<Oid>,
    timeouts: Option<Oid>,
    max_retransmits: Option<Oid>,
    no_channels: Option<Oid>,
    too_large: Option<Oid>,
}

/// Insert a row into the `ErrorMetrics` table
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference
/// * `tm` - A `Telemetry` reference
/// * `em` - An `ErrorMetrics` struct
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn insert(
    pkt: &MeshPacket,
    tm: &Telemetry,
    em: &ErrorMetrics,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    sqlx::query!(
        "
INSERT INTO
  ErrorMetrics (
    msg_id,
    node_id,
    time,
    collision_rate,
    node_reach,
    num_nodes,
    usefulness,
    avg_delay,
    period,
    errors
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
    $10
  )
            ",
        Oid(pkt.id),
        Oid(pkt.from),
        timestamp(tm.time),
        em.collision_rate,
        em.node_reach,
        em.num_nodes.map(Oid),
        em.usefulness,
        em.avg_delay.map(Oid),
        em.period.map(Oid),
        Some(json!(&Errors {
            no_routes: em.noroute.map(Oid),
            naks: em.naks.map(Oid),
            timeouts: em.timeouts.map(Oid),
            max_retransmits: em.max_retransmit.map(Oid),
            no_channels: em.no_channel.map(Oid),
            too_large: em.too_large.map(Oid)
        })),
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .context("Failed to insert row into ErrorMetrics table")
}
