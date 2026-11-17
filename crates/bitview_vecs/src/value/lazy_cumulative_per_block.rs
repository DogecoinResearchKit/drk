use brk_types::{Bitcoin, CentsTotal, Dollars, Height, SatsTotal, Version};
use vecdb::{Convert, Ident, ReadableCloneableVec};

use crate::{IndexSources, LazyPerBlock, Value};

pub type LazyCumulativeValuePerBlock = Value<
    LazyPerBlock<SatsTotal>,
    LazyPerBlock<CentsTotal>,
    LazyPerBlock<Bitcoin, SatsTotal>,
    LazyPerBlock<Dollars, CentsTotal>,
>;

impl LazyCumulativeValuePerBlock {
    pub fn from_sources(
        name: &str,
        version: Version,
        cumulative_sats: &(impl ReadableCloneableVec<Height, SatsTotal> + ?Sized),
        cumulative_cents: &(impl ReadableCloneableVec<Height, CentsTotal> + ?Sized),
        indexes: &IndexSources,
    ) -> Self {
        let sats = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_sats"),
            version,
            cumulative_sats,
            indexes,
        );
        let btc = LazyPerBlock::from_lazy::<Convert, SatsTotal>(name, version, &sats);
        let cents = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_cents"),
            version,
            cumulative_cents,
            indexes,
        );
        let usd =
            LazyPerBlock::from_lazy::<Convert, CentsTotal>(&format!("{name}_usd"), version, &cents);
        Self {
            btc,
            sats,
            usd,
            cents,
        }
    }
}
