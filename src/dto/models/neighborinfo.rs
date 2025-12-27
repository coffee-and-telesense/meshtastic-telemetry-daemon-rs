use crate::dto::types::DbOps;
use anyhow::Context;

#[derive(derive_new::new, sqlx::FromRow)]
pub struct Neighborinfo {
    msg_id: sqlx::postgres::types::Oid,
    node_id: sqlx::postgres::types::Oid,
    time: chrono::NaiveDateTime,
    last_sent_by_id: Option<sqlx::postgres::types::Oid>,
    node_broadcast_interval_secs: Option<sqlx::postgres::types::Oid>,
    neighbors: Option<Vec<serde_json::Value>>,
}

impl DbOps for Neighborinfo {
    /// Insert a row into the `NeighborInfo` table
    ///
    /// # Arguments
    /// * `self` - A `Neighborinfo` struct
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
            self.msg_id,
            self.node_id,
            self.time,
            self.last_sent_by_id,
            self.node_broadcast_interval_secs,
            self.neighbors.as_deref()
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to insert row into NeighborInfo table")
    }

    /// Update a row in the `NeighborInfo` table
    ///
    /// # Arguments
    /// * `self` - A `Neighborinfo` struct
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
UPDATE NeighborInfo
SET
  node_id = $2,
  time = $3,
  last_sent_by_id = $4,
  node_broadcast_interval_secs = $5,
  neighbors = $6
WHERE
  msg_id = $1
            ",
            self.msg_id,
            self.node_id,
            self.time,
            self.last_sent_by_id,
            self.node_broadcast_interval_secs,
            self.neighbors.as_deref()
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to update row in NeighborInfo table")
    }
}
