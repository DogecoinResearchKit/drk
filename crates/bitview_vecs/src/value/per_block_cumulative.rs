use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{
    Cents, CentsTotal, Height, Sats, SatsTotal, TOTAL_VERSION, Total, Version,
};
use vecdb::{AnyVec, ComputeVec, Database, ReadableVec, Rw, StorageMode, VecValue, WritableVec};

use crate::{CumulativeState, IndexSources, LazyValueBlock, ValueTotalPerBlock};

#[derive(Traversable)]
pub struct ValuePerBlockCumulative<M: StorageMode = Rw> {
    /// Value for the represented block.
    pub block: LazyValueBlock,
    /// Cumulative value through the represented block. At time-period indexes,
    /// the value is taken at the period's final block.
    pub cumulative: ValueTotalPerBlock<M>,
    last_sats: M::WriteOnly<CumulativeState<<SatsTotal as Total>::Acc>>,
    last_cents: M::WriteOnly<CumulativeState<<CentsTotal as Total>::Acc>>,
}

const VERSION: Version = Version::TWO;

impl ValuePerBlockCumulative {
    pub(crate) fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let v = version + VERSION + TOTAL_VERSION;
        let cumulative =
            ValueTotalPerBlock::import(db, &format!("{name}_cumulative"), v, indexes)?;
        let block = LazyValueBlock::from_cumulative_sources(
            name,
            v,
            cumulative.sats.resolutions.height_source(),
            cumulative.cents.resolutions.height_source(),
        );

        Ok(Self {
            block,
            cumulative,
            last_sats: Default::default(),
            last_cents: Default::default(),
        })
    }

    /// Appends one block's sats; fiat values compute apart.
    pub fn push_sats(&mut self, value: Sats) {
        let sats = &mut self.cumulative.sats.height;
        let total = self
            .last_sats
            .add_total_at(Height::from(sats.len()), &*sats, value);
        sats.push(total);
    }

    pub fn compute_from<S>(
        &mut self,
        max_from: Height,
        price_cents: &impl ReadableVec<Height, Cents>,
        source: &impl ReadableVec<Height, S>,
        mut transform: impl FnMut(Height, S) -> Sats,
        exit: &Exit,
    ) -> Result<()>
    where
        S: VecValue,
    {
        let last = &mut self.last_sats;
        self.cumulative.sats.height.compute_transform(
            max_from,
            source,
            |(height, value, this)| {
                let value = transform(height, value);
                last.add_total_at(height, this, value)
            },
            exit,
        )?;
        self.compute_cents(max_from, price_cents, exit)
    }

    pub fn compute_from_pair<S1, S2>(
        &mut self,
        max_from: Height,
        price_cents: &impl ReadableVec<Height, Cents>,
        source1: &impl ReadableVec<Height, S1>,
        source2: &impl ReadableVec<Height, S2>,
        mut transform: impl FnMut(Height, S1, S2) -> Sats,
        exit: &Exit,
    ) -> Result<()>
    where
        S1: VecValue,
        S2: VecValue,
    {
        let last = &mut self.last_sats;
        self.cumulative.sats.height.compute_transform2(
            max_from,
            source1,
            source2,
            |(height, value1, value2, this)| {
                let value = transform(height, value1, value2);
                last.add_total_at(height, this, value)
            },
            exit,
        )?;
        self.compute_cents(max_from, price_cents, exit)
    }

    pub fn compute_cents(
        &mut self,
        max_from: Height,
        price_cents: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        let last = &mut self.last_cents;
        Ok(self.cumulative.cents.height.compute_transform2(
            max_from,
            &self.block.sats,
            price_cents,
            |(height, sats, price, this)| last.add_total_at(height, this, sats.value_at(price)),
            exit,
        )?)
    }
}
