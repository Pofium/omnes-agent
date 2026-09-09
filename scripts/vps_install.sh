#!/usr/bin/env bash
# ==============================================================================
# OmnesAgent — Full VPS Deployment & Installation Script
# Automated one-step deployment: Gateway daemon + Web ADE + Nginx + SSL + systemd
# ==============================================================================
set -euo pipefail

# Default parameters
DOMAIN=""
PORT=""
EMAIL=""
NO_SSL=false
DATA_DIR="/var/lib/omnesagent"
APP_DIR="/opt/omnesagent"
REPO_URL="https://github.com/Pofium/omnes-agent.git"
BRANCH="main"

print_usage() {
    cat <<EOF
Usage: sudo ./vps_install.sh [OPTIONS]

Options:
    --domain <name>      FQDN domain for SSL (e.g. agent.example.com)
    --port <N>           Public HTTP port (default: 80/443 for domain, 8443 for IP)
    --email <email>      Email address for Let's Encrypt SSL certificate
    --no-ssl             Disable Let's Encrypt SSL configuration
    --data-dir <path>    Path to persistent data directory (default: /var/lib/omnesagent)
    --branch <branch>    Git branch to deploy (default: main)
    -h, --help           Show this help message
EOF
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --domain)
            DOMAIN="$2"
            shift 2
            ;;
        --port)
            PORT="$2"
            shift 2
            ;;
        --email)
            EMAIL="$2"
            shift 2
            ;;
        --no-ssl)
            NO_SSL=true
            shift
            ;;
        --data-dir)
            DATA_DIR="$2"
            shift 2
            ;;
        --branch)
            BRANCH="$2"
            shift 2
            ;;
        -h|--help)
            print_usage
            ;;
        *)
            echo "Unknown option: $1"
            print_usage
            ;;
    esac
done

# Step 1: Privilege and environment verification
echo "=== [1/15] Verifying system privileges and environment ==="
if [[ $EUID -ne 0 ]]; then
    echo "ERROR: This script must be run as root (use sudo)." >&2
    exit 1
fi

if [[ -n "$DOMAIN" && "$NO_SSL" = false && -z "$EMAIL" ]]; then
    echo "ERROR: --email is required when configuring SSL with --domain." >&2
    exit 1
fi

# Step 2: System dependencies and packages
echo "=== [2/15] Installing system packages ==="
export DEBIAN_FRONTEND=noninteractive
apt-get update -y
apt-get install -y --no-install-recommends \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    nginx \
    ufw \
    ca-certificates \
    rsync \
    jq

if [[ -n "$DOMAIN" && "$NO_SSL" = false ]]; then
    apt-get install -y --no-install-recommends certbot python3-certbot-nginx
fi

# Step 3: Swap configuration if RAM < 2GB
TOTAL_RAM_MB=$(free -m | awk '/^Mem:/{print $2}')
if [[ $TOTAL_RAM_MB -lt 2000 ]]; then
    if ! grep -q '/swapfile' /proc/swaps; then
        echo "=== [3/15] RAM is ${TOTAL_RAM_MB}MB (<2000MB). Creating 2GB swap space for compilation ==="
        fallocate -l 2G /swapfile || dd if=/dev/zero of=/swapfile bs=1M count=2048
        chmod 600 /swapfile
        mkswap /swapfile
        swapon /swapfile
        if ! grep -q '/swapfile' /etc/fstab; then
            echo '/swapfile none swap sw 0 0' >> /etc/fstab
        fi
    else
        echo "=== [3/15] Swap space already active ==="
    fi
else
    echo "=== [3/15] Sufficient RAM detected (${TOTAL_RAM_MB}MB) ==="
fi

# Step 4: Rust Toolchain Setup
echo "=== [4/15] Verifying Rust toolchain ==="
if ! command -v cargo &>/dev/null; then
    echo "Installing Rust toolchain via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
else
    echo "Rust is already installed: $(cargo --version)"
fi

