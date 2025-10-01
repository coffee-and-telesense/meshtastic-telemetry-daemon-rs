use anyhow::{Context, Result};
#[cfg(feature = "debug")]
use log::{error, warn};
use meshtastic::utils::stream::available_serial_ports;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::Deserialize;
use std::io::{self, BufRead};

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

        opt.max_connections(self.max_connections)
            .min_connections(self.min_connections);

        #[cfg(feature = "debug")]
        {
            opt.sqlx_logging(true);
            opt.sqlx_logging_level(log::LevelFilter::Trace);
        }
        #[cfg(feature = "syslog")]
        {
            opt.sqlx_logging(false);
            opt.sqlx_logging_level(log::LevelFilter::Warn);
        }
        Database::connect(opt)
            .await
            .with_context(|| "Failed to connect to the database")
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
        if self.serial.port.is_empty() {
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
}
