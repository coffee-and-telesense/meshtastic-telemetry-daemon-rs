use anyhow::Context;
use config::Config;
#[cfg(feature = "debug")]
use log::{error, warn};
use meshtastic::utils::stream::available_serial_ports;
use serde::Deserialize;
#[cfg(debug_assertions)]
use std::collections::HashMap;
use std::io::{self, BufRead};

/// Read config file
///
/// # Arguments
/// * `p` - Path to the config file
///
/// # Returns
/// * `Config` - Config struct with keys and values
///
/// # Panics
/// Will panic if the configuration file cannot be read
pub(crate) fn read_config(p: &str) -> config::Config {
    match config::Config::builder()
        .add_source(config::File::with_name(p))
        .build()
        .with_context(|| format!("Failed to read config file from {p}"))
    {
        Ok(rv) => {
            #[cfg(debug_assertions)]
            println!(
                "{:?}",
                rv.clone()
                    .try_deserialize::<HashMap<String, String>>()
                    .expect("Failed to deserialize config values in println of read_config()")
            );
            rv
        }
        Err(e) => {
            panic!("{e:#}");
        }
    }
}

/// Get config value
///
/// # Arguments
/// * `T` - The type to get from the config (not `String`, see `get_cfg_string()`)
/// * `cfg` - The Config struct reference
/// * `key` - The key reference to search for
///
/// # Returns
/// * `T` - The value for the `key` of type `T`
///
/// # Panics
/// Will panic if the key cannot be read from the Config struct
pub(crate) fn get_cfg<'d, T: Deserialize<'d>>(cfg: &Config, key: &str) -> T {
    match cfg
        .get::<T>(key)
        .with_context(|| format!("Failed to read {key} from Config struct"))
    {
        Ok(rv) => rv,
        Err(e) => {
            panic!("{e:#}");
        }
    }
}

/// Get config string value
///
/// # Arguments
/// * `cfg` - The Config struct reference
/// * `key` - The key reference to search for
///
/// # Returns
/// * `String` - The value for the key of a String type
///
/// # Panics
/// Will panic if the key cannot be read from the Config struct
pub(crate) fn get_cfg_string(cfg: &Config, key: &str) -> String {
    match cfg
        .get_string(key)
        .with_context(|| format!("Failed to read {key} from Config struct"))
    {
        Ok(rv) => rv,
        Err(e) => {
            panic!("{e:#}");
        }
    }
}

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
pub(crate) fn build_db_connection_string(cfg: &Config) -> String {
    // Parse config with error handling
    format!(
        "postgres://{}:{}@{}:{}/{}",
        get_cfg_string(cfg, "user"),
        get_cfg_string(cfg, "password"),
        get_cfg_string(cfg, "host"),
        get_cfg::<u16>(cfg, "port"),
        get_cfg_string(cfg, "dbname")
    )
}

/// Get the serial port to listen on
///
/// Either the serial port will be found in the config file, or the serial port will be specified
/// at the command line by the user.
///
/// # Arguments
/// * `cfg` - The Config struct reference
///
/// # Returns
/// * `String` - The path of the serial port as a string
///
/// # Panics
/// This panics if a serial port is not provided by the user in the case that the config file does
/// not provide a serial port path
pub(crate) fn get_serial_port(cfg: &Config) -> String {
    match cfg
        .get_string("serial_port")
        .with_context(|| "Failed to read serial_port from config file")
    {
        Err(e) => {
            error!("{e:#}");
            warn!("Prompting user for serial port instead");
            match available_serial_ports()
                .with_context(|| "Failed to enumerate list of serial ports")
            {
                Ok(ap) => println!("Available ports: {ap:?}"),
                Err(e) => {
                    error!("{e:#}");
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
                    panic!("{e:#}");
                }
            }
        }
        Ok(sp) => sp,
    }
}
