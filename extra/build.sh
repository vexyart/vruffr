#!/usr/bin/env bash
# Build script for skesvg
# Creates release binary and universal macOS DMG

set -euo pipefail

PROJECT="skesvg"
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

echo "=== Building $PROJECT v$VERSION ==="

# Ensure we have the required targets
rustup target add aarch64-apple-darwin x86_64-apple-darwin 2>/dev/null || true

# Build for both architectures
echo "Building for Apple Silicon (aarch64)..."
cargo build --release --target aarch64-apple-darwin

echo "Building for Intel (x86_64)..."
cargo build --release --target x86_64-apple-darwin

# Create universal binary
echo "Creating universal binary..."
mkdir -p target/universal-apple-darwin/release
lipo -create \
    target/aarch64-apple-darwin/release/$PROJECT \
    target/x86_64-apple-darwin/release/$PROJECT \
    -output target/universal-apple-darwin/release/$PROJECT

# Verify universal binary
echo "Verifying universal binary..."
file target/universal-apple-darwin/release/$PROJECT
lipo -info target/universal-apple-darwin/release/$PROJECT

# Create DMG
echo "Creating DMG..."
DMG_NAME="${PROJECT}-${VERSION}-macos-universal"
DMG_DIR="target/dmg"
DMG_PATH="target/${DMG_NAME}.dmg"

rm -rf "$DMG_DIR"
mkdir -p "$DMG_DIR"

# Copy binary
cp target/universal-apple-darwin/release/$PROJECT "$DMG_DIR/"

# Create README for DMG
cat > "$DMG_DIR/README.txt" << EOF
$PROJECT v$VERSION
==================

Universal macOS binary (Apple Silicon + Intel)

Installation:
  1. Copy '$PROJECT' to /usr/local/bin/ or ~/bin/
  2. Make executable: chmod +x /usr/local/bin/$PROJECT

Usage:
  $PROJECT input.svg -o output.png
  $PROJECT input.svg -o output.svg
  $PROJECT --help

For more info: https://github.com/adam/skesvg
EOF

# Remove old DMG if exists
rm -f "$DMG_PATH"

# Create DMG
hdiutil create -volname "$DMG_NAME" \
    -srcfolder "$DMG_DIR" \
    -ov -format UDZO \
    "$DMG_PATH"

# Clean up
rm -rf "$DMG_DIR"

echo ""
echo "=== Build Complete ==="
echo "Release binary:   target/release/$PROJECT"
echo "Universal binary: target/universal-apple-darwin/release/$PROJECT"
echo "DMG:              $DMG_PATH"
echo ""
ls -lh "$DMG_PATH"
