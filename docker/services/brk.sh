#!/usr/bin/env bash
# Run native BRK (dogecoin-featured) on :7070 for the Cloudflare tunnel.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BRK_BIN="${BRK_BIN:-$ROOT/target/release/brk}"
DATADIR="${DOGECOIN_DATADIR:-$HOME/Library/Application Support/Dogecoin}"
BRKDIR="${BRK_DATA_DIR:-$HOME/.brk}"
PORT="${BRK_PORT:-7070}"

if [[ ! -x "$BRK_BIN" ]]; then
  echo "brk: missing binary at $BRK_BIN (build with: cargo build --release -p brk_cli --features dogecoin)" >&2
  exit 1
fi

# Prefer cookie when present; otherwise ~/.brk/config.toml rpcuser/rpcpassword.
# Force IPv4 — Dogecoin listens on *:22555 (v4); localhost can resolve to ::1.
exec "$BRK_BIN" \
  --chain dogecoin \
  --brkport "$PORT" \
  --rpcconnect 127.0.0.1 \
  --bitcoindir "$DATADIR" \
  --blocksdir "$DATADIR/blocks" \
  --brkdir "$BRKDIR"
