use brk_types::{Height, Total};
use vecdb::{ReadableVec, VecValue};

/// Writer-only running totals, reloaded from the source when the requested length differs from
/// the cached one.
pub struct CumulativeState<R> {
    last: Option<(usize, R)>,
}

impl<R> Default for CumulativeState<R> {
    fn default() -> Self {
        Self { last: None }
    }
}

impl<R: Clone + Default> CumulativeState<R> {
    pub fn accumulate(
        &mut self,
        len: usize,
        load: impl FnOnce() -> Option<R>,
        add: impl FnOnce(&mut R),
    ) -> R {
        let mut values = match self.last.take() {
            Some((cached_len, values)) if cached_len == len => values,
            _ => load().unwrap_or_default(),
        };
        add(&mut values);
        self.last = Some((len + 1, values.clone()));
        values
    }

    /// The running total at `height`, continuing the totals `stored` holds below it.
    #[inline]
    pub(crate) fn add_at(
        &mut self,
        height: Height,
        stored: &impl ReadableVec<Height, R>,
        add: impl FnOnce(&mut R),
    ) -> R
    where
        R: VecValue,
    {
        self.accumulate(
            usize::from(height),
            || {
                height
                    .decremented()
                    .and_then(|height| stored.collect_one(height))
            },
            add,
        )
    }

    /// The total at `height`: `part` added exactly to the running sum of the totals `stored`
    /// holds below it.
    #[inline]
    pub(crate) fn add_total_at<T>(
        &mut self,
        height: Height,
        stored: &impl ReadableVec<Height, T>,
        part: T::Part,
    ) -> T
    where
        T: Total<Acc = R> + VecValue,
    {
        let acc = self.accumulate(
            usize::from(height),
            || {
                height
                    .decremented()
                    .and_then(|height| stored.collect_one(height))
                    .map(T::acc)
            },
            |acc| T::add(acc, part),
        );
        T::from_acc(&acc)
    }
}
