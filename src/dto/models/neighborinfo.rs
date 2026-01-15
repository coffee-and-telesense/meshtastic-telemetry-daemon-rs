#[derive(derive_new::new, sqlx::FromRow)]
pub struct Neighborinfo {
    msg_id: sqlx::postgres::types::Oid,
    node_id: sqlx::postgres::types::Oid,
    time: chrono::NaiveDateTime,
    last_sent_by_id: Option<sqlx::postgres::types::Oid>,
    node_broadcast_interval_secs: Option<sqlx::postgres::types::Oid>,
    neighbors: Option<Vec<serde_json::Value>>,
}
