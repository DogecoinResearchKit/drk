use bitview_compute::NumericValue;
use brk_error::Result;
use brk_types::{Bitcoin, Cents, CentsTotal, Dollars, Sats, SatsSigned, SatsTotal, Version};
use schemars::JsonSchema;
use vecdb::{Convert, Database, PcoVecValue, Rw};

use crate::{IndexSources, LazyPerBlock, PerBlock, Value};

/// A sats type that converts to bitcoin.
pub trait AmountType: NumericValue + JsonSchema + Into<Bitcoin> {}

impl AmountType for Sats {}

impl AmountType for SatsSigned {}

/// Sats and cents each own one shared source cache.
pub type ValuePerBlock<M = Rw, S = Sats, C = Cents> = Value<
    PerBlock<S, M>,
    PerBlock<C, M>,
    LazyPerBlock<Bitcoin, S>,
    LazyPerBlock<Dollars, C>,
>;

/// Running totals: [`SatsTotal`] and [`CentsTotal`].
pub type ValueTotalPerBlock<M = Rw> = ValuePerBlock<M, SatsTotal, CentsTotal>;

impl<S, C> ValuePerBlock<Rw, S, C>
where
    S: PcoVecValue + PartialOrd + JsonSchema + Into<Bitcoin> + 'static,
    C: PcoVecValue + PartialOrd + JsonSchema + Into<Dollars> + 'static,
{
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let sats = PerBlock::import(db, &format!("{name}_sats"), version, indexes)?;

        let btc = LazyPerBlock::from_resolutions::<Convert>(name, version, &sats);

        let cents = PerBlock::import(db, &format!("{name}_cents"), version, indexes)?;

        let usd =
            LazyPerBlock::from_resolutions::<Convert>(&format!("{name}_usd"), version, &cents);

        Ok(Self {
            btc,
            sats,
            usd,
            cents,
        })
    }
}
