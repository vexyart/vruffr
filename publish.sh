#!/usr/bin/env bash
# publish.sh - Build, version, and publish vruffr
# Vruffr: Rust utility for rough/sketchy vector graphics rendering.
# Calls build.sh + install.sh, bumps version with gitnextver, then publishes to crates.io.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==> Running build..."
"$SCRIPT_DIR/build.sh"

echo "==> Running install..."
"$SCRIPT_DIR/install.sh"

echo "==> Bumping version with gitnextver..."
uvx gitnextver@latest

echo "==> Publishing to crates.io..."
cargo publish -p vruffr-cli

echo "==> Publish complete."
