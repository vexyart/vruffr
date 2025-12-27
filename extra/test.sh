#!/bin/bash
# Full quality check: format, lint, test
set -e

cd "$(dirname "$0")/.."

echo "==> Formatting..."
cargo fmt --check

echo "==> Linting (core packages)..."
cargo clippy -p roughr -p rough_tiny_skia -p vruffr-cli -- -D warnings

echo "==> Testing..."
cargo test -p roughr -p rough_tiny_skia -p vruffr-cli

echo "==> Doc tests..."
cargo test --doc -p roughr -p rough_tiny_skia -p vruffr-cli

echo "==> All checks passed!"
