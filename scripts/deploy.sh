#!/usr/bin/env bash
# CI/CD deploy script — build release binary and restart services.
# Called manually or by git post-merge hook.
#
# Usage: ./scripts/deploy.sh [--no-restart]

set -euo pipefail

SIGIL_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$SIGIL_ROOT"

echo "[deploy] Building release binary..."
cargo build --release -p sigil 2>&1 | tail -3

if [[ "${1:-}" == "--no-restart" ]]; then
    echo "[deploy] Build complete (restart skipped)."
    exit 0
fi

echo "[deploy] Restarting sigil-daemon..."
sudo systemctl restart sigil-daemon
sleep 3

echo "[deploy] Restarting sigil-web..."
sudo systemctl restart sigil-web
sleep 2

# Verify
DAEMON_STATUS=$(systemctl is-active sigil-daemon 2>/dev/null || echo "failed")
WEB_STATUS=$(systemctl is-active sigil-web 2>/dev/null || echo "failed")

echo "[deploy] daemon: $DAEMON_STATUS | web: $WEB_STATUS"

if [[ "$DAEMON_STATUS" == "active" && "$WEB_STATUS" == "active" ]]; then
    echo "[deploy] Deploy successful."

    # Reindex graph after deploy
    if command -v sigil &>/dev/null; then
        sigil graph index -r sigil 2>/dev/null &
        echo "[deploy] Graph reindex started in background."
    fi
else
    echo "[deploy] WARNING: One or more services failed to start!"
    exit 1
fi
