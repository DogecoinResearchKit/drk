//! Chain-agnostic primitives re-export and chain constants for BRK.
//!
//! Enable the `litecoin` Cargo feature (and disable the default `bitcoin`
//! feature) to compile BRK against the Litecoin network:
//!
//! ```toml
//! brk_chain = { ..., default-features = false, features = ["litecoin"] }
//! ```
//!
//! All downstream crates that depend on `brk_chain` then access protocol types
//! through `brk_chain::primitives`, which resolves to the `bitcoin`, `litecoin`,
//! or `dogecoin` (`bitcoin-dogecoin`) crate depending on the active feature.

mod chain;
pub use chain::{Chain, ChainConstants};

pub mod address;

// Re-export the active chain's primitive types under a stable module path so
// downstream crates can write `use brk_chain::primitives as bitcoin` and keep
// all existing `bitcoin::Foo` references working without modification.

#[cfg(all(feature = "bitcoin", not(feature = "litecoin"), not(feature = "dogecoin")))]
pub use ::bitcoin as primitives;

#[cfg(feature = "litecoin")]
pub use ::litecoin as primitives;

// Dogecoin uses the `bitcoin-dogecoin` crate (dfinity/rust-dogecoin), which is
// imported under the `dogecoin` package alias. Litecoin takes precedence when
// both features are somehow enabled, matching the existing bitcoin/litecoin
// convention above.
#[cfg(all(feature = "dogecoin", not(feature = "litecoin")))]
pub use ::dogecoin as primitives;
