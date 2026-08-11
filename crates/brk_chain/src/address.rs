//! Chain-appropriate address types.
//!
//! [`primitives`](crate::primitives) swaps the whole primitives crate per Cargo
//! feature, which keeps every existing `bitcoin::Foo` reference compiling
//! unmodified. It does *not* swap `Network` **variants**, and the two upstream
//! forks disagree about what those variants mean:
//!
//! * `litecoin` patched the address prefixes in place and dispatches on
//!   `NetworkKind::Main`. There `Network::Bitcoin` reads as "mainnet", and
//!   mainnet is Litecoin, so `L...` comes out correct.
//! * `bitcoin-dogecoin` left the inherited rust-bitcoin namespace intact and
//!   added Dogecoin as a sibling `dogecoin` module carrying its own `Address`,
//!   `Network`, and prefix constants. Root `Network::Bitcoin` still means
//!   Bitcoin and still encodes version byte `0x00`, i.e. `1...`.
//!
//! So naming `Network::Bitcoin` at a call site is correct on two chains and
//! silently emits **valid Bitcoin addresses** on the third — no compile error,
//! no type mismatch, nothing to notice. This module removes the opportunity:
//! call sites use [`Address`] and [`MAINNET`] and never name a variant.
//!
//! Note that [`Address`] is a *different type* per chain, not just a different
//! value. On Dogecoin it is `primitives::dogecoin::Address`, which encodes only
//! P2PKH (`0x1e`) and P2SH (`0x16`) — Dogecoin has no segwit, so
//! `Address::from_script` rejects witness programs instead of inventing a
//! bech32 string for them. Callers must handle that error rather than
//! unwrapping it.

// Litecoin takes precedence when both features are somehow enabled, matching
// the convention in `lib.rs`.
#[cfg(all(feature = "dogecoin", not(feature = "litecoin")))]
mod active {
    pub use crate::primitives::dogecoin::{Address, Network};

    /// Mainnet of the active chain.
    pub const MAINNET: Network = Network::Dogecoin;

    /// Parse a `getblockchaininfo.chain` value into the active chain's network.
    ///
    /// `dogecoin::Network` implements `FromStr` over its own display names
    /// (`"dogecoin"`, `"testnet"`, `"regtest"`) rather than Core's chain
    /// argument, and has no `from_core_arg`, so the mapping is spelled out.
    pub fn network_from_core_arg(core_arg: &str) -> Option<Network> {
        match core_arg {
            "main" => Some(Network::Dogecoin),
            "test" => Some(Network::Testnet),
            "regtest" => Some(Network::Regtest),
            _ => None,
        }
    }
}

#[cfg(not(all(feature = "dogecoin", not(feature = "litecoin"))))]
mod active {
    pub use crate::primitives::{Address, Network};

    /// Mainnet of the active chain.
    pub const MAINNET: Network = Network::Bitcoin;

    /// Parse a `getblockchaininfo.chain` value into the active chain's network.
    pub fn network_from_core_arg(core_arg: &str) -> Option<Network> {
        Network::from_core_arg(core_arg).ok()
    }
}

pub use active::{Address, MAINNET, Network, network_from_core_arg};
