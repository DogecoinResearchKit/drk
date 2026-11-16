# Dogecoin delta vs upstream Bitview

Upstream (bitcoinresearchkit/brk, v0.13+) is the Bitview plugin
architecture. The pre-Bitview Dogecoin fork is `feat/dogecoin-scaffold`
(commits `b78cf6f87`, `389d4d5b5`, `73b60fb92`, `d0e1539c3`); its work was
re-ported onto Bitview on `rebuild/bitview`.

## Dogecoin facts

- Blk magic `c0c0c0c0`; RPC port 22555; datadir `~/.dogecoin`
  (macOS `~/Library/Application Support/Dogecoin`).
- 60-second blocks (1,440 a day). Genesis 2013-12-06 (unix 1386325540), 88 DOGE.
- Rewards: random before 145,000 (Dogeview reports the cap,
  `(1,000,000 >> halvings) - 1` DOGE, and no unclaimed reward); then
  `500,000 >> halvings`; halving every 100,000 blocks through 600,000
  (2015-02-25); a fixed 10,000 DOGE since. No supply cap.
- Difficulty: DigiShield, per block, since 145,000. Dogeview's
  `BLOCKS_PER_DIFF_EPOCH` is a 240-block reporting window (keeps the index u16).
- AuxPoW (merged mining with Litecoin) from 371,337: header version bit
  `1 << 8`, an `AuxPow` payload follows the 80-byte header.
- No BIP30 duplicate coinbases, no BIP54, no SegWit (pre-SegWit RPC: `getblocktemplate`
  txs lack `hash` / `weight`).
- Addresses: P2PKH `D…`, P2SH `9…` / `A…` (`bitcoin::dogecoin::Address`).

## Port map (old DRK -> Bitview)

| Old DRK | Bitview home |
|---|---|
| `brk_chain` alias (`bitcoin` / `bitcoin-dogecoin`) | `crates/brk_chain` (`dogecoin` feature) |
| Chain constants (blocks/day, RPC port, datadir, magic) | `brk_types/src/{block_targets,height}.rs`, `brk_rpc/src/connect_args.rs`, `brk_reader/src/scan.rs` |
| AuxPoW header parsing | `brk_types/src/dogecoin.rs` (`skip_auxpow`), `brk_reader/src/parse.rs` (+ tests), `bitview_query/src/impl/block/info.rs` |
| RPC 1.0 / pre-SegWit tolerance | `brk_rpc/src/rpc_client/{responses,methods,block_template}.rs` |
| Subsidy, genesis, unclaimed, hashrate | `brk_types/src/height.rs`, `bitview_plugin_supply`, `bitview_plugin_mining`, `bitview_vecs/.../difficulty_to_hashrate.rs` |
| Sub-cent cents | `brk_types/src/cents.rs` `CENTS_PER_DOLLAR` (1,000,000 under dogecoin) |
| CryptoCompare prices | `bitview_plugin_price/src/exchange.rs` (CryptoCompare + Bitfinex + Coinbase) |
| Fenwick significant-digit buckets | upstream already: `bitview_distribution/src/state/cost_basis/price_index.rs` |
| Pools (LTC list placeholder) | `bitview_primitives/pools-doge-v2.json`, `pool_slug.rs` slots 172-190 |
| dogeview rebrand, gold, formatters | `crates/brk_relabel`, `bitview_website/build.rs` + `dogecoin/` overlay |

## Where each decision is enforced (re-check on every merge)

- AuxPoW: `brk_reader/src/parse.rs` (after `HEADER_LEN`), `brk_types/src/read_block.rs`
  `pre_tx_len`, `bitview_plugin_indexer/src/processor/block/mod.rs` overhead,
  `bitview_query/src/impl/block/info.rs` `aux_pow_len`.
- Indexer: `bitview_plugin_indexer/src/constants.rs` (`VERSION + 2_000`,
  empty `DUPLICATE_COINBASES`).
- Supply and mining: `bitview_plugin_supply/src/{compute,import}.rs`
  (`genesis_reward`), `bitview_plugin_mining/src/rewards/compute.rs`
  (clamped `unclaimed_rewards`), `bitview_plugin_bedrock` (`FLOOR_SUPPLY_START`
  1,050,000), `bitview_cohort/src/halving_epoch.rs` (eras 0-5).
- Max supply: `brk_types::MAX_SUPPLY_SATS` in `bitview_query/src/impl/urpd/entries.rs`
  and `brk_rpc` block-template audit (`max_money`).
- Addresses: `brk_types/src/chain_address.rs`, used by `addr.rs`, `addr_bytes.rs`,
  `blk/src/fields.rs`, query block info.
- Price: `bitview_plugin_price/src/{compute,exchange,lib}.rs` (store version
  offset), `brk_oracle/src/{scale,seed}.rs` (scale-aware `cents_to_bin`,
  `FIRST_PRICE_HEIGHT = 0`), `bitview_query/src/impl/oracle.rs` (refusals,
  `live_price` = tip close).
- Difficulty API: `bitview_query/src/impl/mining/difficulty.rs`
  (`TARGET_BLOCK_SECONDS`, `QUARTER_EPOCH = 59`).
- Display: `brk_relabel` hooked in `bitview_query/src/vecs.rs`,
  `bitview_server/src/api/openapi/mod.rs`, `bitview_bindgen/src/constants.rs`,
  `bitview_devtools/src/bin/bindgen.rs`, `bitview_mcp/src/{main,page,server,logo}.rs`,
  `bitview_website/{build.rs,src/website.rs}`.

## Runtime

- `CRYPTOCOMPARE_API_KEY` gives price history from 2014; without it, prices
  before Bitfinex / Coinbase listings (2021) carry their earliest close.
- Public site name `dogeview.space`; repository `IndigoNakamoto/drk`.
