#!/usr/bin/env bash
# Keep Dogecoin Core alive for launchd. Qt -daemon forks; we wait on the pid
# so KeepAlive restarts after a crash (e.g. disk write failure).
set -euo pipefail

QT="${DOGECOIN_QT:-/Applications/Dogecoin-Qt.app/Contents/MacOS/Dogecoin-Qt}"
DATADIR="${DOGECOIN_DATADIR:-$HOME/Library/Application Support/Dogecoin}"
PIDFILE="${DOGECOIN_PIDFILE:-$DATADIR/dogecoind.pid}"

if [[ ! -x "$QT" ]]; then
  echo "dogecoin: missing binary at $QT" >&2
  exit 1
fi

mkdir -p "$DATADIR"

wait_pid() {
  local pid="$1"
  while kill -0 "$pid" 2>/dev/null; do
    sleep 10
  done
}

# Already running (e.g. previous GUI session) — adopt and wait.
if [[ -f "$PIDFILE" ]]; then
  existing="$(cat "$PIDFILE" 2>/dev/null || true)"
  if [[ -n "${existing:-}" ]] && kill -0 "$existing" 2>/dev/null; then
    echo "dogecoin: adopting pid $existing"
    wait_pid "$existing"
    echo "dogecoin: process $existing exited"
    exit 1
  fi
  rm -f "$PIDFILE"
fi

echo "dogecoin: starting daemon"
"$QT" -daemon -datadir="$DATADIR" -pid="$PIDFILE"

for _ in $(seq 1 60); do
  if [[ -f "$PIDFILE" ]]; then
    pid="$(cat "$PIDFILE")"
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      echo "dogecoin: running pid $pid"
      wait_pid "$pid"
      echo "dogecoin: process $pid exited"
      exit 1
    fi
  fi
  sleep 1
done

echo "dogecoin: failed to start (no pid file)" >&2
exit 1
