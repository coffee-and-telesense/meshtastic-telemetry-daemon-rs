use crate::util::config::DEPLOYMENT_LOCATION;
use anyhow::{Context, Error};
use meshtastic::protobufs::NodeInfo;
use sqlx::postgres::types::Oid;

/// Insert a row into the `NodeInfo` table
///
/// # Arguments
/// * `ni` - A `NodeInfo` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn insert(
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
INSERT INTO
  NodeInfo (
    node_id,
    longname,
    shortname,
    hwmodel,
    deployment_location
)
VALUES
  (
    $1,
    $2,
    $3,
    $4,
    $5
  )
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
    .with_context(|| "Failed to insert row into NodeInfo table")
}

/// Update a row in the `NodeInfo` table
///
/// # Arguments
/// * `ni` - A `NodeInfo` reference
/// * `pool` - A `Pool<Postgres>` reference
///
/// # Returns
/// * `anyhow::Result<PgQueryResult, anyhow::Error>` - Anyhow result and error with debug info
pub(crate) async fn update(
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
UPDATE NodeInfo
SET
  longname = $2,
  shortname = $3,
  hwmodel = $4,
  deployment_location = $5
WHERE
  node_id = $1
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
    .with_context(|| "Failed to update row in NodeInfo table")
}
