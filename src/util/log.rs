//! Logging setup

#[cfg(feature = "journald")]
use anyhow::Context as _;
use anyhow::Result;
#[cfg(not(feature = "journald"))]
use tracing_subscriber::fmt::time::ChronoLocal;

/// Initializes the global logger
#[expect(
    clippy::unnecessary_wraps,
    reason = "Conditional compilation uses Result for journald feature"
)]
pub(crate) fn set_logger() -> Result<()> {
    // Standard output when not using journald
    #[cfg(not(feature = "journald"))]
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(ChronoLocal::rfc_3339())
        .init();

    // Direct journald integration — structured fields preserved
    #[cfg(feature = "journald")]
    tracing_journald::layer()
        .context("Failed to connect to journald")?
        .init();

    Ok(())
}
