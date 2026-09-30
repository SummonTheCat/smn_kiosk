#!/usr/bin/env bash
set -euo pipefail

APP_NAME="smn_kiosk"
DEPLOY_DIR="/opt/${APP_NAME}"
SERVICE_FILE="/etc/systemd/system/${APP_NAME}.service"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
RUN_USER="${SUDO_USER:-$USER}"

cd "$PROJECT_DIR"

sudo tee "$SERVICE_FILE" >/dev/null <<EOF_SERVICE
[Unit]
Description=SMN Kiosk server
After=network.target

[Service]
Type=simple
User=${RUN_USER}
WorkingDirectory=${DEPLOY_DIR}
ExecStart=${DEPLOY_DIR}/${APP_NAME} --public --port 8000
Restart=always
RestartSec=2

[Install]
WantedBy=multi-user.target
EOF_SERVICE

sudo systemctl daemon-reload
sudo systemctl enable "$APP_NAME"

"${SCRIPT_DIR}/deploy_service.sh"
