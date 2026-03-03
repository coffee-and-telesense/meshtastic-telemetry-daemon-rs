use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initializes the global logger
pub(crate) fn set_logger() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(feature = "debug") {
            EnvFilter::new("debug")
        } else {
            EnvFilter::new("warn")
        }
    });

    let registry = tracing_subscriber::registry().with(filter);

    #[cfg(feature = "trace")]
    {
        let console_layer = console_subscriber::spawn();
        registry.with(console_layer).init();
    }

    #[cfg(feature = "journald")]
    {
        // Direct journald integration — structured fields preserved
        let journald = tracing_journald::layer().expect("failed to connect to journald");
        registry.with(journald).init();
    }

    #[cfg(not(any(feature = "journald", feature = "trace")))]
    {
        // Fallback: standard output with timestamps
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_timer(tracing_subscriber::fmt::time::LocalTime::rfc_3339());
        registry.with(fmt_layer).init();
    }
}

/// Logs tokio runtime metrics (workers, alive tasks, queue depth).
#[cfg(feature = "log_perf")]
#[inline]
pub(crate) fn log_perf() {
    use tokio::runtime::Handle;

    let metrics = Handle::current().metrics();
    let nw = metrics.num_workers();
    let nat = metrics.num_alive_tasks();
    let gqd = metrics.global_queue_depth();
    tracing::info!(
        workers = nw,
        alive_tasks = nat,
        queue_depth = gqd,
        "runtime perf"
    );
}