# Step 5: Flutter SDK Setup (for web frontend build)
echo "=== [5/15] Verifying Flutter SDK for Web ADE ==="
export FLUTTER_ROOT="/opt/flutter"
export PATH="$FLUTTER_ROOT/bin:$PATH"

if ! command -v flutter &>/dev/null; then
    echo "Installing Flutter SDK to /opt/flutter..."
    git clone https://github.com/flutter/flutter.git -b stable "$FLUTTER_ROOT"
    git config --global --add safe.directory "$FLUTTER_ROOT"
    flutter precache --web
    flutter config --no-analytics
else
    echo "Flutter is already installed: $(flutter --version | head -n 1)"
fi

# Step 6: System User and Directory Structure
echo "=== [6/15] Setting up system user and directories ==="
if ! id -u omnesagent &>/dev/null; then
    useradd -r -s /bin/bash -m -d "$DATA_DIR" omnesagent
fi

mkdir -p "$APP_DIR"/{bin,web_dist,releases,source}
mkdir -p "$DATA_DIR"/data

# Step 7: Clone / Update repository
echo "=== [7/15] Fetching source code ==="
if [[ -d "$APP_DIR/source/.git" ]]; then
    cd "$APP_DIR/source"
    git fetch --all
    git checkout "$BRANCH"
    git pull origin "$BRANCH"
else
    git clone -b "$BRANCH" "$REPO_URL" "$APP_DIR/source"
    cd "$APP_DIR/source"
fi

GIT_SHA=$(git rev-parse --short HEAD)
RELEASE_DIR="$APP_DIR/releases/$GIT_SHA"
mkdir -p "$RELEASE_DIR"/{bin,web_dist}

# Step 8: Build backend (Rust release binary)
echo "=== [8/15] Compiling backend release binary ==="
cd "$APP_DIR/source/backend"
cargo build --release --bin omnesagent
cp target/release/omnesagent "$RELEASE_DIR/bin/"

