#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

cd "$PROJECT_DIR"

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "Error: not inside a git repository"
    exit 1
fi

BRANCH="$(git rev-parse --abbrev-ref HEAD)"

if [ "$BRANCH" = "HEAD" ]; then
    echo "Error: repository is in detached HEAD state"
    exit 1
fi

echo "=== Syncing source from origin/${BRANCH} ==="
git fetch origin "$BRANCH"
git reset --hard "origin/${BRANCH}"

echo "=== Building and redeploying ==="
"${SCRIPT_DIR}/deploy_service.sh"
