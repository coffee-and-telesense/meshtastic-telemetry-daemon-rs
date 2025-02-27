use crate::build_db_connection_string;
use anyhow::{Context, Result};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

/// Setup a Postgresql connection pool
///
/// # Arguments
/// * `cfg` - A `config::Config` instance which has the settings toml
///
/// # Returns
/// * `Result<DatabaseConnection>` - An `anyhow` result with a connection pool to the postgresql
///   database
pub async fn setup(cfg: &config::Config) -> Result<DatabaseConnection> {
    // Connect to postgres db
    let mut opt = ConnectOptions::new(build_db_connection_string(cfg));
    #[cfg(debug_assertions)]
    {
        opt.sqlx_logging(true);
        opt.sqlx_logging_level(log::LevelFilter::Debug);
    }
    #[cfg(not(debug_assertions))]
    {
        opt.sqlx_logging(false);
        opt.sqlx_logging_level(log::LevelFilter::Off);
    }
    Database::connect(opt)
        .await
        .with_context(|| "Failed to connect to the database")
}
