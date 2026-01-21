#[cfg(feature = "syslog")]
use anyhow::Context;
use chrono::Local;
use log::LevelFilter;
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    sync::Once,
};
#[cfg(feature = "syslog")]
use syslog::{BasicLogger, Facility, Formatter3164};

/// The global boolean indicating if the logger is set or not
static INIT_LOGGER: Once = Once::new();

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
    INIT_LOGGER.call_once(|| {
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
                process: String::from("mesh_telem"),
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
    });
}

thread_local! {
    /// Tokio worker safe buffer for storing the timestamp cached string
    static TS_CACHE: RefCell<(i64, String)> = RefCell::new((0, String::with_capacity(32)));
}

/// Timestamp cached string
pub(crate) struct CachedTs;

/// Display the cached timestamp string only invalidating if a second has passed
impl Display for CachedTs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let now = Local::now();
        let sec = now.timestamp();

        TS_CACHE.with(|cell| {
            use std::fmt::Write;

            let mut cache = cell.borrow_mut();
            if cache.0 != sec {
                cache.0 = sec;
                cache.1.clear();
                write!(cache.1, "{} - ", now.format("%Y-%m-%d %H:%M:%S"))?;
            }
            f.write_str(&cache.1)
        })
    }
}

/// Performance metrics with regular printing for debugging
#[cfg(feature = "log_perf")]
#[inline]
pub(crate) fn log_perf() {
    use crate::log_msg;
    use tokio::runtime::Handle;

    let metrics = Handle::current().metrics();
    let nw = metrics.num_workers();
    let nat = metrics.num_alive_tasks();
    let gqd = metrics.global_queue_depth();
    log_msg!(
        log::Level::Info,
        "RUNTIME PERF: {nw} workers used, {nat} alive tasks, {gqd} global queue depth"
    );
}

/// Log a message to the system logger with timestamp
///
/// # Arguments
/// * `msg` - an arbitrary String message to print
/// * `lvl` - the log level to use
///
/// # Returns
/// None
#[macro_export]
macro_rules! log_msg {
    ($lvl:expr, $($arg:tt)*) => {{
        match $lvl {
            log::Level::Error => log::error!("{}{}", $crate::util::log::CachedTs, format_args!($($arg)*)),
            log::Level::Warn  => log::warn!("{}{}",  $crate::util::log::CachedTs, format_args!($($arg)*)),
            log::Level::Info  => log::info!("{}{}",  $crate::util::log::CachedTs, format_args!($($arg)*)),
            log::Level::Debug => log::debug!("{}{}", $crate::util::log::CachedTs, format_args!($($arg)*)),
            log::Level::Trace => log::trace!("{}{}", $crate::util::log::CachedTs, format_args!($($arg)*)),
        }
    }};
}
