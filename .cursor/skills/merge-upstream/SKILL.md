---
name: merge-upstream
description: >-
  Merge or pull bitcoinresearchkit/brk (Bitview) into this Dogecoin fork
  (drk / Dogeview) and re-check every Dogecoin seam. Use when the user says
  merge upstream, pull upstream, sync with BRK / Bitview, rebase on
  bitcoinresearchkit, or update from upstream main.
---

# Merge upstream Bitview into Dogecoin

This repo is a Dogecoin fork of [bitcoinresearchkit/brk](https://github.com/bitcoinresearchkit/brk),
the Bitview plugin architecture (`bitview_plugin_*` crates composed in
`bitview_default`, run by `bitviewd`). Everything Dogecoin-specific sits behind
the `dogecoin` Cargo feature, so the Bitcoin build must keep compiling. The
Litecoin sibling (LitecoinResearchKit/lrk) solves the same seams; its diff is
the template when upstream moves one.

Read [reference.md](reference.md) for the Dogecoin facts, the port map and
where each decision is enforced.

## When invoked

Copy this checklist and work it in order. Do not skip bindgen or the check script.

```
Merge progress:
- [ ] Fetch upstream, merge on a new branch
- [ ] Re-run scripts/dogecoin/propagate_feature.py
- [ ] Both builds green (dogecoin and bitcoin)
- [ ] Re-check every "enforced at" location in reference.md
- [ ] Website facts still found (the dogecoin build fails if not)
- [ ] Bindgen WITH dogecoin
- [ ] Run scripts/check.sh
- [ ] Smoke APIs against Dogecoin Core before swapping any live binary
```

## 1. Fetch and merge

```bash
git remote add upstream-brk https://github.com/bitcoinresearchkit/brk.git 2>/dev/null || true
git fetch upstream-brk
git switch -c merge/upstream-$(date +%Y%m%d)
git merge upstream-brk/main
```

Resolve conflicts keeping upstream's structure; re-apply Dogecoin changes
behind `#[cfg(feature = "dogecoin")]`.

**Never**

- `git push --force` to `main`
- Delete a live data dir to "fix" the merge
- Run bindgen without `--features dogecoin`

## 2. Feature graph and builds

New upstream crates need the `dogecoin` feature forwarded:

```bash
python3 scripts/dogecoin/propagate_feature.py
cargo check --workspace --all-targets --features dogecoin
cargo check --workspace --all-targets
```

Re-run the script whenever a crate gains cfg code or a dependency.

## 3. Re-check the Dogecoin seams

Upstream refactors move code: walk "Where each decision is enforced" in
reference.md and confirm each still applies. The usual breakages:

- a new block or header reader that seeks past 80 bytes without
  `brk_types::dogecoin::skip_auxpow` (every block from 371,337 carries AuxPoW);
- a new constant assuming 144 blocks/day, 600-second blocks, 2,016-block
  epochs, 210,000-block halvings, 21M supply or Bitcoin's RPC port (use
  `brk_types::{TARGET_BLOCKS_PER_DAY, TARGET_BLOCK_SECONDS, BLOCKS_PER_DIFF_EPOCH,
  BLOCKS_PER_HALVING, HALVING_CYCLE_BLOCKS, MAX_SUPPLY_SATS}`, `brk_rpc::DEFAULT_RPC_PORT`);
- a new `* 100` / `/ 100.0` between cents and dollars (use
  `brk_types::CENTS_PER_DOLLAR[_F64]`: a Dogecoin `Cents` is a micro-dollar);
- `bitcoin::Address` / `Network::Bitcoin` formatting an address (use
  `brk_types::{ChainAddress, ChainNetwork::MAINNET}`);
- a new oracle consumer (Dogecoin has none; price is exchange closes);
- an upstream pool id colliding with `pools-doge-v2.json` slots 172-190;
- page text that a `FACTS` entry in `crates/bitview_website/build.rs` anchors
  on: the dogecoin build panics naming the file and text; rewrite the entry.

Bump `bitview_plugin_indexer` `VERSION` (dogecoin offset) if indexed data
changed: a full reindex into a fresh data dir is then expected.

## 4. Regenerate clients as Dogecoin

```bash
rustup component add rustfmt   # bindgen formats the Rust client
cargo bindgen --features dogecoin
```

Do not hand-edit generated clients; regenerate. Display text comes from
`crates/brk_relabel`: fix a wrong word there (or in the website `FACTS`), not
in the output.

If upstream changed an icon, re-run `scripts/dogecoin/gold_icons.py` (needs
Pillow, see its docstring).

## 5. Run the check script

```bash
.cursor/skills/merge-upstream/scripts/check.sh
```

Fix anything it prints, then re-run until exit 0.

## 6. Smoke before live swap

Build and run against a separate data dir, with Dogecoin Core synced
(`server=1`, RPC on 22555, blk files from genesis):

```bash
cargo build --release -p bitviewd --features dogecoin
curl -sf http://127.0.0.1:7070/health
curl -sf http://127.0.0.1:7070/api/block-height/371337           # first AuxPoW block
curl -sf http://127.0.0.1:7070/api/series/price/height | tail -c 200   # sub-cent dollars
curl -sf http://127.0.0.1:7070/api/v1/difficulty-adjustment
```

## Conflict rules

- **Headers:** AuxPoW skip stays in `brk_reader` parse and the query's block info.
- **Price:** no on-chain oracle; `bitview_plugin_price` reads exchange closes
  under dogecoin and the oracle endpoints refuse. `Cents` stays micro-dollars.
- **Supply:** Dogecoin has no cap; `MAX_SUPPLY_SATS` is `u64::MAX`. Koinu
  supply nears `u64::MAX` around 2032: watch upstream's sat arithmetic.
- **VERSION bumps:** say so; compute will replay. Do not wipe a data dir unless the user asks.

## Afterward

Summarize: what upstream brought, which Dogecoin seams needed re-porting,
bindgen result, check.sh output, and whether a live swap is still pending.
