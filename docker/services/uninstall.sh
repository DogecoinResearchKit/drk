#!/usr/bin/env bash
set -euo pipefail

AGENTS="$HOME/Library/LaunchAgents"
UID_NUM="$(id -u)"
DOMAIN="gui/${UID_NUM}"

labels=(
  com.dogeview.watchdog
  com.dogeview.cloudflared
  com.dogeview.brk
  com.dogeview.dogecoin
)

for label in "${labels[@]}"; do
  launchctl bootout "$DOMAIN/$label" 2>/dev/null || true
  rm -f "$AGENTS/${label}.plist"
  echo "unloaded $label"
done
