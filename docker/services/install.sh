#!/usr/bin/env bash
# Install dogeview LaunchAgents (dogecoin, native brk, cloudflared, watchdog).
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
AGENTS="$HOME/Library/LaunchAgents"
LOGS="$HOME/Library/Logs/dogeview"
UID_NUM="$(id -u)"
DOMAIN="gui/${UID_NUM}"

mkdir -p "$AGENTS" "$LOGS"
chmod +x "$DIR"/*.sh

labels=(
  com.dogeview.dogecoin
  com.dogeview.brk
  com.dogeview.cloudflared
  com.dogeview.watchdog
)

echo "Stopping Docker brk (native brk will own :7070 + ~/.brk)..."
docker update --restart=no brk 2>/dev/null || true
docker compose -f "$DIR/../docker-compose.yml" stop brk 2>/dev/null || true

echo "Stopping manual cloudflared (if any)..."
pkill -f 'cloudflared tunnel --config' 2>/dev/null || true
sleep 1

# Plists may hardcode another checkout path (e.g. Projects/drk); rewrite to this repo.
REPO_ROOT="$(cd "$DIR/../.." && pwd)"

echo "Installing plists (repo root: $REPO_ROOT)..."
for label in "${labels[@]}"; do
  src="$DIR/${label}.plist"
  dst="$AGENTS/${label}.plist"
  # BSD sed: plain substitutions (no -E) so both checkout paths rewrite cleanly.
  sed -e "s|/Users/indigo/Projects/drk|${REPO_ROOT}|g" \
      -e "s|/Users/indigo/Dev/drk|${REPO_ROOT}|g" \
      -e "s|/Users/indigo/Projects/lrk|${REPO_ROOT}|g" \
      -e "s|/Users/indigo/Dev/lrk|${REPO_ROOT}|g" \
      "$src" >"$dst"
  launchctl bootout "$DOMAIN/$label" 2>/dev/null || true
  launchctl bootstrap "$DOMAIN" "$dst"
  launchctl enable "$DOMAIN/$label" 2>/dev/null || true
  # Do not kickstart -k here — KeepAlive jobs block kickstart until exit.
  launchctl start "$label" 2>/dev/null || true
  echo "  loaded $label"
done

echo
echo "Done. Logs: $LOGS"
echo "  launchctl print $DOMAIN/com.dogeview.brk | head"
echo "  tail -f $LOGS/brk.out.log"
echo
echo "Note: leave Docker Desktop brk stopped. Index resumes from ~/.brk."
