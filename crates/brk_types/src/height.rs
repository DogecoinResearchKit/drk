use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Rem},
};

use bitcoin::locktime::absolute::Height as AbsoluteHeight;
use derive_more::{Add, Deref, From, Into};
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CheckedSub, Sats};

#[cfg(feature = "storage")]
use std::io::Error as IoError;

#[cfg(feature = "storage")]
use std::path::Path;

#[cfg(feature = "storage")]
use std::fs;
#[cfg(feature = "storage")]
use vecdb::{Bytes, Pco, PrintableIndex, Stamp, VecIndex};

#[cfg(not(feature = "dogecoin"))]
pub const BLOCKS_PER_DIFF_EPOCH: u32 = 2016;
/// Dogecoin retargeted every 240 blocks until Digishield (145,000) and every block since; the
/// pre-Digishield interval keeps difficulty epochs a usable (and `u16`-sized) index.
#[cfg(feature = "dogecoin")]
pub const BLOCKS_PER_DIFF_EPOCH: u32 = 240;

#[cfg(not(feature = "dogecoin"))]
pub const BLOCKS_PER_HALVING: u32 = 210_000;
/// Dogecoin's reward halved every 100,000 blocks until 600,000, then stayed at 10,000 DOGE.
#[cfg(feature = "dogecoin")]
pub const BLOCKS_PER_HALVING: u32 = 100_000;

/// About four years of blocks: the span windows sized "one Bitcoin halving" stand for.
#[cfg(not(feature = "dogecoin"))]
pub const HALVING_CYCLE_BLOCKS: u32 = BLOCKS_PER_HALVING;
#[cfg(feature = "dogecoin")]
pub const HALVING_CYCLE_BLOCKS: u32 = 4 * crate::TARGET_BLOCKS_PER_YEAR as u32;

/// The first height with a deterministic Dogecoin subsidy; earlier rewards were random.
#[cfg(feature = "dogecoin")]
pub const DOGECOIN_SIMPLIFIED_REWARDS_HEIGHT: u32 = 145_000;

/// A block height: how many blocks came before it.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Deref,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    Hash,
    Add,
    From,
    Into,
)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "storage", derive(Pco))]
#[allow(clippy::duplicated_attributes)]
#[cfg_attr(
    feature = "schemars",
    schemars(example = 0, example = 210_000, example = 420_000, example = 840_000)
)]
pub struct Height(u32);

impl Height {
    pub const ZERO: Self = Self(0);
    pub const fn new(height: u32) -> Self {
        Self(height)
    }

    #[cfg(feature = "storage")]
    pub fn write(&self, path: &Path) -> Result<(), IoError> {
        fs::write(path, self.to_bytes())
    }

    pub fn incremented(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn decremented(self) -> Option<Self> {
        CheckedSub::checked_sub(self, 1_u32)
    }

    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    pub fn left_before_next_diff_adj(self) -> u32 {
        BLOCKS_PER_DIFF_EPOCH - (*self % BLOCKS_PER_DIFF_EPOCH)
    }

    pub fn left_before_next_halving(self) -> u32 {
        BLOCKS_PER_HALVING - (*self % BLOCKS_PER_HALVING)
    }

    /// Bitcoin Core's `GetBlockSubsidy`: 50 BTC shifted right once per halving.
    #[cfg(not(feature = "dogecoin"))]
    pub fn subsidy(self) -> Sats {
        Sats::from(
            u64::from(Sats::FIFTY_BTC)
                .checked_shr(self.0 / BLOCKS_PER_HALVING)
                .unwrap_or(0),
        )
    }

    /// Dogecoin Core's `GetDogecoinBlockSubsidy`. Before 145,000 the reward was drawn at random
    /// up to `(1,000,000 >> halvings) - 1` DOGE; that cap is returned there. Then
    /// `500,000 >> halvings` DOGE until 600,000, and 10,000 DOGE from then on.
    #[cfg(feature = "dogecoin")]
    pub fn subsidy(self) -> Sats {
        let halvings = self.0 / BLOCKS_PER_HALVING;
        let doge = if self.0 < DOGECOIN_SIMPLIFIED_REWARDS_HEIGHT {
            (1_000_000_u64 >> halvings) - 1
        } else if halvings < 6 {
            500_000_u64 >> halvings
        } else {
            10_000
        };
        Sats::from(doge * Sats::ONE_BTC_U64)
    }

    pub fn index_name() -> &'static str {
        "height"
    }
}

impl PartialEq<u64> for Height {
    fn eq(&self, other: &u64) -> bool {
        u64::from(self.0) == *other
    }
}

impl Add<u32> for Height {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self::from(self.0 + rhs)
    }
}

impl Add<u64> for Height {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self::from(self.0 + rhs as u32)
    }
}

impl Add<usize> for Height {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::from(self.0 + rhs as u32)
    }
}

impl CheckedSub<Height> for Height {
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl CheckedSub<u32> for Height {
    fn checked_sub(self, rhs: u32) -> Option<Self> {
        self.0.checked_sub(rhs).map(Height::from)
    }
}

impl CheckedSub<usize> for Height {
    fn checked_sub(self, rhs: usize) -> Option<Self> {
        (self.0 as usize).checked_sub(rhs).map(|h| Self(h as u32))
    }
}

impl AddAssign<usize> for Height {
    fn add_assign(&mut self, rhs: usize) {
        *self = self.add(rhs);
    }
}

impl CheckedSub<u64> for Height {
    fn checked_sub(self, rhs: u64) -> Option<Self> {
        u64::from(self.0).checked_sub(rhs).map(|h| Self(h as u32))
    }
}

impl AddAssign<u64> for Height {
    fn add_assign(&mut self, rhs: u64) {
        *self = self.add(rhs);
    }
}

impl Rem<Height> for Height {
    type Output = Self;
    fn rem(self, rhs: Height) -> Self::Output {
        Self(self.0.rem(rhs.0))
    }
}

impl Rem<usize> for Height {
    type Output = Self;
    fn rem(self, rhs: usize) -> Self::Output {
        Self(self.0.rem(Height::from(rhs).0))
    }
}

impl From<u64> for Height {
    #[inline]
    fn from(value: u64) -> Self {
        Self(value as u32)
    }
}

impl From<usize> for Height {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl From<Height> for usize {
    #[inline]
    fn from(value: Height) -> Self {
        value.0 as usize
    }
}

impl From<Height> for u64 {
    #[inline]
    fn from(value: Height) -> Self {
        value.0 as u64
    }
}

impl From<AbsoluteHeight> for Height {
    #[inline]
    fn from(value: AbsoluteHeight) -> Self {
        Self(value.to_consensus_u32())
    }
}

crate::impl_store_value!(Height, u32);

#[cfg(feature = "storage")]
impl From<Stamp> for Height {
    #[inline]
    fn from(value: Stamp) -> Self {
        let u = u64::from(value);
        debug_assert!(u <= u32::MAX as u64);
        Self(u as u32)
    }
}

#[cfg(feature = "storage")]
impl From<Height> for Stamp {
    #[inline]
    fn from(value: Height) -> Self {
        Self::from(value.0 as u64)
    }
}

#[cfg(feature = "storage")]
impl PrintableIndex for Height {
    fn to_string() -> &'static str {
        Self::index_name()
    }
}

#[cfg(feature = "storage")]
impl VecIndex for Height {
    const INITIAL_CAPACITY: usize = 1_200_000;
}

crate::impl_int_format!(Height);
