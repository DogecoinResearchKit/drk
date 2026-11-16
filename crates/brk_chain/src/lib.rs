//! The workspace depends on this crate under the name `bitcoin`
//! (`bitcoin = { package = "brk_chain", ... }`), so every `bitcoin::` path in
//! Bitview resolves to the `bitcoin` crate by default and to `bitcoin-dogecoin`
//! (a rust-bitcoin fork with AuxPoW headers) under the `dogecoin` feature.
//!
//! Cargo unifies features per build, so enabling `dogecoin` anywhere switches
//! every crate. Crates with Dogecoin-only code must therefore forward their own
//! `dogecoin` feature here and to every dependency that also has one.

#[cfg(not(feature = "dogecoin"))]
pub use bitcoin_crate::*;

#[cfg(feature = "dogecoin")]
pub use ::dogecoin::*;
