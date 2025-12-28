# vruffr

[![CI](https://github.com/vexyart/vruffr/actions/workflows/ci.yml/badge.svg)](https://github.com/vexyart/vruffr/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Transform SVG graphics into hand-drawn sketch-style output with wobbly lines, crosshatch fills, and artistic imperfection.

## Quick Install

```bash
# Build from source
git clone https://github.com/vexyart/vruffr.git
cd vruffr
cargo build --release
./target/release/vruffr --help
```

## Quick Start

```bash
# Convert SVG to sketchy PNG
vruffr input.svg -o output.png

# Increase roughness for more hand-drawn look
vruffr input.svg -o rough.png --roughness 2.5

# Use hachure fill instead of crosshatch
vruffr input.svg -o hatch.png --fill-style hachure
```

## More Examples

```bash
# Scale output 2x
vruffr input.svg -o large.png --scale 2.0

# Transparent background
vruffr input.svg -o transparent.png --background transparent

# Reproducible output with seed
vruffr input.svg -o v1.png --seed 12345

# Validate without rendering
vruffr input.svg -o /dev/null --dry-run

# Vintage look: sepia + film grain
vruffr input.svg -o vintage.png --color-mode sepia --noise 0.2

# Grayscale/monochrome
vruffr input.svg -o mono.png --color-mode grayscale
```

## Adaptive Roughness

When `--adaptive-strength` is set (0.0-2.0), roughness automatically scales based on element size:

- **Small elements** get reduced roughness to stay legible
- **Large elements** can have increased roughness for more sketch effect

The formula: `effective_roughness = base_roughness * (element_size / reference_size)^(strength * 0.5)`

```bash
# Normal scaling (recommended starting point)
vruffr input.svg -o output.png --adaptive-strength 1.0

# Aggressive scaling (more size variation)
vruffr input.svg -o output.png --adaptive-strength 1.5

# Custom reference size (default: 100px)
vruffr input.svg -o output.png --adaptive-strength 1.0 --reference-size 50
```

## All Options

| Flag | Default | Description |
|------|---------|-------------|
| `--roughness` | 1.0 | Line perturbation (0-10) |
| `--bowing` | 1.0 | Line curvature (0-10) |
| `--seed` | 42 | Random seed for reproducibility |
| `--fill-style` | crosshatch | Fill style: hachure, crosshatch |
| `--hachure-angle` | -41 | Angle of hachure lines (degrees) |
| `--hachure-gap` | 4.0 | Gap between hachure lines |
| `--fill-weight` | 0.5 | Thickness of fill lines |
| `--stroke-width` | - | Override stroke width |
| `--background` | white | Background color (name or #RRGGBB) |
| `--no-fill` | false | Skip fill rendering |
| `--no-stroke` | false | Skip stroke rendering |
| `--scale` | 1.0 | Output scale factor |
| `--adaptive-strength` | 0.0 | Size-dependent roughness (0=off, 1=normal, 2=aggressive) |
| `--reference-size` | 100 | Reference element size for adaptive scaling |
| `--deduplicate` | false | Remove duplicate stacked paths |
| `--dedup-epsilon` | 0.1 | Tolerance for path deduplication |
| `--color-mode` | color | Color mode: color, grayscale, sepia, invert |
| `--noise` | 0.0 | Film grain intensity (0.0-1.0) |
| `--edge-roughen` | 0.0 | Edge roughening intensity (0.0-1.0) |
| `--duotone` | - | Duotone colors: "#shadow,#highlight" |
| `--stroke-scale` | 1.0 | Stroke width multiplier |
| `--dpi` | 150 | Output resolution (SVG assumes 96) |

## Library Usage

```rust
use vruffr::{render_sketch, SketchOptions};

let svg_data = std::fs::read_to_string("input.svg")?;
let options = SketchOptions {
    roughness: 1.5,
    adaptive_strength: 1.0,
    ..Default::default()
};
let pixmap = render_sketch(&svg_data, &options)?;
pixmap.save_png("output.png")?;
```

## Crates

| Crate | Description |
|-------|-------------|
| `roughr` | Core sketch primitives (Rough.js port) |
| `rough_tiny_skia` | tiny-skia rendering backend |
| `rough_piet` | piet rendering backend |
| `rough_vello` | vello GPU rendering backend |
| `points_on_curve` | Bezier curve utilities |
| `svg_path_ops` | SVG path manipulation |

## License

MIT License - see [LICENSE](LICENSE).

Based on [rough-rs](https://github.com/orhanbalci/rough-rs) by [@orhanbalci](https://github.com/orhanbalci).
