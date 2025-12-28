# CLI Reference

Complete reference for all vruffr command-line options.

## Synopsis

```bash
vruffr [OPTIONS] <INPUT> -o <OUTPUT>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<INPUT>` | Input SVG file path, or `-` for stdin |

## Options

### Output

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--output` | `-o` | required | Output file (PNG or SVG) |
| `--format` | `-f` | auto | Output format: `png`, `svg`, `svgplain` |
| `--quiet` | `-q` | false | Suppress output messages |

Format is inferred from output extension if not specified.

### Sketch Parameters

| Option | Short | Default | Range | Description |
|--------|-------|---------|-------|-------------|
| `--roughness` | `-r` | 1.0 | 0.0-10.0 | Line wobble intensity |
| `--bowing` | `-b` | 1.0 | 0.0-10.0 | Line curve/bow amount |
| `--seed` | | 42 | any u64 | Random seed for reproducibility |

### Fill Options

| Option | Default | Description |
|--------|---------|-------------|
| `--fill-style` | crosshatch | Fill pattern: `hachure`, `crosshatch` |
| `--hachure-angle` | -41 | Angle of fill lines in degrees |
| `--hachure-gap` | 4.0 | Gap between fill lines in pixels |
| `--fill-weight` | 0.5 | Thickness of fill lines |
| `--no-fill` | false | Skip fill rendering (strokes only) |
| `--no-stroke` | false | Skip stroke rendering (fills only) |

### Dimensions

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--width` | | from SVG | Output width in pixels |
| `--height` | | from SVG | Output height in pixels |
| `--scale` | `-s` | 1.0 | Scale factor (e.g., 2.0 for 2x) |

### Appearance

| Option | Default | Description |
|--------|---------|-------------|
| `--background` | white | Background: `transparent`, `white`, `black`, `#RRGGBB` |
| `--stroke-width` | from SVG | Override stroke width |
| `--font` | | Font family for text elements |
| `--font-size` | from SVG | Font size in points |

### Advanced

| Option | Default | Description |
|--------|---------|-------------|
| `--adaptive-strength` | 0.0 | Adaptive roughness (0=off, 1=normal, 2=aggressive) |
| `--reference-size` | 100.0 | Reference size for adaptive scaling |
| `--deduplicate` | false | Remove duplicate stacked paths |
| `--dedup-epsilon` | 0.1 | Tolerance for path matching |
| `--dry-run` | false | Validate SVG without rendering |

## Examples

### Basic Conversion

```bash
vruffr logo.svg -o logo.png
vruffr diagram.svg -o diagram.svg
```

### Artistic Sketch

```bash
vruffr art.svg -o sketch.png \
  --roughness 2.5 \
  --bowing 1.5 \
  --fill-style hachure \
  --hachure-angle 30
```

### Clean Technical Drawing

```bash
vruffr schematic.svg -o clean.png \
  --roughness 0.3 \
  --fill-style crosshatch \
  --hachure-gap 2.0
```

### Large Poster

```bash
vruffr design.svg -o poster.png \
  --scale 4.0 \
  --adaptive-strength 1.0
```

### Transparent Sticker

```bash
vruffr icon.svg -o sticker.png \
  --background transparent \
  --no-fill
```

### Batch Processing

```bash
for f in *.svg; do
  vruffr "$f" -o "${f%.svg}.png" --seed 42
done
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (invalid input, write failure, etc.) |

## Environment

vruffr respects standard environment variables but has no specific configuration.
