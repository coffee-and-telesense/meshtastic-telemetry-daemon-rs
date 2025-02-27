#[cfg(not(debug_assertions))]
use anyhow::Context;
#[cfg(not(debug_assertions))]
use log::LevelFilter;
#[cfg(not(debug_assertions))]
use syslog::{BasicLogger, Facility, Formatter3164};

/// Set logger for CLI
///
/// If building a debug build, this will log to stdout using colog.
///
/// If building a release build, this will log to the system log using syslog.
///
/// # Arguments
/// None
///
/// # Returns
/// None
pub(crate) fn set_logger() {
    #[cfg(feature = "debug")]
    colog::init();
    #[cfg(not(debug_assertions))]
    {
        let formatter = Formatter3164 {
            facility: Facility::LOG_USER, //TODO: this could probably be something else, check libc
            hostname: None,
            process: "mesh_telem".into(),
            pid: 0,
        };
        match syslog::unix(formatter).with_context(|| "Could not connect to syslog posix socket") {
            Ok(logger) => {
                let _ = log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
                    .map(|()| log::set_max_level(LevelFilter::Warn))
                    .with_context(|| "Failed to set logger to syslog")
                    .inspect_err(|e| {
                        error!("{e}");
                        warn!("Continuing execution");
                    });
            }
            Err(e) => {
                error!("{e}");
                warn!("Continuing execution");
            }
        }
    }
}
