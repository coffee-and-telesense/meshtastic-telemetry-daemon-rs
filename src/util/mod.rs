use chrono::{DateTime, NaiveDateTime, Utc};

/// Config file interaction module
pub(crate) mod config;
/// Set logger for CLI module
pub(crate) mod log;
/// Local state of the program (necessary evil due to requests for features)
pub(crate) mod state;

/// Minimum valid epoch — 2025-01-01 12:00:00 UTC.
///
/// Packets with timestamps at or before this value are treated as
/// having no valid time, falling back to the daemon's wall clock.
/// This filters out uninitialized devices that report epoch 0 or
/// small values from their RTC.
const MIN_VALID_EPOCH: u32 = 1_735_689_600;

/// Maximum concurrent packet-processing tasks.
///
/// Bounded to twice the DB pool size (so tasks can overlap decode
/// and I/O) but capped at 32 to prevent memory pressure on
/// embedded targets like `BeagleBone` an`OpenWRT`.
pub const MAX_INFLIGHT_TASKS: usize = 32;

/// Create a timestamp from a given epoch `u32`
///
/// # Arguments
/// * `epoch` - epoch value (`u32`)
///
/// # Returns
/// * `NaiveDateTime` - from an epoch value (`u32`) or the `Utc::now()` value of the daemon if the epoch value is
///   zero
///
/// # Panics
/// * If the epoch is more than 250,000 year from the common era or if the nanoseconds is > 2
#[inline]
pub(crate) fn timestamp(epoch: u32) -> NaiveDateTime {
    if epoch > MIN_VALID_EPOCH {
        // Not recording timestamps earlier than 01/01/2025 12:00:00
        DateTime::from_timestamp(i64::from(epoch), 0)
            .map_or_else(|| Utc::now().naive_utc(), |dt| dt.naive_utc())
    } else {
        Utc::now().naive_utc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_epoch_returns_correct_datetime() {
        // 2025-01-01 12:00:01 UTC
        let dt = timestamp(1_735_689_601);
        assert_eq!(
            dt,
            DateTime::from_timestamp(1_735_689_601, 0)
                .unwrap()
                .naive_utc()
        );
    }

    #[test]
    fn zero_epoch_falls_back_to_now() {
        let before = Utc::now().naive_utc();
        let dt = timestamp(0);
        let after = Utc::now().naive_utc();
        assert!(dt >= before && dt <= after);
    }

    #[test]
    fn boundary_epoch_falls_back_to_now() {
        // Exactly the cutoff — should use Utc::now()
        let before = Utc::now().naive_utc();
        let dt = timestamp(1_735_689_600);
        let after = Utc::now().naive_utc();
        assert!(dt >= before && dt <= after);
    }

    #[test]
    fn one_above_boundary_uses_packet_time() {
        let dt = timestamp(1_735_689_601);
        assert_eq!(dt.and_utc().timestamp(), 1_735_689_601);
    }

    #[test]
    fn max_u32_epoch() {
        // u32::MAX = 4294967295, which is 2106-02-07
        // Should succeed, not fall back
        let dt = timestamp(u32::MAX);
        assert_eq!(dt.and_utc().timestamp(), i64::from(u32::MAX));
    }
}
