use brk_types::{Sats, Timestamp};

/// Age of a UTXO or address balance.
#[derive(Debug, Clone, Copy)]
pub struct Age {
    /// Age in hours (primary internal unit for cohort boundaries)
    hours: usize,
    /// Fractional age in days, for satoshi-days destroyed.
    days: f64,
}

impl Age {
    #[inline]
    pub fn new(current_timestamp: Timestamp, prev_timestamp: Timestamp) -> Self {
        Self {
            hours: current_timestamp.hours_since(prev_timestamp),
            days: current_timestamp.days_since(prev_timestamp),
        }
    }

    /// Hours old (for cohort bucket lookup via HOURS_* boundaries)
    #[inline]
    pub fn hours(&self) -> usize {
        self.hours
    }

    /// Supply in satoshis times fractional age in days, rounded down. Wider than `u64`: a
    /// Dogecoin whale output passes `u64::MAX` sat-days in a few years.
    #[inline]
    pub fn satdays_destroyed(&self, supply: Sats) -> u128 {
        (u64::from(supply) as f64 * self.days).floor() as u128
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn satdays_destroyed_does_not_wrap_past_u64() {
        let age = Age {
            hours: 24_000,
            days: 1_000.0,
        };
        let supply = Sats::from(1_000_000_000 * Sats::ONE_BTC_U64);
        let satdays = age.satdays_destroyed(supply);
        assert!(satdays > u128::from(u64::MAX));
        assert_eq!(satdays, 100_000_000_000_000_000_000);
    }
}
