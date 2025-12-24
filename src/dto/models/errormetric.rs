#[derive(derive_new::new, sqlx::FromRow, sqlxinsert::PgInsert)]
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
