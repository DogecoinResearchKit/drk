# Dogecoin Research Kit

[![MIT Licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/IndigoNakamoto/drk/blob/main/docs/LICENSE.md)

> A Dogecoin fork of [Bitcoin Research Kit](https://github.com/bitcoinresearchkit/brk) (BRK), derived from the [Litecoin Research Kit](https://github.com/IndigoNakamoto/lrk) (LRK).

Open-source Dogecoin data toolkit that can parse blocks, index the chain, compute metrics, serve data, and render it — all from a Dogecoin Core node. It combines what on-chain analytics providers and block explorers do separately into a single self-hostable package.

## Status

DRK is an early-stage scaffold. It reuses LRK's `brk_chain` chain-abstraction layer to add a compile-time `dogecoin` target (network magic, RPC port, block time, data directories, and the `bitcoin-dogecoin` primitives crate). The consensus- and economics-specific porting work is still in progress:

- **AuxPoW parsing** — Dogecoin merge-mines against Litecoin, so from block 371,337 each block header is followed by a variable-length `CAuxPow` payload before the transaction count. The block reader must detect the AuxPoW version flag and consume this payload.
- **Subsidy schedule** — Dogecoin has no perpetual halving; the block reward follows a step-down schedule that settles at a fixed 10,000 DOGE from block 600,000 onward.
- **Time constants** — Dogecoin targets 1-minute blocks (1,440/day).
- **Overflow review** — Dogecoin's supply approaches `u64::MAX` in koinu; cumulative aggregates may need to move to `u128`.

See `docs/ARCHITECTURE.md` and the in-code `TODO(dogecoin)` markers for details.

## Data

**Zero external dependencies.** DRK needs only a Dogecoin Core node. Metrics are computed locally from your own copy of the blockchain.

**Blockchain:** Blocks, transactions, addresses, UTXOs.

**Metrics:** Supply distributions, holder cohorts, network activity, fee markets, mining, and market indicators.

**Indexes:** Date, height, address type, UTXO age.

**Mempool:** Fee estimation, projected blocks, unconfirmed transactions.

## Usage

### Self-host

DRK must be built with the `dogecoin` Cargo feature so the AuxPoW-aware block/transaction decoder is selected at compile time. A default (Bitcoin) build fails to parse merge-mined blocks.

```bash
git clone https://github.com/IndigoNakamoto/drk.git && cd drk
cargo install --locked --path crates/brk_cli --features dogecoin
brk --chain dogecoin
```

All you need is Dogecoin Core with `server=1`. Dogecoin mainnet RPC runs on port 22555.

### Library

```bash
cargo add brk --features dogecoin
```

Build custom applications in Rust. Use the full stack or individual components (parser, indexer, computer, server).

## Links

- [Changelog](./CHANGELOG.md)
- [Contributing](https://github.com/IndigoNakamoto/drk/issues)
- [Upstream LRK](https://github.com/IndigoNakamoto/lrk)
- [Upstream BRK](https://github.com/bitcoinresearchkit/brk)

## License

[MIT](./LICENSE.md)
