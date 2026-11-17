//! Sums of amounts over many blocks: cumulatives and the rolling windows read from them.
//!
//! Bitcoin's sums fit their part's `u64`, so a total is the part itself. Dogecoin's all-time
//! transfer volume passed `u64::MAX` koinu (about 184.5B DOGE) within weeks of genesis, and its
//! all-time dollar flows near the micro-dollar limit; vecdb's Pco storage stops at `u64`. There a
//! total is stored as an `f64` of the part's unit and summed exactly in a `u128`.

#[cfg(feature = "dogecoin")]
use derive_more::{Add, AddAssign, Sub};
#[cfg(all(feature = "dogecoin", feature = "schemars"))]
use schemars::JsonSchema;
#[cfg(feature = "dogecoin")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "dogecoin")]
use crate::{Bitcoin, CENTS_PER_DOLLAR_F64, CheckedSub, Dollars};
use crate::{Cents, Sats};

#[cfg(all(feature = "dogecoin", feature = "storage"))]
use vecdb::Pco;

/// Added to the version of every series stored as a total.
#[cfg(all(feature = "storage", not(feature = "dogecoin")))]
pub const TOTAL_VERSION: vecdb::Version = vecdb::Version::ZERO;
#[cfg(all(feature = "storage", feature = "dogecoin"))]
pub const TOTAL_VERSION: vecdb::Version = vecdb::Version::ONE;

/// A sum of sats over many blocks.
#[cfg(not(feature = "dogecoin"))]
pub type SatsTotal = Sats;
/// A sum of cents over many blocks.
#[cfg(not(feature = "dogecoin"))]
pub type CentsTotal = Cents;

/// A total and the exact running sum it is stored from.
pub trait Total: Copy {
    /// What each block adds.
    type Part: Copy;
    /// The exact running sum.
    type Acc: Clone + Default;

    /// The running sum a stored total continues from.
    fn acc(self) -> Self::Acc;
    fn add(acc: &mut Self::Acc, part: Self::Part);
    fn from_acc(acc: &Self::Acc) -> Self;
}

#[cfg(not(feature = "dogecoin"))]
impl Total for Sats {
    type Part = Sats;
    type Acc = Sats;

    #[inline]
    fn acc(self) -> Sats {
        self
    }

    #[inline]
    fn add(acc: &mut Sats, part: Sats) {
        *acc += part;
    }

    #[inline]
    fn from_acc(acc: &Sats) -> Self {
        *acc
    }
}

#[cfg(not(feature = "dogecoin"))]
impl Total for Cents {
    type Part = Cents;
    type Acc = Cents;

    #[inline]
    fn acc(self) -> Cents {
        self
    }

    #[inline]
    fn add(acc: &mut Cents, part: Cents) {
        *acc += part;
    }

    #[inline]
    fn from_acc(acc: &Cents) -> Self {
        *acc
    }
}

/// A sum of koinu over many blocks, in koinu (whole until 2^53 koinu, about 90M DOGE, then
/// within one part in 2^53).
#[cfg(feature = "dogecoin")]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Add, AddAssign, Sub)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct SatsTotal(f64);

/// A sum of micro-dollars over many blocks, in micro-dollars; NaN where undefined.
#[cfg(feature = "dogecoin")]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Add, AddAssign, Sub)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
pub struct CentsTotal(f64);

#[cfg(feature = "dogecoin")]
mod dogecoin {
    use super::*;

    /// `CentsTotal`'s running sum once a NaN part made it undefined.
    const NAN_ACC: u128 = u128::MAX;

    impl Total for SatsTotal {
        type Part = Sats;
        type Acc = u128;

        #[inline]
        fn acc(self) -> u128 {
            self.0.max(0.0).round() as u128
        }

        #[inline]
        fn add(acc: &mut u128, part: Sats) {
            *acc += part.as_u128();
        }

        #[inline]
        fn from_acc(acc: &u128) -> Self {
            Self(*acc as f64)
        }
    }

