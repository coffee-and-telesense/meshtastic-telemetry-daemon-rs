use anyhow::{Context, Result};
use log::{error, warn};
use meshtastic::utils::stream::available_serial_ports;
use microxdg::XdgApp;
use serde::Deserialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{
    borrow::Cow,
    fs,
    io::{self, BufRead},
};
use tokio::sync::OnceCell;

/// Deployment location constant to initialize with config value
pub(crate) static DEPLOYMENT_LOCATION: OnceCell<&'static str> = OnceCell::const_new();

/// Example config file to write in case one cannot be found
static EXAMPLE_CONFIG: &[u8] = include_bytes!("example_config.toml");

/// XDG app specification for use (possibly caching or saving state)
static APP: OnceCell<XdgApp> = OnceCell::const_new();

/// Struct representing a Postgres connection's settings
#[derive(Debug, Deserialize)]
struct PostgresConnection<'a> {
    /// Username for Postgres db
    user: Cow<'a, str>,
    /// Password for Postgres db
    password: Cow<'a, str>,
    /// Port for Postgres db
    port: u32,
    /// Hostname of Postgres db
    host: Cow<'a, str>,
    /// Database name for Postgres db
    dbname: Cow<'a, str>,
    /// Maximum connection workers for db connection and half of incoming packets bound (max 32)
    max_connections: u32,
    /// Minimum connection workers for db connection
    min_connections: u32,
}

impl PostgresConnection<'_> {
    /// Setup a `PostgreSQL` connection pool
    ///
    /// # Returns
    /// * `Result<PgPool>` - An `anyhow` result with a connection pool to the `PostgreSQL`
    ///   database
    ///
    /// # Panics
    /// Will panic if the database connection string is longer than 128 characters long
    async fn setup(&self) -> Result<PgPool> {
        use std::fmt::Write;

        // Write the database connection string into a String with a given capacity
        let mut s = String::with_capacity(256);
        write!(
            s,
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.dbname
        )
        .expect("Unable to write postgres connection string from config variables");

        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .connect(s.as_str())
            .await
            .map_err(anyhow::Error::from)
    }
}

/// Struct representing a connection to a serial port's settings
#[derive(Debug, Deserialize)]
struct SerialConnection<'a> {
    /// The path to the serial port of a connected Meshtastic node, if left
    /// blank the user is prompted for the path out of a list of possible paths
    port: Cow<'a, str>,
}

/// Struct representing configured deployment information, like location
#[derive(Debug, Deserialize)]
pub struct DeploymentSettings<'a> {
    /// The name of this group of nodes
    pub location: Cow<'a, str>,
}

/// Settings struct that parses a config and sets up
#[derive(Debug, Deserialize)]
pub struct Settings<'a> {
    /// The Postgres connection config
    postgres: PostgresConnection<'a>,
    /// The serial connection to a Meshtastic node config
    serial: SerialConnection<'a>,
    /// The deployment config
    pub deployment: DeploymentSettings<'a>,
}

impl<'a> Settings<'a> {
    /// Read config file and create settings structure
    ///
    /// # Returns
    /// * `Settings` - `Settings` struct with keys and values
    ///
    /// # Panics
    /// Will panic if the configuration file or directory cannot be read or created
    pub(crate) fn new() -> Self {
        // Create the XDG app while also setting a global static APP
        match APP.set(
            match XdgApp::new("meshtastic_telemetry")
                .context("Unable to initialize meshtastic_telemetry XDG Application")
            {
                Ok(x) => x,
                Err(e) => panic!("{e}"),
            },
        ) {
            Ok(()) => (),
            Err(e) => panic!("{e}"),
        }

        // Check the config directory, if it does not exist then create it
        let config_dir = match APP
            .get()
            .expect("APP OnceCell not initialized before use")
            .app_config()
            .context("Unable to find meshtastic_telemetry XDG configuration directory")
        {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };
        match config_dir.try_exists() {
            Ok(b) => {
                if !b {
                    match fs::create_dir(config_dir.as_path()) {
                        Ok(()) => (),
                        Err(e) => panic!("{e}"),
                    }
                }
            }
            Err(e) => panic!("{e}"),
        }

        // Check the config directory for a `config.toml` file, if it does not exist then create it
        let config_file = match APP
            .get()
            .expect("APP OnceCell not initialized before use")
            .app_config_file("config.toml")
            .with_context(|| {
                format!(
                    "Failed to find meshtastic_telemetry config.toml in {}",
                    config_dir.display()
                )
            }) {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        };
        match config_file.try_exists() {
            Ok(b) => {
                if !b {
                    match fs::write(config_file.as_path(), EXAMPLE_CONFIG) {
                        Ok(()) => (),
                        Err(e) => panic!("{e}"),
                    }
                }
            }
            Err(e) => panic!("{e}"),
        }

        // Read the configuration
        match config::Config::builder()
            .add_source(config::File::from(config_file))
            .build()
            .context("Failed to read config file")
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
    pub(crate) fn get_serial_port(&'a self) -> Cow<'a, str> {
        if self.serial.port.is_empty() {
            warn!("Prompting user for serial port instead");
            match available_serial_ports().context("Failed to enumerate list of serial ports") {
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
                .context("Could not read from stdin")
            {
                Ok(sp) => Cow::Owned(sp),
                Err(e) => {
                    eprintln!("No serial port provided by user");
                    panic!("{e}");
                }
            }
        } else {
            Cow::Borrowed(&self.serial.port)
        }
    }

    /// Setup a Postgres connection
    ///
    /// # Returns
    /// * `Result<PgPool>` - An `anyhow` result with a connection pool to the `PostgreSQL`
    ///   database
    pub(crate) async fn setup_postgres(&self) -> Result<PgPool> {
        self.postgres.setup().await
    }

    /// Get the maximum connections value to bound in-flight tasks for received packets
    ///
    /// # Arguments
    /// * `&self` - A reference to the `Settings` struct
    ///
    /// # Returns
    /// * `usize` - The configured number of maximum connections
    pub(crate) fn get_max_connections(&self) -> usize {
        self.postgres.max_connections as usize
    }
}
