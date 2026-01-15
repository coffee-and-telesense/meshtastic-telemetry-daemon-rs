#[allow(clippy::too_many_arguments)]
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
