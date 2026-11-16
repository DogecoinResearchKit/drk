#!/usr/bin/env bash
# Non-destructive Dogecoin-merge gate for the Bitview layout. Run from the repo root.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"

fail=0
say() { printf '%s\n' "$*"; }
bad() { say "FAIL: $*"; fail=1; }
ok() { say "OK: $*"; }
# check <file> <pattern> <ok message> <fail message>
check() {
  if grep -q -- "$2" "$1" 2>/dev/null; then ok "$3"; else bad "$4"; fi
}

check crates/brk_reader/src/parse.rs 'skip_auxpow' "reader skips AuxPoW" \
  "crates/brk_reader/src/parse.rs lost the AuxPoW skip (blocks from 371,337 misparse)"
check crates/bitview_query/src/impl/block/info.rs 'skip_auxpow' "block info skips AuxPoW" \
  "bitview_query block info lost the AuxPoW skip"
check crates/brk_reader/src/scan.rs 'feature = "dogecoin"' "Dogecoin blk magic present" \
  "crates/brk_reader/src/scan.rs lost the Dogecoin magic"
check crates/brk_types/src/cents.rs 'CENTS_PER_DOLLAR: u64 = 1_000_000' "micro-dollar cents present" \
  "crates/brk_types/src/cents.rs lost Dogecoin's CENTS_PER_DOLLAR"
check crates/brk_oracle/src/scale.rs 'CENTS_PER_DOLLAR_F64' "oracle bins are scale-aware" \
  "brk_oracle cents_to_bin divides by 100 again"
check crates/bitview_plugin_price/src/lib.rs 'mod exchange' "exchange-close prices present" \
  "bitview_plugin_price lost the Dogecoin exchange module"
check crates/bitview_plugin_indexer/src/constants.rs 'feature = "dogecoin"' "indexer Dogecoin VERSION/BIP30 present" \
  "indexer constants lost the dogecoin VERSION offset or empty DUPLICATE_COINBASES"
check crates/brk_types/src/height.rs 'DOGECOIN_SIMPLIFIED_REWARDS_HEIGHT' "Dogecoin subsidy schedule present" \
  "brk_types height.rs lost Dogecoin's subsidy schedule"
check crates/bitview_primitives/src/pools.rs 'pools-doge-v2.json' "Dogecoin pool list present" \
  "bitview_primitives pools.rs no longer loads pools-doge-v2.json"
check crates/bitview_query/src/vecs.rs 'brk_relabel' "series descriptions relabeled" \
  "bitview_query vecs.rs no longer relabels descriptions"

say ""
say "Checking both builds..."
cargo check -q --workspace --all-targets --features dogecoin && ok "dogecoin build" || bad "dogecoin build"
cargo check -q --workspace --all-targets && ok "bitcoin build" || bad "bitcoin build"

say ""
say "Checking Dogeview's pages for Bitcoin's words, dates and orange..."
cargo build -q -p bitview_website --features dogecoin
OUT="$(ls -td "$CARGO_TARGET_DIR"/debug/build/bitview_website-*/out/showcases | head -1)"
# Kept on purpose: pool names, unit keys (`Bitcoin:`), the day1 index's 2009 origin, Logs' ten-minute
# window, and Studio's palette orange.
leftovers="$(cd "$OUT" && rg -n -o '.{0,40}(\bBitcoin\b|\bBTC\b|bitview\.dev|bitcoind|bitcoin-cli|bitcoinresearchkit\.org|\b2009\b|\b2010\b|210,000|210_000|ten minutes|0\.191 44\.41|#f7931a|rgb\(230 91 0).{0,40}' \
  --glob '!modules/**' --glob '!changes/**' --glob '!assets/pools/**' --glob '!studio/sitemap.xml' \
  --glob '!studio/palette.js' --glob '!logs/**' \
  | grep -v -F -e 'Bitcoin.com' -e 'BTC.' -e 'BTC Guild' -e 'Bitcoin:' -e 'Jan 2009' \
    -e 'Date.UTC(2009, 0, 1)' -e '2009-l' -e '1 January 2009, the genesis' || true)"
if [[ -n "$leftovers" ]]; then
  bad "Bitcoin leftovers in the dogecoin showcases (add a FACTS entry in crates/bitview_website/build.rs or a rule in brk_relabel):"
  say "$leftovers"
else
  ok "no Bitcoin leftovers in the dogecoin showcases"
fi

say ""
say "Running dogecoin tests..."
if cargo test -q --features dogecoin \
  -p brk_types -p brk_rpc -p brk_reader -p brk_relabel -p bitview_primitives \
  -p bitview_distribution -p bitview_plugin_price; then
  ok "cargo test --features dogecoin"
else
  bad "cargo test --features dogecoin"
fi

say ""
if [[ "$fail" -ne 0 ]]; then
  say "merge-upstream check FAILED"
  exit 1
fi
say "merge-upstream check passed"
exit 0
