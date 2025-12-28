# Advanced Usage

## Library Usage

Use vruffr as a Rust library for programmatic control.

```rust
use vruffr::{render_sketch, SketchOptions, SketchFillStyle};

fn main() -> anyhow::Result<()> {
    let svg_data = std::fs::read_to_string("input.svg")?;

    let options = SketchOptions {
        roughness: 1.5,
        bowing: 1.0,
        seed: 42,
        fill_style: SketchFillStyle::Hachure,
        hachure_angle: -41.0,
        hachure_gap: 4.0,
        ..Default::default()
    };

    let pixmap = render_sketch(&svg_data, &options)?;
    pixmap.save_png("output.png")?;

    Ok(())
}
```

## SVG Output

Generate sketch-style SVG instead of rasterized PNG:

```rust
use vruffr::render_to_svg;

let (svg_output, warnings) = render_to_svg(&svg_data, &options)?;
std::fs::write("output.svg", svg_output)?;
```

## Handling Warnings

```rust
use vruffr::render_sketch_with_warnings;

let (pixmap, warnings) = render_sketch_with_warnings(&svg_data, &options)?;

if warnings.has_text {
    eprintln!("Warning: Text elements not rendered");
}
if warnings.has_images {
    eprintln!("Warning: Embedded images not rendered");
}
```

## Validation Only

```rust
use vruffr::validate_svg;

let info = validate_svg(&svg_data)?;
println!("Size: {}x{}", info.width, info.height);
println!("Paths: {}", info.path_count);
```

## Batch Processing Script

```bash
#!/bin/bash
# Process all SVGs in a directory

INPUT_DIR="./input"
OUTPUT_DIR="./output"
mkdir -p "$OUTPUT_DIR"

for svg in "$INPUT_DIR"/*.svg; do
    name=$(basename "$svg" .svg)
    vruffr "$svg" -o "$OUTPUT_DIR/${name}.png" \
        --roughness 1.5 \
        --seed 42 \
        --quiet
    echo "Processed: $name"
done
```

## Integration with Other Tools

### With ImageMagick

```bash
# Add border after sketching
vruffr input.svg -o temp.png --background transparent
convert temp.png -bordercolor "#333" -border 20 final.png
```

### With Inkscape

```bash
# Convert complex SVG to simplified version first
inkscape input.svg --export-plain-svg=simple.svg
vruffr simple.svg -o output.png
```

### With FFmpeg (Animation)

```bash
# Create frames with varying roughness
for i in $(seq 1 30); do
    r=$(echo "scale=2; 0.5 + $i * 0.1" | bc)
    vruffr input.svg -o "frame_$i.png" --roughness $r --seed $i
done

# Combine to video
ffmpeg -framerate 10 -i frame_%d.png -c:v libx264 animation.mp4
```

## Performance Tips

1. **Use --quiet** in batch jobs to skip console output
2. **Set explicit dimensions** when you don't need SVG's native size
3. **Enable --deduplicate** for SVGs with known duplicates
4. **Use lower roughness** for faster rendering (fewer curve samples)

## Troubleshooting

### "SVG contains text elements"

Text must be converted to paths before rendering:

```bash
# Convert text to paths with Inkscape
inkscape input.svg --export-text-to-path --export-plain-svg=text-as-paths.svg
vruffr text-as-paths.svg -o output.png
```

### "SVG contains embedded images"

Embedded raster images are skipped. Trace them to paths first or composite after.

### Empty output

Check for:
- All-transparent paths
- Paths outside viewBox
- Very small paths with high roughness

Use `--dry-run` to validate:

```bash
vruffr input.svg -o /dev/null --dry-run
```
