# vruffr Examples

Example SVG files and Rust code for vruffr.

## SVG Files

- `sag.svg` - Simple geometric shapes
- `tiger.svg` - Classic SVG tiger
- `demo-shapes.svg` - Created by demo.sh

## Running Examples

### Rust Examples

```bash
# Basic library usage
cargo run -p vruffr-cli --example basic
```

### CLI Demos

```bash
# Run all demos
./demo.sh all

# Specific demos
./demo.sh basic      # Basic conversions
./demo.sh styles     # Fill style comparison
./demo.sh roughness  # Roughness levels
./demo.sh adaptive   # Adaptive roughness
./demo.sh batch      # Batch processing
```

### Output

Demo outputs go to `examples/output/`.

```bash
ls examples/output/
```

## Quick Renders

```bash
# Render included examples
./target/release/vruffr examples/sag.svg -o /tmp/sag-sketch.png
./target/release/vruffr examples/tiger.svg -o /tmp/tiger-sketch.png --roughness 2.0
```
