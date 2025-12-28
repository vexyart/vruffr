# Quick Start

Transform your first SVG in 60 seconds.

## Basic Usage

```bash
# Convert SVG to PNG
vruffr input.svg -o output.png

# Convert SVG to sketchy SVG
vruffr input.svg -o output.svg
```

## Adjust Roughness

```bash
# Smooth (minimal sketch effect)
vruffr input.svg -o smooth.png --roughness 0.5

# Default
vruffr input.svg -o default.png --roughness 1.0

# Rough (strong sketch effect)
vruffr input.svg -o rough.png --roughness 3.0
```

## Fill Styles

```bash
# Crosshatch (default) - grid pattern
vruffr input.svg -o cross.png --fill-style crosshatch

# Hachure - parallel lines
vruffr input.svg -o hatch.png --fill-style hachure
```

## Control Hachure Lines

```bash
# Change line angle (degrees)
vruffr input.svg -o angled.png --hachure-angle 45

# Change line spacing
vruffr input.svg -o dense.png --hachure-gap 2.0
vruffr input.svg -o sparse.png --hachure-gap 8.0
```

## Scale Output

```bash
# 2x size
vruffr input.svg -o large.png --scale 2.0

# Explicit dimensions
vruffr input.svg -o sized.png --width 800 --height 600
```

## Background Colors

```bash
# White (default)
vruffr input.svg -o white.png --background white

# Transparent
vruffr input.svg -o transparent.png --background transparent

# Custom hex color
vruffr input.svg -o colored.png --background "#f0e6d2"
```

## Reproducible Output

```bash
# Same seed = same output
vruffr input.svg -o v1.png --seed 12345
vruffr input.svg -o v2.png --seed 12345
# v1.png and v2.png are identical
```

## Validate Without Rendering

```bash
# Check if SVG is valid
vruffr input.svg -o /dev/null --dry-run
# Output: Valid SVG: input.svg (800x600, 42 paths)
```

## Stdin Input

```bash
# Pipe SVG content
cat logo.svg | vruffr - -o output.png
```

## Next Steps

- [CLI Reference](cli.md) - All options explained
- [Fill Styles Guide](../guides/fill-styles.md) - Visual comparisons
- [Advanced Usage](../guides/advanced.md) - Adaptive roughness, dedup