    impl Total for CentsTotal {
        type Part = Cents;
        type Acc = u128;

        #[inline]
        fn acc(self) -> u128 {
            if self.0.is_nan() {
                NAN_ACC
            } else {
                self.0.max(0.0).round() as u128
            }
        }

        #[inline]
        fn add(acc: &mut u128, part: Cents) {
            match part.finite_inner() {
                Some(part) if *acc != NAN_ACC => *acc += part as u128,
                _ => *acc = NAN_ACC,
            }
        }

        #[inline]
        fn from_acc(acc: &u128) -> Self {
            if *acc == NAN_ACC {
                Self(f64::NAN)
            } else {
                Self(*acc as f64)
            }
        }
    }

    impl From<Sats> for SatsTotal {
        #[inline]
        fn from(value: Sats) -> Self {
            Self(f64::from(value))
        }
    }

    /// Rounded; a total over `u64::MAX` koinu saturates.
    impl From<SatsTotal> for Sats {
        #[inline]
        fn from(value: SatsTotal) -> Self {
            Sats::from(value.0.max(0.0))
        }
    }

    impl From<SatsTotal> for Bitcoin {
        #[inline]
        fn from(value: SatsTotal) -> Self {
            Bitcoin::from(value.0 / Sats::ONE_BTC_U64 as f64)
        }
    }

    impl From<f64> for SatsTotal {
        #[inline]
        fn from(value: f64) -> Self {
            Self(value)
        }
    }

    impl From<SatsTotal> for f64 {
        #[inline]
        fn from(value: SatsTotal) -> Self {
            value.0
        }
    }

    impl From<usize> for SatsTotal {
        #[inline]
        fn from(value: usize) -> Self {
            Self(value as f64)
        }
    }

    impl From<u64> for SatsTotal {
        #[inline]
        fn from(value: u64) -> Self {
            Self(value as f64)
        }
    }

    /// Deltas of a flow's running total; a rounding step below zero reads zero.
    impl CheckedSub for SatsTotal {
        #[inline]
        fn checked_sub(self, rhs: Self) -> Option<Self> {
            Some(Self(non_negative(self.0 - rhs.0)))
        }
    }

    /// Zero for a negative value; NaN stays NaN.
    #[inline]
    fn non_negative(value: f64) -> f64 {
        if value < 0.0 { 0.0 } else { value }
    }

    impl From<Cents> for CentsTotal {
        #[inline]
        fn from(value: Cents) -> Self {
            Self(f64::from(value))
        }
    }

    /// Rounded; NaN stays NaN and a total past `u64` saturates.
    impl From<CentsTotal> for Cents {
        #[inline]
        fn from(value: CentsTotal) -> Self {
            Cents::from(value.0)
        }
    }

    impl From<CentsTotal> for Dollars {
        #[inline]
        fn from(value: CentsTotal) -> Self {
            Dollars::from(value.0 / CENTS_PER_DOLLAR_F64)
        }
    }

    impl From<f64> for CentsTotal {
        #[inline]
        fn from(value: f64) -> Self {
            Self(value)
        }
    }

    impl From<CentsTotal> for f64 {
        #[inline]
        fn from(value: CentsTotal) -> Self {
            value.0
        }
    }

    impl From<usize> for CentsTotal {
        #[inline]
        fn from(value: usize) -> Self {
            Self(value as f64)
        }
    }

    impl From<u64> for CentsTotal {
        #[inline]
        fn from(value: u64) -> Self {
            Self(value as f64)
        }
    }

    /// An undefined side gives an undefined delta; a rounding step below zero reads zero.
    impl CheckedSub for CentsTotal {
        #[inline]
        fn checked_sub(self, rhs: Self) -> Option<Self> {
            Some(Self(non_negative(self.0 - rhs.0)))
        }
    }

    crate::impl_float_ord_format!(SatsTotal);
    crate::impl_float_ord_format!(CentsTotal);
}
