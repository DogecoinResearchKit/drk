mod fenwick;
mod groups;
mod percentiles;
mod receive;
mod send;
mod tick_tock;
mod vecs;

/// Rounding precision for UTXO cost basis prices, in significant digits. Applied
/// via `round_to` (scale-invariant) rather than `round_to_dollar`, which would
/// collapse every sub-dollar price to $0 on the Dogecoin micro-dollar scale.
const COST_BASIS_PRICE_DIGITS: i32 = 5;

pub use groups::*;
