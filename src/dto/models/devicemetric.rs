use crate::dto::types::DbOps;
use anyhow::Context;

#[derive(derive_new::new, sqlx::FromRow)]
pub struct Devicemetric {
    msg_id: sqlx::postgres::types::Oid,
    node_id: sqlx::postgres::types::Oid,
    time: chrono::NaiveDateTime,
    battery_levels: Option<sqlx::postgres::types::Oid>,
    voltage: Option<f32>,
    channelutil: Option<f32>,
    airutil: Option<f32>,
    latitude: Option<i32>,
    longitude: Option<i32>,
    longname: Option<String>,
    shortname: Option<String>,
    hwmodel: Option<i32>,
}

impl DbOps for Devicemetric {
    /// Insert a row into the `DeviceMetrics` table
    ///
    /// # Arguments
    /// * `self` - A `Devicemetric` struct
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
  DeviceMetrics (
    msg_id,
    node_id,
    time,
    battery_levels,
    voltage,
    channelutil,
    airutil,
    latitude,
    longitude,
    longname,
    shortname,
    hwmodel
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
    $12
  )
            ",
            self.msg_id,
            self.node_id,
            self.time,
            self.battery_levels,
            self.voltage,
            self.channelutil,
            self.airutil,
            self.latitude,
            self.longitude,
            self.longname,
            self.shortname,
            self.hwmodel
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to insert row into DeviceMetrics table")
    }

    /// Update a row in the `DeviceMetrics` table
    ///
    /// # Arguments
    /// * `self` - A `Devicemetric` struct
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
UPDATE DeviceMetrics
SET
  node_id = $2,
  time = $3,
  battery_levels = $4,
  voltage = $5,
  channelutil = $6,
  airutil = $7,
  latitude = $8,
  longitude = $9,
  longname = $10,
  shortname = $11,
  hwmodel = $12
WHERE
  msg_id = $1
            ",
            self.msg_id,
            self.node_id,
            self.time,
            self.battery_levels,
            self.voltage,
            self.channelutil,
            self.airutil,
            self.latitude,
            self.longitude,
            self.longname,
            self.shortname,
            self.hwmodel
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to update row in DeviceMetrics table")
    }
}
