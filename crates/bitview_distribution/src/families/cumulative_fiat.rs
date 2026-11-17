use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{
    FiatType, LazyFiatPerBlockCumulativeWithSums, LazyWindowStartVec, TotalSource, shapes::Windows,
};
use brk_error::Result;
use brk_types::{Cents, CentsSigned, CentsTotal, Total, Version};
use vecdb::{Database, PcoVecValue, Rw, StorageMode};

/// A per-block fiat amount and the type its running total is stored as.
pub trait FiatFlow: FiatType {
    type Total: FiatType + PcoVecValue + Total<Part = Self>;
}

impl FiatFlow for Cents {
    type Total = CentsTotal;
}

impl FiatFlow for CentsSigned {
    type Total = CentsSigned;
}

/// One cohort's fiat flow, stored as its running total `{name}_cumulative_cents`.
#[derive(Traversable)]
pub struct CumulativeFiat<C: FiatFlow = Cents, M: StorageMode = Rw> {
    #[traversable(flatten)]
    value: LazyFiatPerBlockCumulativeWithSums<C::Total>,
    #[traversable(hidden)]
    pub stored: TotalSource<C::Total, M>,
}

impl<C: FiatFlow> CumulativeFiat<C> {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = TotalSource::import(
            db,
            &format!("{name}_cumulative_cents"),
            version + Version::TWO,
        )?;
        let value = LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
            name,
            version,
            stored.cumulative_source(),
            mappings,
            window_starts,
        );
        Ok(Self { value, stored })
    }

    #[inline(always)]
    pub fn push_block(&mut self, value: C) {
        self.stored.push_block(value);
    }
}
