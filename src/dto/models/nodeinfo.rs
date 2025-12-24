/// static data.
#[derive(Debug, Clone, derive_new::new, sqlx::FromRow, sqlxinsert::PgInsert)]
pub struct Nodeinfo {
    node_id: sqlx::postgres::types::Oid,
    longname: String,
    shortname: String,
    hwmodel: i32,
    deployment_location: String,
}
