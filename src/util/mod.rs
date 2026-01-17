use chrono::{NaiveDateTime, Utc};

/// Config file interaction module
pub(crate) mod config;
/// Set logger for CLI module
pub(crate) mod log;
/// Local state of the program (necessary evil due to requests for features)
pub(crate) mod state;

/// Create a timestamp from a given epoch `u32`
///
/// # Arguments
/// * `epoch` - u32 epoch value
///
/// # Returns
/// * `NaiveDateTime` - from a u32 epoch value or the `Utc::now()` value of the daemon if the epoch value is
///   zero
///
/// # Panics
/// * If the epoch is more than 250,000 year from the common era or if the nanosecs is > 2
#[inline]
pub(crate) fn timestamp(epoch: u32) -> NaiveDateTime {
    if epoch > 1_735_689_600 {
        // Not recording timestamps earlier than 01/01/2025 12:00:00
        #[allow(deprecated)]
        NaiveDateTime::from_timestamp(epoch.into(), 0)
    } else {
        Utc::now().naive_utc()
    }
}
