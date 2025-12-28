# vruffr-wasm

WASM bindings for vruffr sketch renderer.

## Building

```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Build WASM package
cd wasm
wasm-pack build --target web
```

## Usage

```javascript
import init, { render_to_svg, validate_svg } from './pkg/vruffr_wasm.js';

await init();

const svgInput = `<svg viewBox="0 0 100 100">
  <circle cx="50" cy="50" r="40" fill="blue"/>
</svg>`;

const options = {
  roughness: 1.5,
  bowing: 1.0,
  fill_style: "hachure",
  seed: 42
};

const sketchSvg = render_to_svg(svgInput, options);
document.getElementById('output').innerHTML = sketchSvg;
```

## API

### `render_to_svg(svg_data: string, options: object): string`

Render SVG to sketch-style SVG output.

Options:
- `roughness` (number): Line wobble (0.0-10.0, default: 1.0)
- `bowing` (number): Line curve (0.0-10.0, default: 1.0)
- `seed` (number): Random seed for reproducibility
- `fill_style` (string): "hachure" or "crosshatch"
- `hachure_angle` (number): Fill line angle in degrees
- `hachure_gap` (number): Fill line spacing in pixels

### `validate_svg(svg_data: string): object`

Validate SVG and return info: `{ width, height, valid }`.

### `version(): string`

Get vruffr version string.

## Status

⚠️ **Work in Progress** - Full rendering pipeline not yet implemented in WASM.
Use the CLI for full functionality.
