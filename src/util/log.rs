//! Logging setup

#[cfg(feature = "journald")]
use anyhow::Context as _;
use anyhow::Result;
#[cfg(not(feature = "journald"))]
use tracing_subscriber::fmt::{layer, time::ChronoLocal};
use tracing_subscriber::{Layer as _, layer::SubscriberExt as _, util::SubscriberInitExt as _};

/// Initializes the global logger
#[expect(
    clippy::unnecessary_wraps,
    reason = "Conditional compilation uses Result for journald feature"
)]
pub(crate) fn set_logger() -> Result<()> {
    // Standard output when not using journald
    #[cfg(not(feature = "journald"))]
    tracing_subscriber::registry()
        .with(
            layer()
                .with_target(false)
                .with_timer(ChronoLocal::rfc_3339()),
        )
        .init();

    // Direct journald integration — structured fields preserved
    #[cfg(feature = "journald")]
    tracing_subscriber::registry()
        .with(tracing_journald::layer().context("Failed to connect to journald")?)
        .init();

    Ok(())
}
