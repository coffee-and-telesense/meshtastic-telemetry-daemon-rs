use crate::dto::types::timestamp;
use anyhow::Context;
use meshtastic::protobufs::{MeshPacket, Neighbor, NeighborInfo};
use serde_json::{Value, json};
use sqlx::postgres::types::Oid;

/// Insert a row into the `NeighborInfo` table
///
/// # Arguments
/// * `pkt` - A `MeshPacket` reference
/// * `nbi` - A `NeighborInfo` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn insert(
    pkt: &MeshPacket,
    nbi: &NeighborInfo,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    let neighbors = nbi
        .neighbors
        .iter()
        .map(|n| {
            json!(&Neighbor {
                node_id: n.node_id,
                snr: n.snr,
                last_rx_time: n.last_rx_time,
                node_broadcast_interval_secs: n.node_broadcast_interval_secs,
                num_packets_rx: n.num_packets_rx,
                rssi: n.rssi,
            })
        })
        .collect::<Vec<Value>>();

    sqlx::query!(
        "
INSERT INTO
  NeighborInfo (
    msg_id,
    node_id,
    time,
    last_sent_by_id,
    node_broadcast_interval_secs,
    neighbors
)
VALUES
  (
    $1,
    $2,
    $3,
    $4,
    $5,
    $6
  )
            ",
        Oid(pkt.id),
        Oid(pkt.from),
        timestamp(pkt.rx_time),
        Some(Oid(nbi.last_sent_by_id)),
        Some(Oid(nbi.node_broadcast_interval_secs)),
        Some(neighbors.as_slice()),
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .with_context(|| "Failed to insert row into NeighborInfo table")
}
