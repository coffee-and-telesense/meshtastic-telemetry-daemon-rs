use crate::dto::entities::{airqualitymetrics, devicemetrics, environmentmetrics, nodeinfo};
use anyhow::Context;
#[cfg(feature = "debug")]
use log::{error, info};
use sea_orm::{
    sea_query::TableCreateStatement, ConnectionTrait, DatabaseConnection, DbBackend, Schema,
    Statement,
};
use std::time::Instant;

const TABLES: [&str; 3] = ["airqualitymetrics", "devicemetrics", "environmentmetrics"];

/// Drop old table rows periodically
///
/// # Arguments
/// * `db` - A `DatabaseConnection` to the sqlite db
/// * `last` - An `Instant` representing the last time old table rows were dropped
///
/// # Returns
/// * An `Instant` representing the completion of this dropping of old table rows
pub async fn drop_old_rows(db: &DatabaseConnection, last: Instant) -> Instant {
    if last.elapsed().as_secs() >= 7200 {
        for t in TABLES {
            match db
                .execute(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    format!("DELETE FROM {} WHERE time <= date('now','-30 day')", t),
                ))
                .await
            {
                Ok(a) => info!(
                    "Successfully dropped {} old rows from sqlite",
                    a.rows_affected()
                ),
                Err(e) => error!("Error dropping old rows from sqlite: {e}"),
            }
        }
    }
    Instant::now()
}

/// Optimize the db regularly for memory usage and performance
/// https://www.sqlite.org/pragma.html#pragma_optimize
///
/// # Arguments
/// * `db` - A `DatabaseConnection` to the sqlite db
/// * `last` - An `Instant` representing the last time a pragma optimize was ran
///
/// # Returns
/// * An `Instant` representing the completion of this pragma optimize
pub async fn pragma_optimize(db: &DatabaseConnection, last: Instant) -> Instant {
    if last.elapsed().as_secs() >= 86400 {
        match db
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "pragma optimize",
            ))
            .await
        {
            Ok(a) => info!("Optimized sqlite: {} rows affected", a.rows_affected()),
            Err(e) => error!("Error optimizing sqlite: {e}"),
        }
    }
    Instant::now()
}

/// Setup tables for sqlite db
///
/// # Arguments
/// * `db` - A `DatabaseConnection` to the sqlite db
pub async fn setup_schema(db: &DatabaseConnection) {
    info!("Setting up sqlite database");
    let schema = Schema::new(DbBackend::Sqlite);
    let em_stmt: TableCreateStatement = schema.create_table_from_entity(environmentmetrics::Entity);
    let dm_stmt: TableCreateStatement = schema.create_table_from_entity(devicemetrics::Entity);
    let ni_stmt: TableCreateStatement = schema.create_table_from_entity(nodeinfo::Entity);
    let aqm_stmt: TableCreateStatement = schema.create_table_from_entity(airqualitymetrics::Entity);
    match db
        .execute(db.get_database_backend().build(&ni_stmt))
        .await
        .with_context(|| "node info table creation failed for sqlite")
    {
        Ok(_) => {}
        Err(e) => {
            error!("{e}, it likely already exists, skipping creation of other tables");
            return;
        }
    }
    match db
        .execute(db.get_database_backend().build(&aqm_stmt))
        .await
        .with_context(|| "air quality metrics table creation failed for sqlite")
    {
        Ok(_) => {}
        Err(e) => {
            error!("{e}");
            return;
        }
    }
    match db
        .execute(db.get_database_backend().build(&dm_stmt))
        .await
        .with_context(|| "device metrics table creation failed for sqlite")
    {
        Ok(_) => {}
        Err(e) => {
            error!("{e}");
            return;
        }
    }
    match db
        .execute(db.get_database_backend().build(&em_stmt))
        .await
        .with_context(|| "environment metrics table creation failed for sqlite")
    {
        Ok(_) => {}
        Err(e) => {
            error!("{e}");
        }
    }
}
