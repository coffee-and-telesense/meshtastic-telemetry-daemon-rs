#[allow(clippy::too_many_arguments)]
#[derive(derive_new::new, sqlx::FromRow)]
pub struct Localstat {
    msg_id: sqlx::postgres::types::Oid,
    node_id: sqlx::postgres::types::Oid,
    time: chrono::NaiveDateTime,
    uptime_seconds: Option<sqlx::postgres::types::Oid>,
    channel_util: Option<f32>,
    air_util_tx: Option<f32>,
    num_packets_tx: Option<sqlx::postgres::types::Oid>,
    num_packets_rx: Option<sqlx::postgres::types::Oid>,
    num_packets_rx_bad: Option<sqlx::postgres::types::Oid>,
    num_online_nodes: Option<sqlx::postgres::types::Oid>,
    num_total_nodes: Option<sqlx::postgres::types::Oid>,
    num_rx_dupe: Option<sqlx::postgres::types::Oid>,
    num_tx_relay: Option<sqlx::postgres::types::Oid>,
    num_tx_relay_canceled: Option<sqlx::postgres::types::Oid>,
}
