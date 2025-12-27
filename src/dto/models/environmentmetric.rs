use crate::dto::types::DbOps;
use anyhow::Context;

#[derive(derive_new::new, sqlx::FromRow)]
pub struct Environmentmetric {
    msg_id: sqlx::postgres::types::Oid,
    node_id: sqlx::postgres::types::Oid,
    time: chrono::NaiveDateTime,
    tempurature: Option<f32>,
    relative_humidity: Option<f32>,
    barometric_pressure: Option<f32>,
    gas_resistance: Option<f32>,
    iaq: Option<sqlx::postgres::types::Oid>,
    wind_direction: Option<sqlx::postgres::types::Oid>,
    wind_speed: Option<f32>,
    wind_gust: Option<f32>,
    wind_lull: Option<f32>,
    rainfall_1_h: Option<f32>,
    rainfall_24_h: Option<f32>,
    sensor_type: Option<i32>,
}

impl DbOps for Environmentmetric {
    /// Insert a row into the `EnvironmentMetrics` table
    ///
    /// # Arguments
    /// * `self` - An `Environmentmetric` struct
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
  EnvironmentMetrics (
    msg_id,
    node_id,
    TIME,
    tempurature,
    relative_humidity,
    barometric_pressure,
    gas_resistance,
    iaq,
    wind_direction,
    wind_speed,
    wind_gust,
    wind_lull,
    rainfall_1h,
    rainfall_24h,
    sensor_type
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
    $14,
    $15
  )
            ",
            self.msg_id,
            self.node_id,
            self.time,
            self.tempurature,
            self.relative_humidity,
            self.barometric_pressure,
            self.gas_resistance,
            self.iaq,
            self.wind_direction,
            self.wind_speed,
            self.wind_gust,
            self.wind_lull,
            self.rainfall_1_h,
            self.rainfall_24_h,
            self.sensor_type
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to insert row into EnvironmentMetrics table")
    }

    /// Update a row in the `EnvironmentMetrics` table
    ///
    /// # Arguments
    /// * `self` - An `Environmentmetric` struct
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
UPDATE EnvironmentMetrics
SET
  node_id = $2,
  time = $3,
  tempurature = $4,
  relative_humidity = $5,
  barometric_pressure = $6,
  gas_resistance = $7,
  iaq = $8,
  wind_direction = $9,
  wind_speed = $10,
  wind_gust = $11,
  wind_lull = $12,
  rainfall_1h = $13,
  rainfall_24h = $14,
  sensor_type = $15
WHERE
  msg_id = $1
            ",
            self.msg_id,
            self.node_id,
            self.time,
            self.tempurature,
            self.relative_humidity,
            self.barometric_pressure,
            self.gas_resistance,
            self.iaq,
            self.wind_direction,
            self.wind_speed,
            self.wind_gust,
            self.wind_lull,
            self.rainfall_1_h,
            self.rainfall_24_h,
            self.sensor_type
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to update row in EnvironmentMetrics table")
    }
}