# Step 9: Build frontend Web ADE
echo "=== [9/15] Building Frontend Web ADE ==="
cd "$APP_DIR/source/frontend/web"
flutter build web --release --web-renderer canvaskit
cp -r build/web/* "$RELEASE_DIR/web_dist/"

# Step 10: Atomic release link
echo "=== [10/15] Activating release artifacts ==="
ln -sfn "$RELEASE_DIR" "$APP_DIR/releases/current"
cp "$RELEASE_DIR/bin/omnesagent" "$APP_DIR/bin/omnesagent"
rsync -a --delete "$RELEASE_DIR/web_dist/" "$APP_DIR/web_dist/"

# Install update helper script
cp "$APP_DIR/source/scripts/omnesagent-update.sh" "$APP_DIR/omnesagent-update.sh" || true
chmod +x "$APP_DIR/omnesagent-update.sh" 2>/dev/null || true

# Step 11: Gateway Configuration
echo "=== [11/15] Writing Gateway configuration ==="
cat > "$APP_DIR/config.toml" <<EOF
[gateway]
host = "127.0.0.1"
port = 42617
web_dist_dir = "$APP_DIR/web_dist"
trust_forwarded_headers = true

[data]
dir = "$DATA_DIR/data"
EOF

chown -R omnesagent:omnesagent "$APP_DIR"
chown -R omnesagent:omnesagent "$DATA_DIR"

# Step 12: systemd Service Unit
echo "=== [12/15] Configuring systemd service ==="
cat > /etc/systemd/system/omnesagent.service <<EOF
[Unit]
Description=OmnesAgent Autonomous Gateway
After=network.target

[Service]
Type=simple
User=omnesagent
Group=omnesagent
WorkingDirectory=$APP_DIR
ExecStart=$APP_DIR/bin/omnesagent serve
Restart=always
RestartSec=5
Environment=OMNESAGENT_CONFIG=$APP_DIR/config.toml
LimitNOFILE=65536

# Security Sandboxing
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=false
ReadWritePaths=$DATA_DIR $APP_DIR
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable omnesagent.service
systemctl restart omnesagent.service

# Step 13: Nginx Proxy Configuration
echo "=== [13/15] Configuring Nginx reverse proxy ==="
NGINX_CONF="/etc/nginx/sites-available/omnesagent"
SERVER_NAME="${DOMAIN:-_}"
PUBLIC_PORT="${PORT:-80}"

cat > "$NGINX_CONF" <<EOF
server {
    listen $PUBLIC_PORT;
    server_name $SERVER_NAME;

    client_max_body_size 100M;
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/wasm;
    gzip_min_length 256;

    location / {
        proxy_pass http://127.0.0.1:42617;
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }

    location /ws/ {
        proxy_pass http://127.0.0.1:42617;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
    }
}
EOF

ln -sfn "$NGINX_CONF" /etc/nginx/sites-enabled/omnesagent
rm -f /etc/nginx/sites-enabled/default

# SSL Certification
if [[ -n "$DOMAIN" && "$NO_SSL" = false ]]; then
    echo "Obtaining SSL certificate via certbot for $DOMAIN..."
    systemctl reload nginx || true
    certbot --nginx -d "$DOMAIN" --non-interactive --agree-tos --email "$EMAIL" --redirect || {
        echo "WARNING: Certbot SSL setup failed, continuing with HTTP."
    }
fi

nginx -t
systemctl restart nginx

# Step 14: Firewall (UFW)
echo "=== [14/15] Configuring firewall ==="
if command -v ufw &>/dev/null; then
    ufw allow 22/tcp || true
    ufw allow 80/tcp || true
    ufw allow 433/tcp || true
    ufw allow 443/tcp || true
    if [[ -n "$PORT" && "$PORT" != "80" && "$PORT" != "443" ]]; then
        ufw allow "$PORT"/tcp || true
    fi
fi

# Step 15: Self-Check & Validation
echo "=== [15/15] Running health self-checks ==="
sleep 3

# 1. systemd active
if ! systemctl is-active --quiet omnesagent.service; then
    echo "ERROR: omnesagent.service is not running!" >&2
    journalctl -u omnesagent.service -n 30 --no-pager
    exit 1
fi

# 2. Gateway health endpoint
if ! curl -sf http://127.0.0.1:42617/api/health >/dev/null; then
    echo "ERROR: Gateway failed health check on http://127.0.0.1:42617/api/health" >&2
    exit 1
fi

# 3. Unauthenticated API returns 401
STATUS=$(curl -so /dev/null -w "%{http_code}" http://127.0.0.1:42617/api/auth/me || true)
if [[ "$STATUS" != "401" && "$STATUS" != "200" ]]; then
    echo "WARNING: Auth status endpoint returned unexpected code: $STATUS"
fi

# 4. Read credentials
ADMIN_CREDS_FILE="$DATA_DIR/.install-credentials"
TEMP_PASSWORD=""
if [[ -f "$ADMIN_CREDS_FILE" ]]; then
    TEMP_PASSWORD=$(cat "$ADMIN_CREDS_FILE" | cut -d':' -f2 | tr -d '\n\r')
fi

ACCESS_URL="http://${DOMAIN:-$(curl -s https://api.ipify.org || echo "YOUR_VPS_IP")}:${PUBLIC_PORT}"
if [[ -n "$DOMAIN" && "$NO_SSL" = false ]]; then
    ACCESS_URL="https://$DOMAIN"
fi

echo ""
echo "=========================================================================="
echo "   ★ OMNESAGENT WEB ADE INSTALLED SUCCESSFULLY! ★"
echo "=========================================================================="
echo "URL:                 $ACCESS_URL"
echo "Username:            admin"
if [[ -n "$TEMP_PASSWORD" ]]; then
    echo "Temporary Password:  $TEMP_PASSWORD"
    echo ""
    echo "ВНИМАНИЕ: При первом входе в браузер вам будет предложено"
    echo "установить новый постоянный пароль администратора."
else
    echo "Password:            (ранее установленный постоянный пароль)"
fi
echo "Update script:       $APP_DIR/omnesagent-update.sh"
echo "Service status:      sudo systemctl status omnesagent"
echo "Service logs:        sudo journalctl -u omnesagent -f"
echo "=========================================================================="
EOF
