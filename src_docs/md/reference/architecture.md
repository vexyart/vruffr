# Architecture

## Overview

vruffr transforms SVG paths through a pipeline:

```
SVG Input → Parse → Preprocess → Sketch → Render → Output
```

## Pipeline Stages

### 1. Parse (usvg)

SVG is parsed and normalized using `usvg`. This handles:

- CSS styling resolution
- Transform flattening
- Gradient/pattern expansion
- Unit conversion

### 2. Preprocess

Optional preprocessing steps:

- **Deduplication**: Removes stacked duplicate paths
- **Path signature**: Computes bbox, length, vertex count, hash

### 3. Sketch (vruffr-core)

Each path is transformed into sketch-style operations:

- Line segments get wobble based on roughness
- Curves are sampled and re-approximated
- Fills are converted to hachure/crosshatch patterns

Core types:

```rust
// vruffr_core::core
pub struct Options {
    pub roughness: f64,
    pub bowing: f64,
    pub seed: u64,
    // ...
}

// vruffr_core::generator
pub struct Generator { /* ... */ }
```

### 4. Render (vruffr-skia)

Sketch operations are rendered to pixels using `tiny-skia`:

```rust
// vruffr-skia
pub fn draw(canvas: &mut Pixmap, drawable: &Drawable, paint: &Paint)
```

### 5. Output

Results are written as:

- **PNG**: Direct pixmap encoding
- **SVG**: Path data converted back to SVG elements

## Crate Dependencies

```
vruffr-cli
    ├── vruffr-core (core primitives)
    │   └── points_on_curve
    ├── vruffr-skia (rendering)
    │   └── tiny-skia
    ├── svg_path_ops
    ├── usvg (SVG parsing)
    └── anyhow, clap
```

## Key Algorithms

### Adaptive Roughness

Scales roughness based on element size to keep small elements legible:

```rust
fn compute_effective_roughness(bbox: Rect, options: &Options) -> f32 {
    let size = (bbox.width() * bbox.height()).sqrt();
    let ratio = size / options.reference_size;
    let scale = ratio.powf(options.adaptive_strength * 0.5);
    options.roughness * scale.clamp(0.2, 2.0)
}
```

### Path Signature (Dedup)

Paths are grouped by signature for deduplication:

```rust
pub struct PathSignature {
    bbox: (i32, i32, i32, i32),  // Quantized bounding box
    length: i32,                  // Quantized arc length
    vertex_count: usize,          // Number of commands
    command_hash: u64,            // Hash of path commands
    centroid: (i32, i32),         // Center point
}
```

### Hachure Fill

Fills are computed by:

1. Computing path bounding box
2. Generating scan lines at specified angle
3. Finding intersections with path boundary
4. Drawing sketchy lines between intersection pairs

## Extensibility

### Adding Backends

Implement the `Drawable` trait for new rendering backends:

```rust
pub trait Drawable {
    fn draw(&self, o: &OpSet<f32>, paint: &Paint);
}
```

Existing backends:

- `vruffr-skia`: CPU rasterization
- `vruffr-piet`: Cross-platform 2D (uses CoreGraphics on macOS)
- `vruffr-vello`: GPU-accelerated (experimental)

### Adding Fill Styles

Extend `SketchFillStyle` and implement in `vruffr_core::filler`:

```rust
pub enum SketchFillStyle {
    Hachure,
    Crosshatch,
    // Solid,      // Future
    // Dots,       // Future
}
```
