#!/bin/bash
# Full quality check: format, lint, test
set -e

echo "==> Formatting..."
cargo fmt --check

echo "==> Linting..."
cargo clippy -- -D warnings

echo "==> Testing..."
cargo test

echo "==> Doc tests..."
cargo test --doc

echo "==> All checks passed!"
