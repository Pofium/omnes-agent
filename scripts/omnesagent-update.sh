#!/usr/bin/env bash
# ==============================================================================
# OmnesAgent — Zero-Downtime Atomic Update Script
# Usage: sudo ./omnesagent-update.sh [--branch <branch>]
# ==============================================================================
set -euo pipefail

APP_DIR="/opt/omnesagent"
SOURCE_DIR="$APP_DIR/source"
BRANCH="main"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --branch)
            BRANCH="$2"
            shift 2
            ;;
        *)
            shift
            ;;
    esac
done

if [[ $EUID -ne 0 ]]; then
    echo "ERROR: This script must be run as root (use sudo)." >&2
    exit 1
fi

echo "=== OmnesAgent Update: Pulling latest source ($BRANCH) ==="
cd "$SOURCE_DIR"
git fetch --all
git checkout "$BRANCH"
git pull origin "$BRANCH"

GIT_SHA=$(git rev-parse --short HEAD)
RELEASE_DIR="$APP_DIR/releases/$GIT_SHA"
echo "Target release SHA: $GIT_SHA"

mkdir -p "$RELEASE_DIR"/{bin,web_dist}

echo "=== Building Backend release binary ==="
cd "$SOURCE_DIR/backend"
cargo build --release --bin omnesagent
cp target/release/omnesagent "$RELEASE_DIR/bin/"

echo "=== Building Frontend Web ADE ==="
export PATH="/opt/flutter/bin:$PATH"
cd "$SOURCE_DIR/frontend/web"
flutter build web --release --web-renderer canvaskit
cp -r build/web/* "$RELEASE_DIR/web_dist/"

echo "=== Performing Atomic Release Swap ==="
ln -sfn "$RELEASE_DIR" "$APP_DIR/releases/current"
cp "$RELEASE_DIR/bin/omnesagent" "$APP_DIR/bin/omnesagent"
rsync -a --delete "$RELEASE_DIR/web_dist/" "$APP_DIR/web_dist/"

# Preserve permissions
chown -R omnesagent:omnesagent "$APP_DIR"

echo "=== Restarting Services ==="
systemctl restart omnesagent
nginx -t && systemctl reload nginx

echo "=== Health Verification ==="
sleep 2
if systemctl is-active --quiet omnesagent.service && curl -sf http://127.0.0.1:42617/api/health >/dev/null; then
    echo "=========================================================================="
    echo "   ★ OmnesAgent updated successfully to release $GIT_SHA! ★"
    echo "=========================================================================="
else
    echo "ERROR: Health check failed after update!" >&2
    systemctl status omnesagent --no-pager
    exit 1
fi
EOF
