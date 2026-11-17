use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, TOTAL_VERSION, Total, Version};
use vecdb::{AnyVec, Database, PcoVecValue, Rw, StorageMode, WritableVec};

use crate::{CachedSeries, CumulativeState, import_cached};

/// A running total of an amount stored as a cumulative [`Total`] series, summed exactly; block and
/// window views read it.
#[derive(Traversable)]
pub struct TotalSource<T: PcoVecValue + Total, M: StorageMode = Rw> {
    #[traversable(flatten)]
    stored: CachedSeries<Height, T, M>,
    last: M::WriteOnly<CumulativeState<T::Acc>>,
}

impl<T: PcoVecValue + Total> TotalSource<T> {
    pub fn import(db: &Database, name: &str, version: Version) -> Result<Self> {
        Ok(Self {
            stored: import_cached(db, name, version + TOTAL_VERSION)?,
            last: Default::default(),
        })
    }

    pub fn cumulative_source(&self) -> &CachedSeries<Height, T> {
        &self.stored
    }

    #[inline(always)]
    pub fn push_block(&mut self, part: T::Part) {
        let height = Height::from(self.stored.len());
        let total = self.last.add_total_at(height, &self.stored, part);
        self.stored.push(total);
    }
}
