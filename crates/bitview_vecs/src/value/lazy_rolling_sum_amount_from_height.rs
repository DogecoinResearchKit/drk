use brk_types::{Bitcoin, CentsTotal, Dollars, SatsTotal};

use crate::{
    LazyPerBlock, LazyRollingSumFromHeight, PricedRollingSumFromHeight, PricedSumUsd, Value,
};

pub type LazyRollingSumAmountFromHeight = Value<
    LazyRollingSumFromHeight<SatsTotal>,
    PricedRollingSumFromHeight<CentsTotal>,
    LazyPerBlock<Bitcoin, SatsTotal>,
    PricedSumUsd<LazyPerBlock<Dollars, CentsTotal>>,
>;
