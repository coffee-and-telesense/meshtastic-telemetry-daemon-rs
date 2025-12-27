use crate::dto::types::DbOps;
use anyhow::Context;

#[derive(derive_new::new, sqlx::FromRow)]
pub struct Localstat {
    msg_id: sqlx::postgres::types::Oid,
    node_id: sqlx::postgres::types::Oid,
    time: chrono::NaiveDateTime,
    uptime_seconds: Option<sqlx::postgres::types::Oid>,
    channel_util: Option<f32>,
    air_util_tx: Option<f32>,
    num_packets_tx: Option<sqlx::postgres::types::Oid>,
    num_packets_rx: Option<sqlx::postgres::types::Oid>,
    num_packets_rx_bad: Option<sqlx::postgres::types::Oid>,
    num_online_nodes: Option<sqlx::postgres::types::Oid>,
    num_total_nodes: Option<sqlx::postgres::types::Oid>,
    num_rx_dupe: Option<sqlx::postgres::types::Oid>,
    num_tx_relay: Option<sqlx::postgres::types::Oid>,
    num_tx_relay_canceled: Option<sqlx::postgres::types::Oid>,
}

impl DbOps for Localstat {
    /// Insert a row into the `LocalStats` table
    ///
    /// # Arguments
    /// * `self` - A `Localstat` struct
    /// * `pool` - A `Pool<Postgres>` reference
    ///
    /// # Returns
    /// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
    async fn insert(
        &self,
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
            self.msg_id,
            self.node_id,
            self.time,
            self.uptime_seconds,
            self.channel_util,
            self.air_util_tx,
            self.num_packets_tx,
            self.num_packets_rx,
            self.num_packets_rx_bad,
            self.num_online_nodes,
            self.num_total_nodes,
            self.num_rx_dupe,
            self.num_tx_relay,
            self.num_tx_relay_canceled
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to insert row into LocalStats table")
    }

    /// Update a row in the `LocalStats` table
    ///
    /// # Arguments
    /// * `self` - A `Localstat` struct
    /// * `pool` - A `Pool<Postgres>` reference
    ///
    /// # Returns
    /// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
    async fn update(
        &self,
        pool: &sqlx::Pool<sqlx::Postgres>,
    ) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
        sqlx::query!(
            "
UPDATE LocalStats
SET
  node_id = $2,
  TIME = $3,
  uptime_seconds = $4,
  channel_util = $5,
  air_util_tx = $6,
  num_packets_tx = $7,
  num_packets_rx = $8,
  num_packets_rx_bad = $9,
  num_online_nodes = $10,
  num_total_nodes = $11,
  num_rx_dupe = $12,
  num_tx_relay = $13,
  num_tx_relay_canceled = $14
WHERE
  msg_id = $1
            ",
            self.msg_id,
            self.node_id,
            self.time,
            self.uptime_seconds,
            self.channel_util,
            self.air_util_tx,
            self.num_packets_tx,
            self.num_packets_rx,
            self.num_packets_rx_bad,
            self.num_online_nodes,
            self.num_total_nodes,
            self.num_rx_dupe,
            self.num_tx_relay,
            self.num_tx_relay_canceled
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to update row in LocalStats table")
    }
}
