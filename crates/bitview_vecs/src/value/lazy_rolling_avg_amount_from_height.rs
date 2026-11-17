use bitview_primitives::{CentsFract, SatsFract};
use brk_types::{Bitcoin, CentsTotal, Dollars, SatsTotal};

use crate::{
    LazyPerBlock, LazyRollingAvgFromHeight, PricedAvgUsd, PricedRollingAvgFromHeight, Value,
};

pub type LazyRollingAvgAmountFromHeight = Value<
    LazyRollingAvgFromHeight<SatsTotal>,
    PricedRollingAvgFromHeight<CentsTotal>,
    LazyPerBlock<Bitcoin, SatsFract>,
    PricedAvgUsd<LazyPerBlock<Dollars, CentsFract>>,
>;
