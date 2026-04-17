//! Logging setup

#[cfg(feature = "journald")]
use anyhow::Context as _;
use anyhow::Result;
#[cfg(not(feature = "journald"))]
use tracing::subscriber::set_global_default;

/// Initializes the global logger
pub(crate) fn set_logger() -> Result<()> {
    // Standard output when not using journald
    #[cfg(not(feature = "journald"))]
    set_global_default(subscriber::MinimalSubscriber::new())?;

    // Direct journald integration — structured fields preserved
    #[cfg(feature = "journald")]
    tracing_journald::layer()
        .context("Failed to connect to journald")?
        .init();

    Ok(())
}

#[cfg(not(feature = "journald"))]
mod subscriber {
    use core::{error::Error, fmt::Debug};
    use std::io::{Write as _, stderr};
    use tracing::{
        Event, Level, Metadata,
        field::{Field, Visit},
        span::Id,
        subscriber::Subscriber,
    };
    use tracing_core::span;

    /// Minimal field visitor — prints key=value pairs to stderr
    struct FieldPrinter;

    impl Visit for FieldPrinter {
        fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
            if field.name() == "message" {
                eprint!("{value:?}");
            } else {
                eprint!(" {}={value:?}", field.name());
            }
        }

        fn record_str(&mut self, field: &Field, value: &str) {
            if field.name() == "message" {
                eprint!("{value}");
            } else {
                eprint!(" {}={value}", field.name());
            }
        }

        fn record_i64(&mut self, field: &Field, value: i64) {
            eprint!(" {}={value}", field.name());
        }

        fn record_u64(&mut self, field: &Field, value: u64) {
            eprint!(" {}={value}", field.name());
        }

        fn record_bool(&mut self, field: &Field, value: bool) {
            eprint!(" {}={value}", field.name());
        }

        fn record_error(&mut self, field: &Field, value: &(dyn Error + 'static)) {
            eprint!(" {}={value}", field.name());
        }
    }

    /// Minimal subscriber
    pub(crate) struct MinimalSubscriber {
        /// Minimum level to emit
        min_level: Level,
    }

    impl MinimalSubscriber {
        /// Creates a subscriber that emits events at or above `min_level` based on features
        /// e.g. trace sets it to TRACE, debug sets it to INFO, otherwise WARN is the minimum
        pub(crate) const fn new() -> Self {
            let level = if cfg!(feature = "trace") {
                Level::TRACE
            } else if cfg!(feature = "debug") {
                Level::INFO
            } else {
                Level::WARN
            };
            Self { min_level: level }
        }
    }

    impl Subscriber for MinimalSubscriber {
        fn enabled(&self, metadata: &Metadata<'_>) -> bool {
            metadata.level() <= &self.min_level
        }

        fn new_span(&self, _: &span::Attributes<'_>) -> Id {
            // No span tracking — return a dummy id
            Id::from_u64(1)
        }

        fn event(&self, event: &Event<'_>) {
            let meta = event.metadata();
            let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%:z");
            eprint!("{now} {level} ", level = meta.level());
            event.record(&mut FieldPrinter);
            drop(stderr().flush());
            eprintln!();
        }

        fn record(&self, _: &Id, _: &span::Record<'_>) {}
        fn record_follows_from(&self, _: &Id, _: &Id) {}
        fn enter(&self, _: &Id) {}
        fn exit(&self, _: &Id) {}
    }
}
