use crate::util::config::DEPLOYMENT_LOCATION;
use anyhow::{Context, Error};
use meshtastic::protobufs::NodeInfo;
use sqlx::postgres::types::Oid;

/// Upsert (insert or update) a row in the `NodeInfo` table
///
/// # Arguments
/// * `ni` - A `NodeInfo` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn upsert(
    ni: &NodeInfo,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<sqlx::postgres::PgQueryResult, anyhow::Error> {
    if ni.user.is_none() {
        return Result::Err(Error::msg(
            "NodeInfo packet does not contain User information",
        ));
    }

    let loc = DEPLOYMENT_LOCATION
        .get()
        .expect("Unable to get DEPLOYMENT_LOCATION in insert() for NodeInfo table");

    sqlx::query!(
        "
INSERT INTO NodeInfo (node_id, longname, shortname, hwmodel, deployment_location)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (node_id) DO UPDATE SET
    longname            = EXCLUDED.longname,
    shortname           = EXCLUDED.shortname,
    hwmodel             = EXCLUDED.hwmodel,
    deployment_location = EXCLUDED.deployment_location
        ",
        Oid(ni.num),
        ni.user.as_ref().map(|u| u.long_name.as_str()),
        ni.user.as_ref().map(|u| u.short_name.as_str()),
        ni.user.as_ref().map(|u| u.hw_model),
        loc,
    )
    .execute(pool)
    .await
    .map_err(anyhow::Error::from)
    .and_then(|result| {
        if result.rows_affected() == 0 {
            Err(anyhow::anyhow!("Upsert matched 0 rows in NodeInfo"))
        } else {
            Ok(result)
        }
    })
    .with_context(|| "Failed to upsert row in NodeInfo table")
}
