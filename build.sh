#!/usr/bin/env bash
# this_file: build.sh
# Master build script for vruffr
# Usage: ./build.sh [command]
#   all       - Build everything (default)
#   rust      - Build Rust CLI
#   release   - Build release binary
#   universal - Build macOS universal binary
#   docs      - Build documentation
#   wasm      - Build WASM package
#   python    - Build Python wheel
#   test      - Run all tests
#   clean     - Clean build artifacts
#   help      - Show this help

set -euo pipefail
cd "$(dirname "$0")"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${GREEN}==>${NC} $1"; }
warn() { echo -e "${YELLOW}Warning:${NC} $1"; }
error() { echo -e "${RED}Error:${NC} $1" >&2; }

VERSION=$(grep '^version' cli/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

cmd_rust() {
    log "Building Rust CLI (debug)"
    cargo build -p vruffr-cli
    log "Binary: target/debug/vruffr"
}

cmd_release() {
    log "Building Rust CLI (release)"
    cargo build --release -p vruffr-cli
    log "Binary: target/release/vruffr"
}

cmd_universal() {
    if [[ "$(uname)" != "Darwin" ]]; then
        error "Universal binary only available on macOS"
        exit 1
    fi
    log "Building universal macOS binary"
    rustup target add aarch64-apple-darwin x86_64-apple-darwin 2>/dev/null || true

    cargo build --release --target aarch64-apple-darwin -p vruffr-cli
    cargo build --release --target x86_64-apple-darwin -p vruffr-cli

    mkdir -p target/universal-apple-darwin/release
    lipo -create \
        target/aarch64-apple-darwin/release/vruffr \
        target/x86_64-apple-darwin/release/vruffr \
        -output target/universal-apple-darwin/release/vruffr

    log "Universal binary: target/universal-apple-darwin/release/vruffr"
    file target/universal-apple-darwin/release/vruffr
}

cmd_docs() {
    log "Building documentation"
    if ! command -v mkdocs &> /dev/null; then
        warn "mkdocs not found. Installing..."
        pip install mkdocs-material mkdocs-minify-plugin
    fi
    cd src_docs
    mkdocs build -d ../docs
    cd ..
    cp src_docs/md/playground.html docs/
    log "Docs built: docs/"
}

cmd_wasm() {
    log "Building WASM package"
    if [[ ! -d "wasm" ]]; then
        warn "WASM scaffold not found. Run: ./build.sh scaffold-wasm"
        exit 1
    fi
    if ! command -v wasm-pack &> /dev/null; then
        warn "wasm-pack not found. Installing..."
        cargo install wasm-pack
    fi
    cd wasm
    wasm-pack build --target web
    cd ..
    log "WASM built: wasm/pkg/"
}

cmd_python() {
    log "Building Python wheel"
    if [[ ! -d "python" ]]; then
        warn "Python scaffold not found. Run: ./build.sh scaffold-python"
        exit 1
    fi
    if ! command -v maturin &> /dev/null; then
        warn "maturin not found. Installing..."
        pip install maturin
    fi
    cd python
    maturin build --release
    cd ..
    log "Python wheel built: target/wheels/"
}

cmd_test() {
    log "Running tests"
    cargo fmt --check
    cargo clippy -p roughr -p rough_tiny_skia -p vruffr-cli -- -D warnings
    cargo test -p roughr -p rough_tiny_skia -p vruffr-cli
    log "All tests passed!"
}

cmd_clean() {
    log "Cleaning build artifacts"
    cargo clean
    rm -rf docs/ wasm/pkg/ python/target/
    log "Cleaned"
}

cmd_all() {
    cmd_release
    cmd_test
    if command -v mkdocs &> /dev/null; then
        cmd_docs
    else
        warn "Skipping docs (mkdocs not installed)"
    fi
}

cmd_help() {
    head -15 "$0" | tail -13
}

# Main
case "${1:-all}" in
    rust)     cmd_rust ;;
    release)  cmd_release ;;
    universal) cmd_universal ;;
    docs)     cmd_docs ;;
    wasm)     cmd_wasm ;;
    python)   cmd_python ;;
    test)     cmd_test ;;
    clean)    cmd_clean ;;
    all)      cmd_all ;;
    help|--help|-h) cmd_help ;;
    *)
        error "Unknown command: $1"
        cmd_help
        exit 1
        ;;
esac
