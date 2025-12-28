# Sketch Options Guide

Understanding how each option affects the output.

## Roughness

Controls the amount of wobble and imperfection in lines.

| Value | Effect |
|-------|--------|
| 0.0 | Perfectly smooth (no sketch effect) |
| 0.5 | Subtle hand-drawn feel |
| 1.0 | Default - natural sketch |
| 2.0 | Loose, artistic |
| 5.0+ | Very rough, chaotic |

```bash
# Compare
vruffr input.svg -o r05.png --roughness 0.5
vruffr input.svg -o r10.png --roughness 1.0
vruffr input.svg -o r20.png --roughness 2.0
```

## Bowing

Controls how much lines bow/curve between endpoints.

| Value | Effect |
|-------|--------|
| 0.0 | Straight lines |
| 1.0 | Subtle natural curves |
| 3.0+ | Pronounced bowing |

Best combined with roughness for natural look:

```bash
vruffr input.svg -o natural.png --roughness 1.5 --bowing 1.2
```

## Seed

Randomization seed for reproducible output.

```bash
# Same seed = identical output
vruffr input.svg -o a.png --seed 12345
vruffr input.svg -o b.png --seed 12345
diff a.png b.png  # Files are identical

# Different seeds = different variations
vruffr input.svg -o v1.png --seed 1
vruffr input.svg -o v2.png --seed 2
```

## Adaptive Roughness

Scales roughness based on element size. Small elements get less roughness to stay legible.

| Strength | Effect |
|----------|--------|
| 0.0 | Disabled (uniform roughness) |
| 1.0 | Normal scaling |
| 2.0 | Aggressive (small elements almost smooth) |

```bash
# Mixed-size SVG with icons and large shapes
vruffr mixed.svg -o adaptive.png \
  --roughness 2.0 \
  --adaptive-strength 1.0 \
  --reference-size 100
```

The `--reference-size` sets what size (in pixels) gets the base roughness. Elements smaller than this get proportionally less roughness.

## Deduplication

Some SVG editors create duplicate stacked paths (same path with different stroke/fill). This can cause visual artifacts. Enable dedup to merge them:

```bash
vruffr complex.svg -o clean.png --deduplicate
```

Adjust matching tolerance with `--dedup-epsilon` (default 0.1 pixels).
