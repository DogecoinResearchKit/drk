use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use derive_more::Deref;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vecdb::{CheckedSub, Formattable, Pco};

use super::{CentsSats, Dollars, Sats, StoredF64};

/// Number of `Cents` integer units per US dollar — the fixed-point price scale
/// shared by every USD integer type (`Cents`, `CentsSigned`, `CentsCompact`,
/// `CentsSats`, ...).
///
/// Standard builds use 100 (true cents). Dogecoin trades far below one US cent
/// for most of its history, so the `dogecoin` build stores prices in
/// micro-dollars (1e-6 USD) to preserve sub-cent resolution. Every USD↔integer
/// conversion multiplies/divides by this constant and `to_dollars` inverts it,
/// so `Dollars`-typed values stay correct regardless of the scale. The scale is
/// compile-time (per-chain feature), so no cross-chain `u128` overflow applies
/// to a single-chain binary.
#[cfg(not(feature = "dogecoin"))]
pub const CENTS_PER_DOLLAR: u64 = 100;
#[cfg(feature = "dogecoin")]
pub const CENTS_PER_DOLLAR: u64 = 1_000_000;

/// `CENTS_PER_DOLLAR` as `f64`, for float conversions.
pub const CENTS_PER_DOLLAR_F64: f64 = CENTS_PER_DOLLAR as f64;

/// Unsigned cents (u64) - for values that should never be negative.
/// Used for invested capital, realized cap, etc.
#[derive(
    Debug,
    Default,
    Deref,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Pco,
    JsonSchema,
)]
pub struct Cents(u64);

impl Cents {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u64::MAX);

    #[inline]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[inline(always)]
    pub const fn inner(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn as_u128(self) -> u128 {
        self.0 as u128
    }

    #[inline]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    #[inline]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    #[inline]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    pub fn to_dollars(self) -> Dollars {
        Dollars::from(self.0 as f64 / CENTS_PER_DOLLAR_F64)
    }

    /// Round to N significant digits.
    /// E.g., 12345 (= $123.45) with round_to(4) → 12350 (= $123.50)
    /// E.g., 12345 (= $123.45) with round_to(3) → 12300 (= $123.00)
    pub fn round_to(self, digits: i32) -> Self {
        let v = self.0;
        let ilog10 = v.checked_ilog10().unwrap_or(0) as i32;
        if ilog10 >= digits {
            let log_diff = ilog10 - digits + 1;
            let pow = 10u64.pow(log_diff as u32);
            // Add half for rounding
            Self(((v + pow / 2) / pow) * pow)
        } else {
            self
        }
    }

    /// Round to nearest dollar, then apply N significant digits.
    /// E.g., 12345 (= $123.45) → 12300 (= $123.00) with 5 digits
    /// E.g., 1234567 (= $12345.67) → 1234600 (= $12346.00) with 5 digits
    #[inline]
    pub fn round_to_dollar(self, digits: i32) -> Self {
        // Round to nearest dollar (nearest CENTS_PER_DOLLAR units)
        let dollars = (self.0 + CENTS_PER_DOLLAR / 2) / CENTS_PER_DOLLAR;
        // Apply significant digit rounding to dollars, then convert back to cents
        let ilog10 = dollars.checked_ilog10().unwrap_or(0) as i32;
        let rounded_dollars = if ilog10 >= digits {
            let log_diff = ilog10 - digits + 1;
            let pow = 10u64.pow(log_diff as u32);
            ((dollars + pow / 2) / pow) * pow
        } else {
            dollars
        };
        Self(rounded_dollars * CENTS_PER_DOLLAR)
    }
}

impl From<Dollars> for Cents {
    #[inline]
    fn from(value: Dollars) -> Self {
        let f = f64::from(value);
        if f.is_nan() || f < 0.0 {
            Self::ZERO
        } else {
            Self((f * CENTS_PER_DOLLAR_F64).round() as u64)
        }
    }
}

impl From<Cents> for Dollars {
    #[inline]
    fn from(value: Cents) -> Self {
        value.to_dollars()
    }
}

impl From<u64> for Cents {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<usize> for Cents {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u64)
    }
}

impl From<Cents> for u64 {
    #[inline]
    fn from(value: Cents) -> Self {
        value.0
    }
}

impl From<u128> for Cents {
    #[inline]
    fn from(value: u128) -> Self {
        debug_assert!(value <= u64::MAX as u128, "u128 overflow to CentsUnsigned");
        Self(value as u64)
    }
}

impl From<Cents> for u128 {
    #[inline]
    fn from(value: Cents) -> Self {
        value.0 as u128
    }
}

impl From<Cents> for f64 {
    #[inline]
    fn from(value: Cents) -> Self {
        value.0 as f64
    }
}

impl From<f64> for Cents {
    #[inline]
    fn from(value: f64) -> Self {
        if value.is_nan() || value < 0.0 {
            Self::ZERO
        } else {
            Self(value as u64)
        }
    }
}

impl Add for Cents {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Cents {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sum for Cents {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|c| c.0).sum())
    }
}

impl Sub for Cents {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Cents {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul for Cents {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_mul(rhs.0).expect("CentsUnsigned overflow"))
    }
}

impl Mul<u64> for Cents {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: u64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Mul<usize> for Cents {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0 * rhs as u64)
    }
}

impl Mul<StoredF64> for Cents {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: StoredF64) -> Self::Output {
        Self::from(f64::from(self) * f64::from(rhs))
    }
}

impl Mul<Sats> for Cents {
    type Output = CentsSats;
    #[inline]
    fn mul(self, sats: Sats) -> CentsSats {
        CentsSats::new(self.as_u128() * sats.as_u128())
    }
}

impl Div<Cents> for Cents {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Div<u64> for Cents {
    type Output = Self;
    #[inline]
    fn div(self, rhs: u64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl Div<usize> for Cents {
    type Output = Self;
    #[inline]
    fn div(self, rhs: usize) -> Self::Output {
        Self(self.0 / rhs as u64)
    }
}

impl CheckedSub for Cents {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl std::fmt::Display for Cents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = itoa::Buffer::new();
        let str = buf.format(self.0);
        f.write_str(str)
    }
}

impl Formattable for Cents {
    #[inline(always)]
    fn write_to(&self, buf: &mut Vec<u8>) {
        let mut b = itoa::Buffer::new();
        buf.extend_from_slice(b.format(self.0).as_bytes());
    }
}
