#[cfg(feature = "syslog")]
use anyhow::Context;
use chrono::Local;
use log::{Level, LevelFilter, debug, error, info, trace, warn};
#[cfg(feature = "syslog")]
use syslog::{BasicLogger, Facility, Formatter3164};
use tokio::sync::OnceCell;

/// The global boolean indicating if the logger is set or not
static LOGGER_SET: OnceCell<bool> = OnceCell::const_new_with(false);

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
    // Check if global logger is set, if not set the global logger
    if !LOGGER_SET.get().expect("Could not get status of logger") {
        // Set the global bool to true
        match LOGGER_SET.set(true) {
            Ok(()) => log_msg("Set logger", log::Level::Info),
            Err(e) => log_msg(&format!("Failed to set logger {e}"), log::Level::Error),
        }

        // Initialize the log depending on feature used
        #[cfg(feature = "colog")]
        {
            #[cfg(feature = "debug")]
            log::set_max_level(LevelFilter::Trace);
            #[cfg(not(feature = "debug"))]
            log::set_max_level(LevelFilter::Info);
            colog::init();
        }
        #[cfg(feature = "syslog")]
        {
            let formatter = Formatter3164 {
                facility: Facility::LOG_USER, //TODO: this could probably be something else, check libc
                hostname: None,
                process: "mesh_telem".into(),
                pid: 0,
            };
            match syslog::unix(formatter)
                .with_context(|| "Could not connect to syslog posix socket")
            {
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
}

/// Log a message to the system logger with timestamp
///
/// # Arguments
/// * `msg` - an arbitrary String message to print
/// * `lvl` - the log level to use
///
/// # Returns
/// None
#[inline]
pub(crate) fn log_msg(msg: &str, lvl: Level) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S - ");
    match lvl {
        Level::Error => error!("{now}{msg}"),
        Level::Warn => warn!("{now}{msg}"),
        Level::Info => info!("{now}{msg}"),
        Level::Debug => debug!("{now}{msg}"),
        Level::Trace => trace!("{now}{msg}"),
    }
}

/// Performance metrics with regular printing for debugging
#[cfg(feature = "debug")]
#[inline]
pub(crate) fn log_perf() {
    use tokio::runtime::Handle;
    let metrics = Handle::current().metrics();
    let nw = metrics.num_workers();
    let nat = metrics.num_alive_tasks();
    let gqd = metrics.global_queue_depth();
    log_msg(
        &format!("RUNTIME PERF: {nw} workers used, {nat} alive tasks, {gqd} global queue depth"),
        log::Level::Debug,
    );
}
