//! Config handling and serial port prompting

use anyhow::{Context as _, Result, anyhow};
use config::Config;
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use embedded_nano_mesh::{ExactAddressType, Node, NodeConfig};
use embedded_nano_mesh_linux_io::{
    LinuxIO,
    serialport::{self, available_ports},
};
use microxdg::XdgApp;
use serde::Deserialize;
use std::{
    fs,
    io::{BufRead as _, stdin},
    sync::OnceLock,
};

/// Deployment location constant to initialize with config value
pub(crate) static DEPLOYMENT_LOCATION: OnceLock<String> = OnceLock::new();

/// Example config file to write in case one cannot be found
static EXAMPLE_CONFIG: &[u8] = include_bytes!("example_config.toml");

/// XDG application handle for finding config paths.
static APP: OnceLock<XdgApp> = OnceLock::new();

/// Type alias for `PostgreSQL` connection pool
pub(crate) type PgPool = Pool<ConnectionManager<PgConnection>>;

/// Struct representing a Postgres connection's settings
#[derive(Debug, Deserialize)]
struct PostgresConnection {
    /// Username for Postgres db
    user: String,
    /// Password for Postgres db
    password: String,
    /// Port for Postgres db
    port: u16,
    /// Hostname of Postgres db
    host: String,
    /// Database name for Postgres db
    dbname: String,
    /// Maximum connection workers for db connection and half of incoming packets bound (max 32)
    max_connections: u32,
    /// Minimum connection workers for db connection
    min_connections: u32,
}

impl PostgresConnection {
    /// Creates a `PostgreSQL` connection pool from these settings
    fn setup(&self) -> Result<PgPool> {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.dbname
        );
        let manager = ConnectionManager::<PgConnection>::new(url);
        Pool::builder()
            .max_size(self.max_connections)
            .min_idle(Some(self.min_connections))
            .build(manager)
            .map_err(anyhow::Error::from)
    }
}

/// Struct representing a connection to a serial port's settings
#[derive(Debug, Deserialize)]
struct SerialConnection {
    /// The path to the serial port of a connected Meshtastic node, if left
    /// blank the user is prompted for the path out of a list of possible paths
    port: String,
    /// Baud rate for the serial port
    baud: u32,
    /// Device address connected to serial
    device_addr: u8,
    /// Listening period
    listen_period: u32,
}

/// Struct representing configured deployment information, like location
#[derive(Debug, Deserialize)]
pub(crate) struct DeploymentSettings {
    /// The name of this group of nodes
    pub location: String,
}

/// Settings struct that parses a config and sets up
#[derive(Debug, Deserialize)]
pub(crate) struct Settings {
    /// The Postgres connection config
    postgres: PostgresConnection,
    /// The serial connection to a Meshtastic node config
    serial: SerialConnection,
    /// The deployment config
    pub(crate) deployment: DeploymentSettings,
}

impl Settings {
    /// Reads the config file and returns a parsed `Settings` instance
    pub(crate) fn new() -> Result<Self> {
        // Create the XDG app while also setting a global static APP
        APP.set(
            XdgApp::new("meshtastic_telemetry")
                .context("Unable to initialize meshtastic_telemetry XDG Application")?,
        )
        .map_err(|e| anyhow!("Error setting XdgApp: {e:?}"))?;

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
        match Config::builder()
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
    fn get_serial_port(&self) -> Result<String> {
        if self.serial.port.is_empty() {
            tracing::warn!("No serial port provided by the configuration");
            tracing::warn!("Prompting user for serial port instead");
            match available_ports().context("Failed to enumerate list of serial ports") {
                Ok(ap) => {
                    // Filter for only `UsbPort` types
                    let usb_ports: Vec<String> = ap.into_iter().map(|p| p.port_name).collect();
                    println!("Available ports: {usb_ports:?}");
                }
                Err(e) => {
                    tracing::error!(%e);
                    tracing::warn!("User will input their own serial port");
                }
            }
            println!("Enter the name of a port to connect to:");
            let stdin = stdin();
            match stdin
                .lock()
                .lines()
                .next()
                .context("Could not read from stdin")?
            {
                Ok(sp) => Ok(sp),
                Err(e) => {
                    tracing::error!("No serial port provided by user");
                    Err(anyhow!(e))
                }
            }
        } else {
            Ok(self.serial.port.clone())
        }
    }

    /// Sets up a serial port connection to a node
    pub(crate) fn setup_serial(&self) -> Result<(Node, LinuxIO)> {
        let serial =
            LinuxIO::new(serialport::new(self.get_serial_port()?, self.serial.baud).open_native()?);
        let device_addr = ExactAddressType::new(self.serial.device_addr).ok_or(anyhow!(
            "Failed to create ExactAddressType for Serial Mesh interface"
        ))?;
        let node = Node::new(NodeConfig {
            device_address: device_addr,
            listen_period: self.serial.listen_period,
        });

        Ok((node, serial))
    }

    /// Sets up a Postgres connection
    pub(crate) fn setup_postgres(&self) -> Result<PgPool> {
        self.postgres.setup()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{File, FileFormat};

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
            baud = 9600
            device_addr = 1
            listen_period = 150

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
            baud = 9600
            device_addr = 1
            listen_period = 150

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

    #[test]
    fn test_deserialize_settings_missing_postgres_fails() -> Result<()> {
        // TOML is completely missing the [postgres] block
        let toml_content = r#"
            [serial]
            port = "/dev/ttyUSB0"

            [deployment]
            location = "Portland Gateway"
        "#;

        let config_res = Config::builder()
            .add_source(File::from_str(toml_content, FileFormat::Toml))
            .build()?;

        // Attempting to deserialize should fail
        let settings: Result<Settings, _> = config_res.try_deserialize();
        assert!(
            settings.is_err(),
            "Should fail when missing postgres config"
        );
        Ok(())
    }

    #[test]
    fn test_deserialize_settings_invalid_port_type_fails() -> Result<()> {
        // TOML has a string where an integer port is expected
        let toml_content = r#"
            [postgres]
            user = "test_user"
            password = "test_password"
            port = "NOT_A_NUMBER"
            host = "127.0.0.1"
            dbname = "test_db"
            max_connections = 20
            min_connections = 2

            [serial]
            port = "/dev/ttyUSB0"

            [deployment]
            location = "Portland Gateway"
        "#;

        let config_res = Config::builder()
            .add_source(File::from_str(toml_content, FileFormat::Toml))
            .build()?;

        let settings: Result<Settings, _> = config_res.try_deserialize();
        assert!(settings.is_err(), "Should fail when port is not an integer");
        Ok(())
    }
}
