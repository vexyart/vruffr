#!/usr/bin/env bash
# Test script to render test-data/sag.svg with various settings

set -euo pipefail

cd "$(dirname "$0")"

INPUT="test-data/sag.svg"
OUTDIR="test-data"

# Build in release mode
cargo build --release

BIN="./target/release/skesvg"

echo "=== Rendering test-data/sag.svg ==="

# SVG output (default format)
$BIN "$INPUT" -o "$OUTDIR/sag-default.svg"

# PNG output
$BIN "$INPUT" -o "$OUTDIR/sag-default.png" -f png

# Higher roughness (PNG)
$BIN "$INPUT" -o "$OUTDIR/sag-rough.png" -f png -r 3.0 -b 2.0

# Hachure fill (SVG)
$BIN "$INPUT" -o "$OUTDIR/sag-hachure.svg" --fill-style hachure

# 2x scale (PNG)
$BIN "$INPUT" -o "$OUTDIR/sag-2x.png" -f png -s 2.0

# Strokes only (SVG)
$BIN "$INPUT" -o "$OUTDIR/sag-strokes.svg" --no-fill

echo "=== Done ==="
echo "PNG files:"
ls -lh "$OUTDIR"/*.png 2>/dev/null || echo "  (none)"
echo "SVG files:"
ls -lh "$OUTDIR"/*.svg 2>/dev/null || echo "  (none)"
