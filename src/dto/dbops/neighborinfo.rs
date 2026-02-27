use crate::util::timestamp;
use anyhow::Context;
use meshtastic::protobufs::{MeshPacket, NeighborInfo};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::postgres::types::Oid;

#[derive(Serialize)]
struct Neighbor {
    node_id: u32,
    snr: f32,
    last_rx_time: u32,
    node_broadcast_interval_secs: u32,
    num_packets_rx: u32,
    rssi: i32,
}

/// Insert a row into the `NeighborInfo` table from a `MeshPacket`
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
    .context("Failed to insert row into NeighborInfo table")
}
