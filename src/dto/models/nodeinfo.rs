#[derive(derive_new::new, sqlx::FromRow)]
pub struct Nodeinfo {
    node_id: sqlx::postgres::types::Oid,
    longname: String,
    shortname: String,
    hwmodel: i32,
    deployment_location: String,
}
