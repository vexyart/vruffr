# Library API Reference

## Core Types

### SketchOptions

Main configuration struct for rendering.

```rust
pub struct SketchOptions {
    pub roughness: f64,        // Line wobble (0.0-10.0)
    pub bowing: f64,           // Line curve (0.0-10.0)
    pub seed: u64,             // Random seed
    pub width: Option<u32>,    // Output width
    pub height: Option<u32>,   // Output height
    pub fill_style: SketchFillStyle,
    pub hachure_angle: f32,    // Fill line angle (degrees)
    pub hachure_gap: f32,      // Fill line spacing (pixels)
    pub stroke_width: Option<f32>,
    pub background: Option<[u8; 4]>,  // RGBA
    pub no_fill: bool,
    pub no_stroke: bool,
    pub fill_weight: f32,
    pub scale: f32,
    pub font: Option<String>,
    pub font_size: Option<f32>,
    pub adaptive_strength: f32,
    pub reference_size: f32,
    pub deduplicate: bool,
    pub dedup_epsilon: f32,
}
```

### SketchFillStyle

Fill pattern for shapes.

```rust
pub enum SketchFillStyle {
    Hachure,     // Parallel lines
    Crosshatch,  // Grid pattern
}
```

### RenderWarnings

Warnings about unsupported SVG features.

```rust
pub struct RenderWarnings {
    pub has_text: bool,    // Text elements present
    pub has_images: bool,  // Embedded images present
}
```

### SvgInfo

Validation result.

```rust
pub struct SvgInfo {
    pub width: u32,
    pub height: u32,
    pub path_count: usize,
    pub warnings: RenderWarnings,
}
```

## Functions

### render_sketch

Render SVG to PNG pixmap.

```rust
pub fn render_sketch(
    svg_data: &str,
    options: &SketchOptions
) -> Result<Pixmap>
```

### render_sketch_with_warnings

Render with warning information.

```rust
pub fn render_sketch_with_warnings(
    svg_data: &str,
    options: &SketchOptions
) -> Result<(Pixmap, RenderWarnings)>
```

### render_to_svg

Render to SVG string.

```rust
pub fn render_to_svg(
    svg_data: &str,
    options: &SketchOptions
) -> Result<(String, RenderWarnings)>
```

### validate_svg

Validate SVG without rendering.

```rust
pub fn validate_svg(svg_data: &str) -> Result<SvgInfo>
```

## Example

```rust
use vruffr::{render_sketch, SketchOptions, SketchFillStyle};
use anyhow::Result;

fn main() -> Result<()> {
    let svg = r#"
        <svg viewBox="0 0 100 100">
            <circle cx="50" cy="50" r="40" fill="blue"/>
        </svg>
    "#;

    let options = SketchOptions {
        roughness: 1.5,
        fill_style: SketchFillStyle::Hachure,
        ..Default::default()
    };

    let pixmap = render_sketch(svg, &options)?;
    pixmap.save_png("circle.png")?;
    Ok(())
}
```

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `vruffr` (cli) | CLI and library facade |
| `roughr` | Core sketch primitives |
| `rough_tiny_skia` | CPU rendering backend |
| `points_on_curve` | Bezier utilities |
| `svg_path_ops` | SVG path manipulation |
