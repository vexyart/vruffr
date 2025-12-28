# Fill Styles Guide

vruffr supports two fill styles for shape interiors.

## Crosshatch (Default)

Grid pattern with lines at two angles. Creates denser, more traditional shading.

```bash
vruffr input.svg -o crosshatch.png --fill-style crosshatch
```

Best for:

- Technical illustrations
- Architectural sketches
- Dense shading

## Hachure

Parallel lines at a single angle. Creates lighter, more open fills.

```bash
vruffr input.svg -o hachure.png --fill-style hachure
```

Best for:

- Artistic sketches
- Quick renders
- Large areas

## Controlling Fill Lines

### Angle

```bash
# Horizontal lines
vruffr input.svg -o h0.png --fill-style hachure --hachure-angle 0

# Vertical lines
vruffr input.svg -o h90.png --fill-style hachure --hachure-angle 90

# Diagonal (default is -41)
vruffr input.svg -o h45.png --fill-style hachure --hachure-angle 45
```

### Gap (Density)

```bash
# Dense (lines close together)
vruffr input.svg -o dense.png --hachure-gap 2.0

# Default spacing
vruffr input.svg -o normal.png --hachure-gap 4.0

# Sparse (lines far apart)
vruffr input.svg -o sparse.png --hachure-gap 8.0
```

### Weight (Line Thickness)

```bash
# Thin fill lines
vruffr input.svg -o thin.png --fill-weight 0.3

# Default
vruffr input.svg -o default.png --fill-weight 0.5

# Thick fill lines
vruffr input.svg -o thick.png --fill-weight 1.0
```

## Disabling Fill or Stroke

```bash
# Strokes only (no fill)
vruffr input.svg -o strokes.png --no-fill

# Fills only (no strokes)
vruffr input.svg -o fills.png --no-stroke
```

## Combining Options

```bash
# Artistic sketch with loose hachure
vruffr art.svg -o artistic.png \
  --fill-style hachure \
  --hachure-angle 30 \
  --hachure-gap 5.0 \
  --roughness 2.0

# Technical diagram with tight crosshatch
vruffr diagram.svg -o technical.png \
  --fill-style crosshatch \
  --hachure-gap 2.0 \
  --roughness 0.5
```
