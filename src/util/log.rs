use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

/// Initializes the global logger
pub(crate) fn set_logger() {
    let app_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(feature = "trace") {
            EnvFilter::new("trace")
        } else if cfg!(feature = "debug") {
            EnvFilter::new("info")
        } else {
            EnvFilter::new("warn")
        }
    });

    let registry = tracing_subscriber::registry();

    // Standard output when not using journald
    #[cfg(not(feature = "journald"))]
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
        .with_filter(app_filter);

    // Direct journald integration — structured fields preserved
    #[cfg(feature = "journald")]
    let journald = tracing_journald::layer()
        .expect("failed to connect to journald")
        .with_filter(app_filter);

    #[cfg(feature = "tokio-console")]
    let console_layer = console_subscriber::spawn();

    // Adding in tokio-console support to standard output
    #[cfg(all(feature = "tokio-console", not(feature = "journald")))]
    registry.with(console_layer).with(fmt_layer).init();

    #[cfg(all(feature = "tokio-console", feature = "journald"))]
    registry.with(console_layer).with(journald).init();

    #[cfg(all(not(feature = "tokio-console"), feature = "journald"))]
    registry.with(journald).init();

    // Fallback: standard output with timestamps, no tokio-console or journald
    #[cfg(not(any(feature = "journald", feature = "tokio-console")))]
    registry.with(fmt_layer).init();
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
