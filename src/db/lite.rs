use anyhow::{Context, Result};
#[cfg(feature = "debug")]
use log::error;
use sea_orm::{
    sea_query::TableCreateStatement, sqlx, ConnectionTrait, DatabaseConnection, DbBackend, Schema,
};
use std::str::FromStr;

use crate::dto::entities::{airqualitymetrics, devicemetrics, environmentmetrics, nodeinfo};

/// Setup `SQLite3` database
///
/// # Returns
/// * `DatabaseConnection` - Connection to the sqlite3 db
pub async fn setup() -> Result<DatabaseConnection> {
    // Create connections options
    let conn_opts =
        sqlx::sqlite::SqliteConnectOptions::from_str("sqlite:///tmp/mesh-tele.db?mode=rw")
            .with_context(|| "Error connecting to sqlite db at /tmp/mesh-tele.db");
    match conn_opts {
        Ok(mut co) => {
            co = co
                // Turn off journaling
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Off)
                // Turn on auto vacuuming
                .auto_vacuum(sqlx::sqlite::SqliteAutoVacuum::Full)
                // Create the file if it is missing
                .create_if_missing(true);
            // Set connection timeout?
            let pool_opts = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1);
            //    .idle_timeout(None)
            //    .max_lifetime(None);
            let pool = pool_opts.connect_with(co).await?;
            let db = sea_orm::SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
            setup_schema(&db).await;
            Ok(db)
        }
        Err(e) => {
            error!("{e:#}");
            panic!("Could not connect to sqlite db");
        }
    }
}

async fn setup_schema(db: &DatabaseConnection) {
    let schema = Schema::new(DbBackend::Sqlite);
    let em_stmt: TableCreateStatement = schema.create_table_from_entity(environmentmetrics::Entity);
    let dm_stmt: TableCreateStatement = schema.create_table_from_entity(devicemetrics::Entity);
    let ni_stmt: TableCreateStatement = schema.create_table_from_entity(nodeinfo::Entity);
    let aqm_stmt: TableCreateStatement = schema.create_table_from_entity(airqualitymetrics::Entity);
    match db
        .execute(db.get_database_backend().build(&ni_stmt))
        .await
        .with_context(|| "node info table creation failed")
    {
        Ok(_) => {}
        Err(e) => {
            error!("{e:#}");
            panic!("Table creation failed for node info")
        }
    }
    match db
        .execute(db.get_database_backend().build(&aqm_stmt))
        .await
        .with_context(|| "air quality metrics table creation failed")
    {
        Ok(_) => {}
        Err(e) => {
            error!("{e:#}");
            panic!("Table creation failed for node info")
        }
    }
    match db
        .execute(db.get_database_backend().build(&dm_stmt))
        .await
        .with_context(|| "device metrics table creation failed")
    {
        Ok(_) => {}
        Err(e) => {
            error!("{e:#}");
            panic!("Table creation failed for node info")
        }
    }
    match db
        .execute(db.get_database_backend().build(&em_stmt))
        .await
        .with_context(|| "environment metrics table creation failed")
    {
        Ok(_) => {}
        Err(e) => {
            error!("{e:#}");
            panic!("Table creation failed for node info")
        }
    }
}
