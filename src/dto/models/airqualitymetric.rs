use crate::dto::types::DbOps;
use anyhow::Context;

#[allow(clippy::too_many_arguments)]
#[derive(derive_new::new, sqlx::FromRow)]
pub struct Airqualitymetric {
    msg_id: sqlx::postgres::types::Oid,
    node_id: sqlx::postgres::types::Oid,
    time: chrono::NaiveDateTime,
    pm_10_standard: Option<sqlx::postgres::types::Oid>,
    pm_25_standard: Option<sqlx::postgres::types::Oid>,
    pm_100_standard: Option<sqlx::postgres::types::Oid>,
    pm_10_environmental: Option<sqlx::postgres::types::Oid>,
    pm_25_environmental: Option<sqlx::postgres::types::Oid>,
    pm_100_environmental: Option<sqlx::postgres::types::Oid>,
    particles_03_um: Option<sqlx::postgres::types::Oid>,
    particles_05_um: Option<sqlx::postgres::types::Oid>,
    particles_10_um: Option<sqlx::postgres::types::Oid>,
    particles_25_um: Option<sqlx::postgres::types::Oid>,
    particles_50_um: Option<sqlx::postgres::types::Oid>,
    particles_100_um: Option<sqlx::postgres::types::Oid>,
    co_2: Option<sqlx::postgres::types::Oid>,
    sensor_type: Option<i32>,
}

impl DbOps for Airqualitymetric {
    /// Insert a row into the `AirQualityMetrics` table
    ///
    /// # Arguments
    /// * `self` - An `Airqualitymetric` struct
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
  AirQualityMetrics (
    msg_id,
    node_id,
    time,
    pm10standard,
    pm25standard,
    pm100standard,
    pm10environmental,
    pm25environmental,
    pm100environmental,
    particles03um,
    particles05um,
    particles10um,
    particles25um,
    particles50um,
    particles100um,
    co2,
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
    $15,
    $16,
    $17
  )
            ",
            self.msg_id,
            self.node_id,
            self.time,
            self.pm_10_standard,
            self.pm_25_standard,
            self.pm_100_standard,
            self.pm_10_environmental,
            self.pm_25_environmental,
            self.pm_100_environmental,
            self.particles_03_um,
            self.particles_05_um,
            self.particles_10_um,
            self.particles_25_um,
            self.particles_50_um,
            self.particles_100_um,
            self.co_2,
            self.sensor_type
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to insert row into AirQualityMetrics table")
    }

    /// Update a row in the `AirQualityMetrics` table
    ///
    /// # Arguments
    /// * `self` - An `Airqualitymetric` struct
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
UPDATE AirQualityMetrics
SET
  node_id = $2,
  time = $3,
  pm10standard = $4,
  pm25standard = $5,
  pm100standard = $6,
  pm10environmental = $7,
  pm25environmental = $8,
  pm100environmental = $9,
  particles03um = $10,
  particles05um = $11,
  particles10um = $12,
  particles25um = $13,
  particles50um = $14,
  particles100um = $15,
  co2 = $16,
  sensor_type = $17
WHERE
  msg_id = $1
            ",
            self.msg_id,
            self.node_id,
            self.time,
            self.pm_10_standard,
            self.pm_25_standard,
            self.pm_100_standard,
            self.pm_10_environmental,
            self.pm_25_environmental,
            self.pm_100_environmental,
            self.particles_03_um,
            self.particles_05_um,
            self.particles_10_um,
            self.particles_25_um,
            self.particles_50_um,
            self.particles_100_um,
            self.co_2,
            self.sensor_type
        )
        .execute(pool)
        .await
        .map_err(anyhow::Error::from)
        .with_context(|| "Failed to update row in AirQualityMetrics table")
    }
}
