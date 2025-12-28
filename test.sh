#!/usr/bin/env bash
# this_file: test.sh
# Quick quality check for vruffr
# Usage: ./test.sh [quick|full]

set -euo pipefail
cd "$(dirname "$0")"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log() { echo -e "${GREEN}==>${NC} $1"; }
fail() { echo -e "${RED}FAIL:${NC} $1" >&2; exit 1; }

quick_check() {
    log "Format check (vruffr packages only)"
    cargo fmt -p roughr -p rough_tiny_skia -p vruffr-cli --check || fail "cargo fmt --check failed"

    log "Clippy (CLI only - upstream crates have pre-existing warnings)"
    cargo clippy -p vruffr-cli --no-deps -- -D warnings || fail "clippy failed"

    log "Tests"
    cargo test -p roughr -p rough_tiny_skia -p vruffr-cli --quiet || fail "tests failed"

    echo ""
    log "All checks passed!"
}

full_check() {
    quick_check

    log "Build release"
    cargo build --release -p vruffr-cli

    log "Run example"
    cargo run -p vruffr-cli --example basic --quiet

    log "Dry-run validation"
    ./target/release/vruffr examples/sag.svg -o /dev/null --dry-run

    echo ""
    log "Full check passed!"
}

case "${1:-quick}" in
    quick) quick_check ;;
    full)  full_check ;;
    *)
        echo "Usage: $0 [quick|full]"
        exit 1
        ;;
esac
