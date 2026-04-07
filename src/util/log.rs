//! Logging setup

#[cfg(feature = "journald")]
use anyhow::Context as _;
use anyhow::Result;
#[cfg(not(feature = "journald"))]
use tracing_subscriber::fmt::{layer, time::ChronoLocal};
use tracing_subscriber::{
    EnvFilter, Layer as _, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

/// Initializes the global logger
#[expect(
    clippy::unnecessary_wraps,
    reason = "Conditional compilation uses Result for journald feature"
)]
pub(crate) fn set_logger() -> Result<()> {
    let app_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if cfg!(feature = "trace") {
            EnvFilter::new("trace")
        } else if cfg!(feature = "debug") {
            EnvFilter::new("info")
        } else {
            EnvFilter::new("warn")
        }
    });

    // Standard output when not using journald
    #[cfg(not(feature = "journald"))]
    tracing_subscriber::registry()
        .with(
            layer()
                .with_target(false)
                .with_timer(ChronoLocal::rfc_3339())
                .with_filter(app_filter),
        )
        .init();

    // Direct journald integration — structured fields preserved
    #[cfg(feature = "journald")]
    tracing_subscriber::registry()
        .with(
            tracing_journald::layer()
                .context("Failed to connect to journald")?
                .with_filter(app_filter),
        )
        .init();

    Ok(())
}
