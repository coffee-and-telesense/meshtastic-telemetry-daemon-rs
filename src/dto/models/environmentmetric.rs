#[allow(clippy::too_many_arguments)]
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
