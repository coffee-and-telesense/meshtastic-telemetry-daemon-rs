use crate::dto::types::DbOps;
use anyhow::Context;

#[derive(derive_new::new, sqlx::FromRow)]
pub struct Nodeinfo {
    node_id: sqlx::postgres::types::Oid,
    longname: String,
    shortname: String,
    hwmodel: i32,
    deployment_location: String,
}

impl DbOps for Nodeinfo {
    /// Insert a row into the `NodeInfo` table
    ///
    /// # Arguments
    /// * `self` - A `Nodeinfo` struct
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
  NodeInfo (
    node_id,
    longname,
    shortname,
    hwmodel,
    deployment_location
)
VALUES
  (
    $1,
    $2,
    $3,
    $4,
    $5
  )
            ",
            self.node_id,
            self.longname,
            self.shortname,
            self.hwmodel,
            self.deployment_location
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to insert row into NodeInfo table")
    }

    /// Update a row in the `NodeInfo` table
    ///
    /// # Arguments
    /// * `self` - A `Nodeinfo` struct
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
UPDATE NodeInfo
SET
  longname = $2,
  shortname = $3,
  hwmodel = $4,
  deployment_location = $5
WHERE
  node_id = $1
            ",
            self.node_id,
            self.longname,
            self.shortname,
            self.hwmodel,
            self.deployment_location
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to update row in NodeInfo table")
    }
}

impl DbOps for &Nodeinfo {
    /// Insert a row into the `NodeInfo` table
    ///
    /// # Arguments
    /// * `self` - A `Nodeinfo` struct
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
  NodeInfo (
    node_id,
    longname,
    shortname,
    hwmodel,
    deployment_location
)
VALUES
  (
    $1,
    $2,
    $3,
    $4,
    $5
  )
            ",
            self.node_id,
            self.longname,
            self.shortname,
            self.hwmodel,
            self.deployment_location
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to insert row into NodeInfo table")
    }

    /// Update a row in the `NodeInfo` table
    ///
    /// # Arguments
    /// * `self` - A `Nodeinfo` struct
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
UPDATE NodeInfo
SET
  longname = $2,
  shortname = $3,
  hwmodel = $4,
  deployment_location = $5
WHERE
  node_id = $1
            ",
            self.node_id,
            self.longname,
            self.shortname,
            self.hwmodel,
            self.deployment_location
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to update row in NodeInfo table")
    }
}
