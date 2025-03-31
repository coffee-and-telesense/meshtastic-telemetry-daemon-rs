use crate::dto::entities::{airqualitymetrics, devicemetrics, environmentmetrics, nodeinfo};
use anyhow::{Context, Result};
#[cfg(feature = "debug")]
use log::error;
use log::LevelFilter;
use sea_orm::{
    sea_query::TableCreateStatement,
    sqlx::{sqlite, ConnectOptions},
    ConnectionTrait, DatabaseConnection, DbBackend, Schema, Statement,
};
use std::{str::FromStr, time::Instant};

/// Setup `SQLite3` database
///
/// # Returns
/// * `DatabaseConnection` - Connection to the sqlite3 db
pub async fn setup() -> Result<DatabaseConnection> {
    // Create connections options
    let conn_opts = sqlite::SqliteConnectOptions::from_str("sqlite:///tmp/mesh-tele.db?mode=rw")
        .with_context(|| "Error connecting to sqlite db at /tmp/mesh-tele.db");
    match conn_opts {
        Ok(mut co) => {
            co = co
                // Try write-ahead logging to allow concurrent reads during writes
                .journal_mode(sqlite::SqliteJournalMode::Wal)
                // Turn on the shared cache
                .shared_cache(true)
                // Try exclusive locking? Useful for when each db has a single thread
                //.locking_mode(sqlite::SqliteLockingMode::Exclusive)
                // 50% default of 100 statement cache
                .statement_cache_capacity(50)
                // Reduce page size to 50% of default 4096
                .page_size(2048)
                // Set synchronous to normal as WAL provides guarantees
                .synchronous(sqlite::SqliteSynchronous::Normal)
                // Store temporary files in memory?
                .pragma("temp_store", "memory")
                // Use memory mapped I/O, since we are 32 bit 2^32 is upper limit. Set to 2^16
                .pragma("mmap_size", "65536")
                // Turn on auto vacuuming
                .auto_vacuum(sqlite::SqliteAutoVacuum::Full)
                // Run a vacuum
                .pragma("vacuum", "")
                // Optimize the DB
                .pragma("optimize", "")
                // Create the file if it is missing
                .create_if_missing(true);
            // Logging settings
            #[cfg(debug_assertions)]
            let c = co.log_statements(LevelFilter::Debug);
            #[cfg(not(debug_assertions))]
            let c = co.log_statements(LevelFilter::Off);
            // Set connection timeout?
            let pool_opts = sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .min_connections(1);
            //    .idle_timeout(None)
            //    .max_lifetime(None);
            let pool = pool_opts.connect_lazy_with(c);
            let db = sea_orm::SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);
            setup_schema(&db).await;
            Ok(db)
        }
        Err(e) => {
            error!("{e}");
            panic!("Could not connect to sqlite db");
        }
    }
}

const TABLES: [&str; 3] = ["airqualitymetrics", "devicemetrics", "environmentmetrics"];

/// Drop old table rows periodically
pub fn drop_old_rows(db: &DatabaseConnection, last: Instant) -> Instant {
    if last.elapsed().as_secs() >= 7200 {
        for t in TABLES {
            db.execute(Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                format!("DELETE FROM {} WHERE time <= date('now','-30 day')", t),
            ));
        }
    }
    Instant::now()
}

/// Optimize the db regularly for memory usage and performance
/// https://www.sqlite.org/pragma.html#pragma_optimize
pub fn pragma_optimize(db: &DatabaseConnection, last: Instant) -> Instant {
    if last.elapsed().as_secs() >= 86400 {
        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "pragma optimize",
        ));
    }
    Instant::now()
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
