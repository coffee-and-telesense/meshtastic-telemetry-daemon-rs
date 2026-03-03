use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initializes the global logger
pub(crate) fn set_logger() {
    let app_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(feature = "debug") {
            EnvFilter::new("debug")
        } else {
            EnvFilter::new("warn")
        }
    });

    let registry = tracing_subscriber::registry();

    #[cfg(feature = "trace")]
    {
        use tracing_subscriber::{Layer, fmt::writer::MakeWriterExt};

        let console_layer = console_subscriber::spawn();
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_filter(app_filter);

        //TODO: tokio-console not showing anything with this pattern below
        //TODO: better timestamps (maybe use Chrono?) to get local time

        registry.with(console_layer).with(fmt_layer).init();
    }

    #[cfg(feature = "journald")]
    {
        // Direct journald integration — structured fields preserved
        let journald = tracing_journald::layer()
            .expect("failed to connect to journald")
            .with_filter(app_filter);
        registry.with(journald).init();
    }

    #[cfg(not(any(feature = "journald", feature = "trace")))]
    {
        // Fallback: standard output with timestamps
        use tracing_subscriber::Layer;
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_filter(app_filter);
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
