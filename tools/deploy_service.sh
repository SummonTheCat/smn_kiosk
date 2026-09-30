#!/usr/bin/env bash
set -euo pipefail

APP_NAME="smn_kiosk"
DEPLOY_DIR="/opt/${APP_NAME}"
STAGE_DIR="/opt/${APP_NAME}.new"
BACKUP_DIR="/opt/${APP_NAME}.previous"
SERVICE_NAME="${APP_NAME}.service"

if [ ! -f "Cargo.toml" ] || [ ! -d "res" ]; then
    echo "Error: run this script from the smn_kiosk project root."
    exit 1
fi

echo "=== Building ${APP_NAME} ==="
cargo build --release

if [ ! -x "target/release/${APP_NAME}" ]; then
    echo "Error: release binary was not produced."
    exit 1
fi

echo "=== Staging deployment ==="
sudo rm -rf "$STAGE_DIR"
sudo mkdir -p "$STAGE_DIR"
sudo install -m 0755 "target/release/${APP_NAME}" "$STAGE_DIR/${APP_NAME}"
sudo cp -a res "$STAGE_DIR/res"

echo "=== Stopping ${SERVICE_NAME} ==="
sudo systemctl stop "$SERVICE_NAME"

echo "=== Activating new deployment ==="
sudo rm -rf "$BACKUP_DIR"
if [ -d "$DEPLOY_DIR" ]; then
    sudo mv "$DEPLOY_DIR" "$BACKUP_DIR"
fi
sudo mv "$STAGE_DIR" "$DEPLOY_DIR"

if sudo systemctl start "$SERVICE_NAME"; then
    echo "=== Deployment successful ==="
    sudo rm -rf "$BACKUP_DIR"
    systemctl --no-pager --full status "$SERVICE_NAME"
    exit 0
fi

echo "Error: new deployment failed to start; restoring previous deployment."
sudo systemctl stop "$SERVICE_NAME" || true
sudo rm -rf "$DEPLOY_DIR"

if [ -d "$BACKUP_DIR" ]; then
    sudo mv "$BACKUP_DIR" "$DEPLOY_DIR"
    sudo systemctl start "$SERVICE_NAME"
    echo "Previous deployment restored."
else
    echo "No previous deployment was available to restore."
fi

exit 1
