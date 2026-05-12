#!/usr/bin/env bash
# install.sh - Install vruffr CLI tool
# Vruffr: Rust utility for rough/sketchy vector graphics rendering.
# Installs the release binary via cargo install.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==> Installing vruffr via cargo..."
cargo install --path cli/

echo "==> Install complete."
