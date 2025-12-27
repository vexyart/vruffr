# vruffr

Transform SVG graphics into hand-drawn sketch-style output with wobbly lines, crosshatch fills, and artistic imperfection.

## Installation

```bash
cargo install --path cli
```

Or build from source:

```bash
cargo build --release
./target/release/vruffr input.svg -o output.png
```

## CLI Usage

```bash
# Basic usage
vruffr input.svg -o output.png

# Adjust roughness (0-10, default: 1.0)
vruffr input.svg -o output.png --roughness 2.5

# Change fill style
vruffr input.svg -o output.png --fill-style hachure

# Scale output
vruffr input.svg -o output.png --scale 2.0

# Adaptive roughness (size-dependent)
vruffr input.svg -o output.png --adaptive-strength 1.0
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
