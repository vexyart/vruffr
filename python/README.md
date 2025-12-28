# vruffr

Python bindings for vruffr sketch renderer.

## Installation

```bash
# From source (requires Rust)
pip install maturin
cd python
maturin develop
```

## Usage

```python
import vruffr

# Basic render
svg = '<svg viewBox="0 0 100 100"><circle cx="50" cy="50" r="40" fill="blue"/></svg>'
png_bytes = vruffr.render_to_png(svg)

with open("output.png", "wb") as f:
    f.write(png_bytes)

# With options
opts = vruffr.SketchOptions(
    roughness=1.5,
    bowing=1.0,
    seed=42,
    fill_style="hachure"
)
svg_output = vruffr.render_to_svg(svg, opts)

# Validate
width, height, valid = vruffr.validate_svg(svg)
print(f"SVG: {width}x{height}, valid={valid}")
```

## API

### `render_to_png(svg_data, options=None) -> bytes`

Render SVG to PNG bytes.

### `render_to_svg(svg_data, options=None) -> str`

Render SVG to sketch-style SVG string.

### `validate_svg(svg_data) -> tuple[int, int, bool]`

Validate SVG and return (width, height, valid).

### `SketchOptions`

```python
SketchOptions(
    roughness=1.0,      # Line wobble (0.0-10.0)
    bowing=1.0,         # Line curve (0.0-10.0)
    seed=42,            # Random seed
    fill_style="crosshatch",  # "hachure" or "crosshatch"
    hachure_angle=-41,  # Fill line angle
    hachure_gap=4.0     # Fill line spacing
)
```

## Status

⚠️ **Work in Progress** - Full rendering pipeline not yet implemented in Python.
Use the CLI for full functionality.
