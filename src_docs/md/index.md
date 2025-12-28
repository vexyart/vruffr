# vruffr

**Convert SVG to hand-drawn sketch-style output.**

vruffr transforms clean vector graphics into organic, hand-drawn looking artwork. Built on Rust for speed, it processes SVG files and outputs sketch-style PNG or SVG.

![Example](https://raw.githubusercontent.com/vexyart/vruffr/main/examples/assets/sag-comparison.png)

## Features

- **Sketch rendering** - Transforms paths with natural wobble and imperfection
- **Fill styles** - Hachure and crosshatch patterns
- **Adaptive roughness** - Scales effect based on element size
- **Deduplication** - Removes stacked duplicate paths
- **Multiple outputs** - PNG raster or SVG vector
- **Reproducible** - Seed-based randomization

## Quick Start

```bash
# Install
cargo install vruffr-cli

# Basic usage
vruffr input.svg -o output.png

# With options
vruffr input.svg -o output.png --roughness 2.0 --fill-style hachure
```

## Example

=== "Input SVG"

    Clean vector graphic

=== "Output (crosshatch)"

    ```bash
    vruffr logo.svg -o sketch.png --fill-style crosshatch
    ```

=== "Output (hachure)"

    ```bash
    vruffr logo.svg -o sketch.png --fill-style hachure
    ```

## Architecture

```
vruffr/
├── roughr/           # Core sketch primitives
├── rough_tiny_skia/  # CPU rendering backend
├── cli/              # Command-line interface
└── wasm/             # Browser playground (coming)
```

## Next Steps

- [Installation](getting-started/installation.md) - Get vruffr running
- [Quick Start](getting-started/quickstart.md) - First sketch in 60 seconds
- [CLI Reference](getting-started/cli.md) - All command options
- [Playground](playground.html) - Try in browser (WASM)
