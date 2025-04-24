#[cfg(feature = "sqlite")]
use crate::lite::setup_schema;
use anyhow::{Context, Result};
use log::LevelFilter;
#[cfg(feature = "debug")]
use log::{error, warn};
use meshtastic::utils::stream::available_serial_ports;
#[cfg(feature = "sqlite")]
use sea_orm::sqlx::{sqlite, ConnectOptions as SqliteConnectionOptions};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::Deserialize;
use std::io::{self, BufRead};
use std::str::FromStr;

/// Struct reprenting a postgres connection's settings
#[cfg(feature = "postgres")]
#[derive(Debug, Deserialize)]
#[allow(unused)]
struct PostgresConnection {
    /// Username for postgres db
    user: String,
    /// Password for postgres db
    password: String,
    /// Port for postgres db
    port: u32,
    /// Hostname of postgres db
    host: String,
    /// Database name for postgres db
    dbname: String,
    /// Maximum connection workers for db connection
    max_connections: u32,
    /// Minimum connection workers for db connection
    min_connections: u32,
}

#[cfg(feature = "postgres")]
impl PostgresConnection {
    /// Build a postgresql connection string
    ///
    /// Formats entries from the config file into:
    /// `postgres://user:password@host:port/database_name`
    ///
    /// # Arguments
    /// * `cfg` - The Config struct reference
    ///
    /// # Returns
    /// * `String` - A postgresql connection string
    fn build_db_connection_string(&self) -> String {
        // Parse config with error handling
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.dbname
        )
    }
    /// Setup a Postgresql connection pool
    ///
    /// # Returns
    /// * `Result<DatabaseConnection>` - An `anyhow` result with a connection pool to the postgresql
    ///   database
    async fn setup(&self) -> Result<DatabaseConnection> {
        // Connect to postgres db
        let mut opt = ConnectOptions::new(self.build_db_connection_string());

        // Set max connections to 2 and minimum to 1
        opt.max_connections(self.max_connections)
            .min_connections(self.min_connections);

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
}

/// Struct reprenting a sqlite connection's settings
#[cfg(feature = "sqlite")]
#[derive(Debug, Deserialize)]
#[allow(unused)]
struct SqliteConnection {
    /// Maximum connection workers for db connection
    max_connections: u32,
    /// Minimum connection workers for db connection
    min_connections: u32,
}

#[cfg(feature = "sqlite")]
impl SqliteConnection {
    /// Setup `SQLite3` database
    ///
    /// # Returns
    /// * `DatabaseConnection` - Connection to the sqlite3 db
    pub async fn setup(&self) -> Result<DatabaseConnection> {
        // Create connections options
        let conn_opts =
            sqlite::SqliteConnectOptions::from_str("sqlite:///tmp/mesh-tele.db?mode=rwc")
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
                    // Create the file if it is missing
                    .create_if_missing(true);
                // Logging settings
                #[cfg(debug_assertions)]
                let c = co.log_statements(LevelFilter::Trace);
                #[cfg(not(debug_assertions))]
                let c = co.log_statements(LevelFilter::Off);
                // Set connection timeout?
                let pool_opts = sqlite::SqlitePoolOptions::new()
                    .min_connections(self.min_connections)
                    .max_connections(self.max_connections);
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
}

/// Struct reprenting a connection to a serial port's settings
#[derive(Debug, Deserialize)]
#[allow(unused)]
struct SerialConnection {
    /// The path to the serial port of a connected Meshtastic node, if left
    /// blank the user is prompted for the path out of a list of possible paths
    port: String,
}

/// Struct representing configured deployment information, like location
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct DeploymentSettings {
    /// The name of this group of nodes
    pub location: String,
}

/// Struct representing configured async runtime settings
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AsyncSettings {
    /// The buffer size impacts how many packets can remain in the queue for
    /// processing and submitting to the database
    pub mpsc_buffer_size: u8,
}

/// Settings struct that parses a config and performs setup
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    /// The postgres connection config
    #[cfg(feature = "postgres")]
    postgres: PostgresConnection,
    /// The sqlite connection config
    #[cfg(feature = "sqlite")]
    sqlite: SqliteConnection,
    /// The serial connection to a Meshtastic node config
    serial: SerialConnection,
    /// The deployment config
    pub deployment: DeploymentSettings,
    /// The asynchronous runtime config
    pub async_runtime: AsyncSettings,
}

impl Settings {
    /// Read config file and create settings structure
    ///
    /// # Arguments
    /// * `p` - Path to the config file
    ///
    /// # Returns
    /// * `Settings` - `Settings` struct with keys and values
    ///
    /// # Panics
    /// Will panic if the configuration file cannot be read
    pub(crate) fn new(p: &str) -> Self {
        match config::Config::builder()
            .add_source(config::File::with_name(p))
            .build()
            .with_context(|| format!("Failed to read config file from {p}"))
        {
            Ok(rv) => rv.try_deserialize().expect("Error deserializing config"),
            Err(e) => {
                panic!("{e}");
            }
        }
    }

    /// Get the serial port to listen on
    ///
    /// Either the serial port will be found in the config file, or the serial port will be specified
    /// at the command line by the user.
    ///
    /// # Returns
    /// * `String` - The path of the serial port as a string
    ///
    /// # Panics
    /// This panics if a serial port is not provided by the user in the case that the config file does
    /// not provide a serial port path
    pub(crate) fn get_serial_port(&self) -> String {
        if self.serial.port == "" {
            warn!("Prompting user for serial port instead");
            match available_serial_ports()
                .with_context(|| "Failed to enumerate list of serial ports")
            {
                Ok(ap) => println!("Available ports: {ap:?}"),
                Err(e) => {
                    error!("{e}");
                    warn!("User will input their own serial port");
                }
            }
            println!("Enter the name of a port to connect to:");

            let stdin = io::stdin();
            match stdin
                .lock()
                .lines()
                .next()
                .expect("Failed to find next line")
                .with_context(|| "Could not read from stdin")
            {
                Ok(sp) => sp,
                Err(e) => {
                    eprintln!("No serial port provided by user");
                    panic!("{e}");
                }
            }
        } else {
            self.serial.port.clone()
        }
    }

    /// Setup postgres connection
    ///
    /// # Returns
    /// * `Result<DatabaseConnection>` - An `anyhow` result with a connection pool to the postgresql
    ///   database
    #[cfg(feature = "postgres")]
    pub(crate) async fn setup_postgres(&self) -> Result<DatabaseConnection> {
        self.postgres.setup().await
    }

    /// Setup sqlite connection
    ///
    /// # Returns
    /// * `Result<DatabaseConnection>` - An `anyhow` result with a connection pool to the sqlite
    ///   database
    #[cfg(feature = "sqlite")]
    pub(crate) async fn setup_sqlite(&self) -> Result<DatabaseConnection> {
        self.sqlite.setup().await
    }
}
