use crate::dto::types::DbOps;
use anyhow::Context;

#[derive(derive_new::new, sqlx::FromRow)]
pub struct Errormetric {
    msg_id: sqlx::postgres::types::Oid,
    node_id: sqlx::postgres::types::Oid,
    time: chrono::NaiveDateTime,
    collision_rate: Option<f32>,
    node_reach: Option<f32>,
    num_nodes: Option<sqlx::postgres::types::Oid>,
    usefulness: Option<f32>,
    avg_delay: Option<sqlx::postgres::types::Oid>,
    period: Option<sqlx::postgres::types::Oid>,
    errors: Option<serde_json::Value>,
}

impl DbOps for Errormetric {
    /// Insert a row into the `ErrorMetrics` table
    ///
    /// # Arguments
    /// * `self` - An `Errormetric` struct
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
            self.msg_id,
            self.node_id,
            self.time,
            self.collision_rate,
            self.node_reach,
            self.num_nodes,
            self.usefulness,
            self.avg_delay,
            self.period,
            self.errors
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to insert row into ErrorMetrics table")
    }

    /// Update a row in the `ErrorMetrics` table
    ///
    /// # Arguments
    /// * `self` - An `Errormetric` struct
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
UPDATE ErrorMetrics
SET
  node_id = $2,
  time = $3,
  collision_rate = $4,
  node_reach = $5,
  num_nodes = $6,
  usefulness = $7,
  avg_delay = $8,
  period = $9,
  errors = $10
WHERE
  msg_id = $1
            ",
            self.msg_id,
            self.node_id,
            self.time,
            self.collision_rate,
            self.node_reach,
            self.num_nodes,
            self.usefulness,
            self.avg_delay,
            self.period,
            self.errors
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to update row in ErrorMetrics table")
    }
}
