#!/usr/bin/env bash
# Test script to render test-data/sag.svg with various settings

set -euo pipefail

cd "$(dirname "$0")/.."

INPUT="extra/test-data/sag.svg"
OUTDIR="extra/test-data"

# Build in release mode
cargo build --release

BIN="./target/release/vruffr"

echo "=== Rendering test-data/sag.svg ==="

# Default PNG output
$BIN "$INPUT" -o "$OUTDIR/sag-default.png"

# Higher roughness (PNG)
$BIN "$INPUT" -o "$OUTDIR/sag-rough.png" --roughness 3.0 --bowing 2.0

# Hachure fill (PNG)
$BIN "$INPUT" -o "$OUTDIR/sag-hachure.png" --fill-style hachure

# 2x scale (PNG)
$BIN "$INPUT" -o "$OUTDIR/sag-2x.png" --scale 2.0

# Strokes only (PNG)
$BIN "$INPUT" -o "$OUTDIR/sag-strokes.png" --no-fill

# Adaptive roughness (PNG)
$BIN "$INPUT" -o "$OUTDIR/sag-adaptive.png" --adaptive-strength 1.0

echo "=== Done ==="
echo "PNG files:"
ls -lh "$OUTDIR"/*.png 2>/dev/null || echo "  (none)"
