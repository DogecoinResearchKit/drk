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

#[cfg(all(test, feature = "dogecoin"))]
mod tests {
    use std::{fs, process};

    use brk_types::{Sats, SatsTotal};
    use vecdb::{AnyStoredVec, Budgeted, ReadableVec};

    use super::*;

    const PART: u64 = u64::MAX / 2;
    const BLOCKS: usize = 1_000;

    fn exact(blocks: usize) -> f64 {
        (blocks as u128 * u128::from(PART)) as f64
    }

    /// Past `u64::MAX` within a few blocks: each total is the exact `u128` sum rounded once, and
    /// a reopen resumes from the stored total, one rounding off at most.
    #[test]
    fn sums_past_u64_exactly_and_resumes_from_the_stored_total() {
        let _ = Budgeted::init_global(1 << 20);
        let dir = std::env::temp_dir().join(format!("bitview-vecs-total-{}", process::id()));
        let _ = fs::remove_dir_all(&dir);
        let open = || {
            let db = Database::open(&dir).unwrap();
            let source = TotalSource::<SatsTotal>::import(&db, "total", Version::ZERO).unwrap();
            (db, source)
        };

        let (db, mut source) = open();
        for _ in 0..BLOCKS / 2 {
            source.push_block(Sats::new(PART));
        }
        let totals = source.cumulative_source().collect();
        for (i, total) in totals.into_iter().enumerate() {
            assert_eq!(f64::from(total), exact(i + 1), "block {i}");
        }
        source.stored.write().unwrap();
        db.flush();
        drop((source, db));

        let (_db, mut source) = open();
        assert_eq!(source.cumulative_source().len(), BLOCKS / 2);
        for _ in 0..BLOCKS / 2 {
            source.push_block(Sats::new(PART));
        }
        let last = f64::from(source.cumulative_source().collect_last().unwrap());
        let expected = exact(BLOCKS);
        assert!(expected > u64::MAX as f64);
        assert!(
            (last - expected).abs() <= expected * f64::EPSILON,
            "{last} vs {expected}"
        );
        drop((source, _db));
        fs::remove_dir_all(&dir).unwrap();
    }
}
