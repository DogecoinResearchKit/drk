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
  `1,000,000 >> halvings` DOGE, and no unclaimed reward); then
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
  `bitview_query/src/impl/block/info.rs` `aux_pow_len`, `bitview_query/src/impl/block/raw.rs`
  (Dogecoin `verify_raw_payload`, `brk_reader::MAGIC_BYTES`; fixtures in `brk_reader/fixtures`).
- Indexer: `bitview_plugin_indexer/src/constants.rs` (`VERSION + 2_000`,
  empty `DUPLICATE_COINBASES`, `RECHECKED_BLOCKS` = `TARGET_BLOCKS_PER_DAY`: 1,440 blocks, a day
  at one-minute blocks, where upstream's 144 is a day at ten).
- Totals: `brk_types/src/totals.rs` (`SatsTotal`, `CentsTotal`, `Total`, `TOTAL_VERSION`); see
  "u64 overflow".
- Pools: `bitview_primitives/pools-doge-v2.json`, matched against each block's own (Dogecoin)
  coinbase tag and payout addresses; a merge-mined block's AuxPoW parent coinbase is not read.
  A block matching no pool stays `Unknown`: there is no guessing from the parent chain.
- `blk` CLI: `blk/src/fields.rs` takes `size`, `weight` and `hex` from `ReadBlock::bytes`
  (AuxPoW included); the re-serialized `bitcoin::Block` drops it.
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
- RPC: `brk_rpc/src/rpc_client/{responses,mempool_entry}.rs` (see "Dogecoin Core 1.14 RPC").
- Display: `brk_relabel` hooked in `bitview_query/src/vecs.rs`,
  `bitview_server/src/api/openapi/mod.rs`, `bitview_bindgen/src/constants.rs`,
  `bitview_devtools/src/bin/bindgen.rs`, `bitview_mcp/src/{main,page,server,logo}.rs`,
  `bitview_website/{build.rs,src/website.rs}`, and bindgen's `relabel_prose` over the catalog's
  type schemas. `bitview_server`'s `served_spec_states_dogecoin` test fails when a relabel fact
  stops matching upstream's API text.

## Runtime

- `CRYPTOCOMPARE_API_KEY` gives price history from 2014; without it, prices
  before Bitfinex / Coinbase listings (2021) carry their earliest close.
- Dogecoin Core 1.14.9 at `~/dogecoin-1.14.9` (`dogecoind`, `dogecoin-cli` on `PATH`), datadir
  `~/Library/Application Support/Dogecoin`, `txindex=1`. The local resolver hides Dogecoin's DNS
  seeds (`seed.multidoge.org` and the rest answer nothing), so a fresh node finds no peers:
  resolve the seeds through public DNS (`dig @1.1.1.1 seed.multidoge.org +short`) and
  `dogecoin-cli addnode <ip>:22556 onetry` each address. Once connected it learns peers itself.
- Public site name `dogeview.space`; repository `IndigoNakamoto/drk`.

## Dogecoin Core 1.14 RPC

Dogecoin Core 1.14 answers as Bitcoin Core 0.14 did. Every call Dogeview makes (`brk_rpc`; `brk_mempool`
goes through it), and what differs:

| Call | Fields read | 1.14 |
|---|---|---|
| `getblockchaininfo` | `chain`, `blocks`, `headers` | same; `chain` is `main`, read as mainnet |
| `getblockcount`, `getbestblockhash`, `getblockhash` (batched) | the result | same |
| `getblockheader` (verbose) | `confirmations`, `height` | same (no `nTx`, not read) |
| `getrawtransaction txid false` | hex | same (0.14 takes a bool); mempool only without `-txindex` |
| `getblocktemplate {"rules":["segwit"]}` | `previousblockhash`, tx `data`, `txid`, `hash`, `depends`, `fee`, `sigops`, `weight` | no `hash`, `weight` (and maybe `sigops`): `fill_pre_segwit` sets wtxid = txid, weight = 4 x size; no `weightlimit` (not read) |
| `getrawmempool false` | txids | same |
| `getmempoolinfo` | `mempoolminfee` (DOGE/kB) | same |
| `getmempoolentry` | `vsize`, `weight`, `time`, `fees.base` | `size`, `fee` instead: read by `mempool_entry.rs`'s Dogecoin `Entry` |
| `sendrawtransaction hex` | txid | same |

Answers are JSON-RPC 1.0 (`"error": null`, no `jsonrpc`), which both clients accept; errors come
with HTTP 500. The reader needs no `xor.dat` (1.14 doesn't write one).

## u64 overflow (audit, Nov 2026)

`Sats` is a `u64` of koinu: about 184.5B DOGE. Release builds wrap on overflow (`overflow-checks =
false`); dev builds panic. Peaks at the current tip:

- **Past the limit (fixed by the totals seam below):** all-time transfer-volume cumulatives and
  their 1w/1m/1y windows. Dogecoin moves billions of DOGE a day, so these passed 184.5B DOGE
  within weeks of genesis: `bitview_plugin_transactions` `transfer_volume`, holders
  `activity.transfer_volume` (and in profit / loss), age `aged_out` and transfer volume in
  profit / loss, address balance bands, UTXO and creation cohorts (`bitview_distribution`
  `CumulativeValue`), and supply velocity, which reads the 1y sum. Unfixed, per-block values and
  windows (deltas of the stored total) read 0 at the block where the total wraps.
- **Sat-days destroyed (fixed):** `Age::satdays_destroyed` and the cohort accumulator are `u128`;
  one whale output passed `u64::MAX` sat-days in a few years.
- **Around 2031 (supply-bounded, ~162B DOGE now, +5.26B a year):** supply snapshots (holders,
  utxo set, address types and bands, bedrock, profit / loss supply) and pools' per-pool reward
  totals (`pool_heights` `Rewards`). Guarded, not widened: under `dogecoin`, `Sats` `+`, `+=` and
  `Sum` are checked and panic with "Dogecoin supply exceeded u64 koinu; widen stock series";
  pools' totals panic with "widen pool totals". **Deadline: widen these stock series before
  2030.** The mining `coinbase` / `subsidy` cumulatives already go through the totals seam.
- **Safe:** fees, OP_RETURN and other burns, per-block values, supply deltas.
- **Cents:** a cents value is a `u64` of micro-dollars, capped near $18.4T; `Cents` addition
  already panics on overflow. USD flow cumulatives (transfer volume, coinbase, subsidy: thermo
  cap) go through `CentsTotal`; realized profit / loss / value destroyed cumulatives stay `Cents`.

### The totals seam

vecdb's `Pco` storage stops at 64-bit values (`u64`, `f64`): there is no `u128` column. So every
multi-block sum of an amount (cumulatives and rolling windows) has the type `SatsTotal` /
`CentsTotal` (`brk_types/src/totals.rs`):

- Bitcoin: plain aliases of `Sats` / `Cents`. Upstream's code and data are unchanged.
- Dogecoin: `f64` newtypes holding koinu / micro-dollars (the `_sats` / `_cents` names keep their
  unit; the `btc` / `usd` views divide). The sum runs exactly in a `u128` (`Total::Acc`) and is
  rounded once per stored row; on reopen it resumes from the stored row, one rounding off at
  most (`total_source.rs` test).

Enforced at:

- `bitview_vecs`: `TotalSource` (beside `CumulativeSource`), `CumulativeState::add_total_at`,
  `ValuePerBlockCumulative` (`cumulative: ValueTotalPerBlock`), the lazy value views
  (`LazyValueBlock` takes the delta on the total and converts it back to `Sats` / `Cents`,
  `LazyPreviousDeltaVec<I, S, T>`), the rolling sum / average views, and `FiatType for CentsTotal`.
- Writers that sum by hand: `bitview_plugin_outputs/src/value/compute.rs`,
  `bitview_plugin_transactions/src/{fees,inscription}/compute.rs` (`collect_last().map(Total::acc)`,
  `SatsTotal::add`, `from_acc`).
- Readers: `bitview_distribution` `CumulativeValue`, `bitview_plugin_age` `aged_out_sources`,
  holders `Columns` / `AccountingTotals` / aSOPR, mining hash value and reward ratios, op_return fee
  share, coinflow, supply velocity, cointime thermo cap, pools coinbase, query reward stats.
- Versions: every retyped column adds `TOTAL_VERSION` (`ZERO` on Bitcoin, `ONE` on Dogecoin).

On a merge, a new upstream cumulative or window of `Sats` / `Cents` should take the total type;
the Dogecoin build fails to compile where a total meets a `Sats` reader. Bindgen's clients don't
change: the catalog serves the `btc` / `usd` views, still `Bitcoin` / `Dollars`.
