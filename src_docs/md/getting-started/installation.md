# Installation

## From Source (Recommended)

Requires Rust 1.70+.

```bash
# Clone repository
git clone https://github.com/vexyart/vruffr.git
cd vruffr

# Build release binary
cargo build --release

# Binary is at target/release/vruffr
./target/release/vruffr --help
```

## Cargo Install

```bash
cargo install vruffr-cli
```

## macOS Universal Binary

Download the DMG from [Releases](https://github.com/vexyart/vruffr/releases):

```bash
# Mount DMG and copy binary
cp /Volumes/vruffr-*/vruffr /usr/local/bin/
chmod +x /usr/local/bin/vruffr
```

## Verify Installation

```bash
vruffr --version
vruffr --help
```

## As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
vruffr = "0.1"
```

## Build Options

### Development Build

```bash
cargo build
./target/debug/vruffr input.svg -o output.png
```

### Universal macOS Binary

```bash
./build.sh
# Creates target/universal-apple-darwin/release/vruffr
```

### Run Tests

```bash
cargo test -p roughr -p rough_tiny_skia -p vruffr-cli
```
