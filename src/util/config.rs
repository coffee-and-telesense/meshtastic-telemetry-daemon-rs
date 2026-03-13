use anyhow::{Context, Result, anyhow};
use meshtastic::utils::stream::available_serial_ports;
use microxdg::XdgApp;
use serde::Deserialize;
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
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

/// XDG application handle for finding config paths.
static APP: OnceCell<XdgApp> = OnceCell::const_new();

/// Struct representing a Postgres connection's settings
#[derive(Debug, Deserialize)]
struct PostgresConnection<'a> {
    /// Username for Postgres db
    user: Cow<'a, str>,
    /// Password for Postgres db
    password: Cow<'a, str>,
    /// Port for Postgres db
    port: u16,
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
    /// Creates a `PostgreSQL` connection pool from these settings
    async fn setup(&self) -> Result<PgPool> {
        let conn = PgConnectOptions::new()
            .username(&self.user)
            .password(&self.password)
            .host(&self.host)
            .port(self.port)
            .database(&self.dbname);

        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect_with(conn)
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
pub(crate) struct DeploymentSettings<'a> {
    /// The name of this group of nodes
    pub location: Cow<'a, str>,
}

/// Settings struct that parses a config and sets up
#[derive(Debug, Deserialize)]
pub(crate) struct Settings<'a> {
    /// The Postgres connection config
    postgres: PostgresConnection<'a>,
    /// The serial connection to a Meshtastic node config
    serial: SerialConnection<'a>,
    /// The deployment config
    pub(crate) deployment: DeploymentSettings<'a>,
}

impl<'a> Settings<'a> {
    /// Reads the config file and returns a parsed `Settings` instance
    pub(crate) fn new() -> Result<Self> {
        // Create the XDG app while also setting a global static APP
        APP.set(
            XdgApp::new("meshtastic_telemetry")
                .context("Unable to initialize meshtastic_telemetry XDG Application")?,
        )?;

        // Check the config directory, if it does not exist then create it
        let config_dir = APP
            .get()
            .context("XDG app initialized twice")?
            .app_config()
            .context("Unable to find meshtastic_telemetry XDG configuration directory")?;
        if !config_dir.try_exists()? {
            fs::create_dir(config_dir.as_path())?;
        }

        // Check the config directory for a `config.toml` file, if it does not exist then create it
        let config_file = APP
            .get()
            .context("XDG app initialized twice")?
            .app_config_file("config.toml")
            .with_context(|| {
                format!(
                    "Failed to find meshtastic_telemetry config.toml in {}",
                    config_dir.display()
                )
            })?;
        if !config_file.try_exists()? {
            fs::write(config_file.as_path(), EXAMPLE_CONFIG)?;
        }

        // Read the configuration
        match config::Config::builder()
            .add_source(config::File::from(config_file))
            .build()
            .context("Failed to read config file")?
            .try_deserialize()
            .context("Error deserializing config")
        {
            Ok(c) => Ok(c),
            Err(e) => Err(anyhow!(e)),
        }
    }

    /// Returns the configured serial port, prompting the user interactively if none is set.
    ///
    /// # Panics
    /// This panics if a serial port is not provided by the user in the case that the config file does
    /// not provide a serial port path
    pub(crate) fn get_serial_port(&'a self) -> Result<Cow<'a, str>> {
        if self.serial.port.is_empty() {
            tracing::warn!("Prompting user for serial port instead");
            match available_serial_ports().context("Failed to enumerate list of serial ports") {
                Ok(ap) => println!("Available ports: {ap:?}"),
                Err(e) => {
                    tracing::error!(%e);
                    tracing::warn!("User will input their own serial port");
                }
            }
            println!("Enter the name of a port to connect to:");

            let stdin = io::stdin();
            match stdin
                .lock()
                .lines()
                .next()
                .context("Could not read from stdin")?
            {
                Ok(sp) => Ok(Cow::Owned(sp)),
                Err(e) => {
                    tracing::error!("No serial port provided by user");
                    Err(anyhow!(e))
                }
            }
        } else {
            Ok(Cow::Borrowed(&self.serial.port))
        }
    }

    /// Sets up a Postgres connection
    pub(crate) async fn setup_postgres(&self) -> Result<PgPool> {
        self.postgres.setup().await
    }

    /// Get the maximum connections value to bound in-flight tasks for received packets
    pub(crate) fn get_max_connections(&self) -> usize {
        self.postgres.max_connections as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, File, FileFormat};

    #[test]
    fn test_deserialize_settings_valid_toml() -> Result<()> {
        let toml_content = r#"
            [postgres]
            user = "test_user"
            password = "test_password"
            port = 5432
            host = "127.0.0.1"
            dbname = "test_db"
            max_connections = 20
            min_connections = 2

            [serial]
            port = "/dev/ttyUSB0"

            [deployment]
            location = "Portland Gateway"
        "#;

        let config = Config::builder()
            .add_source(File::from_str(toml_content, FileFormat::Toml))
            .build()?; // Using `?` instead of `.expect()`

        let settings: Settings = config.try_deserialize()?; // Using `?` here too

        // Assert Postgres configurations
        assert_eq!(settings.postgres.user, "test_user");
        assert_eq!(settings.postgres.password, "test_password");
        assert_eq!(settings.postgres.port, 5432);
        assert_eq!(settings.postgres.host, "127.0.0.1");
        assert_eq!(settings.postgres.dbname, "test_db");
        assert_eq!(settings.postgres.max_connections, 20);
        assert_eq!(settings.postgres.min_connections, 2);

        // Assert Serial configurations
        assert_eq!(settings.serial.port, "/dev/ttyUSB0");
        assert_eq!(settings.get_serial_port()?, "/dev/ttyUSB0");

        // Assert Deployment configurations
        assert_eq!(settings.deployment.location, "Portland Gateway");
        assert_eq!(settings.get_max_connections(), 20);

        Ok(())
    }

    #[test]
    fn test_deserialize_settings_missing_serial_port() -> Result<()> {
        let toml_content = r#"
            [postgres]
            user = "test_user"
            password = "test_password"
            port = 5432
            host = "127.0.0.1"
            dbname = "test_db"
            max_connections = 10
            min_connections = 1

            [serial]
            port = ""

            [deployment]
            location = "Remote Node"
        "#;

        let config = Config::builder()
            .add_source(File::from_str(toml_content, FileFormat::Toml))
            .build()?;

        let settings: Settings = config.try_deserialize()?;
        assert!(settings.serial.port.is_empty());

        Ok(())
    }
}
