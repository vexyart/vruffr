# SPEC.md - vruffr Technical Specification

## Overview

vruffr transforms mathematically perfect SVG vector graphics into hand-drawn sketch-style output with wobbly lines, crosshatch fills, and artistic imperfection. This document specifies the algorithms, data structures, and parameters that define vruffr's behavior.

## Core Concepts

### 1. Rendering Pipeline

```
SVG Input
    | [usvg - parse & normalize]
    v
usvg::Tree (flattened paths, resolved transforms, text-to-paths)
    | [deduplication filter]
    v
Unique Paths (redundant stacked paths removed)
    | [adaptive scaling]
    v
Per-Element SketchOptions (size-dependent roughness)
    | [roughr - generate sketch OpSets]
    v
OpSet (move/line/curve operations with wobble applied)
    | [backend adapter]
    v
Backend-specific paths (tiny-skia Path, vello Scene, piet BezPath)
    | [rasterize/render]
    v
Output (PNG / SVG / native surface)
```

### 2. Coordinate Systems

- **SVG coordinates**: Original document space
- **User coordinates**: After usvg transform flattening (absolute positions)
- **Device coordinates**: Pixel space after scaling

All roughness operations occur in user coordinates after transform flattening.

---

## Algorithm Specifications

### A. Geometric Perturbation (Line Wobble)

The core sketch effect transforms perfect Bezier curves into organic, wobbly strokes.

#### A.1 Segment Decomposition

Long segments are subdivided to create internal vertices for displacement:

```
Input: Line segment from P_start to P_end
Parameters: max_segment_length (default: 10px)

1. Calculate segment_length = distance(P_start, P_end)
2. If segment_length > max_segment_length:
   - num_segments = ceil(segment_length / max_segment_length)
   - Subdivide into num_segments equal parts
3. Each subsegment becomes a candidate for perturbation
```

#### A.2 Control Point Perturbation

Each segment is converted to a cubic Bezier with perturbed control points:

```
Input: Segment P0 -> P3
Parameters: roughness (R), bowing (B), seed (S)

1. Calculate perpendicular normal vector N
2. Generate random offsets using seeded RNG(S):
   - offset1 = R * (random() - 0.5) * segment_length * 0.2
   - offset2 = R * (random() - 0.5) * segment_length * 0.2

3. Calculate control points:
   - C1 = lerp(P0, P3, 0.25) + N * (offset1 + B * segment_length * 0.1)
   - C2 = lerp(P0, P3, 0.75) + N * (offset2 + B * segment_length * 0.1)

4. Output: Cubic Bezier (P0, C1, C2, P3)
```

#### A.3 Multi-Stroke Synthesis

For organic "searching line" effect:

```
Parameters: disable_multi_stroke (default: false)

If multi-stroke enabled:
  1. Generate Path1 with seed S
  2. Generate Path2 with seed S+1 and reduced opacity (0.5-0.7)
  3. Render both paths overlapping
```

---

### B. Duplicate Path Filtering

**Problem**: SVG files may contain multiple paths stacked at identical positions with identical shapes. When roughness is applied, each path receives different random perturbation, causing visual chaos instead of the intended clean overlap.

#### B.1 Path Signature Algorithm

```
Input: Collection of paths from usvg::Tree
Output: Deduplicated collection

For each path P:
  1. Compute geometric signature:
     - bounding_box: (min_x, min_y, max_x, max_y)
     - path_length: total arc length
     - vertex_count: number of path commands
     - command_hash: hash of command types (M, L, C, Z sequence)

  2. Compute position signature:
     - centroid: geometric center of path
     - start_point: first MoveTo coordinate
     - end_point: last point before close/end

  3. Combined signature = (bounding_box, path_length, vertex_count,
                           command_hash, centroid, start_point, end_point)

Path Equivalence Test:
  - Two paths are equivalent if:
    - |bbox1.min_x - bbox2.min_x| < epsilon (0.1px)
    - |bbox1.min_y - bbox2.min_y| < epsilon
    - |bbox1.max_x - bbox2.max_x| < epsilon
    - |bbox1.max_y - bbox2.max_y| < epsilon
    - |path_length1 - path_length2| < epsilon * max_length * 0.01
    - vertex_count1 == vertex_count2
    - command_hash1 == command_hash2

Deduplication:
  - Group paths by signature
  - For each group with >1 path:
    - Keep the first path (or the one with visible stroke/fill)
    - Mark others as duplicates
  - Return non-duplicate paths
```

#### B.2 Stroke/Fill Merge Strategy

When duplicates have different stroke/fill properties:

```
For duplicate group [P1, P2, ...]:
  1. Collect all unique strokes: [S1, S2, ...]
  2. Collect all unique fills: [F1, F2, ...]
  3. Keep primary path P1
  4. Apply roughness once to P1's geometry
  5. Render P1 with each unique stroke/fill combination
     (same roughened geometry, different styling)
```

---

### C. Adaptive Roughness (Size-Dependent)

**Problem**: Uniform roughness causes small elements to become illegible while large elements may appear too clean.

#### C.1 Element Size Calculation

```
Input: Path P with bounding box (min_x, min_y, max_x, max_y)

1. Calculate dimensions:
   - width = max_x - min_x
   - height = max_y - min_y

2. Calculate characteristic_size:
   - size = sqrt(width * height)  // Geometric mean
   - Alternatively: size = max(width, height)  // Maximum dimension
   - Alternatively: size = (width + height) / 2  // Average dimension
```

#### C.2 Adaptive Scaling Function

```
Input: base_roughness, characteristic_size, adaptive_strength
Parameters:
  - reference_size: 100px (calibration point)
  - min_scale: 0.2 (minimum roughness multiplier)
  - max_scale: 2.0 (maximum roughness multiplier)

Algorithm:
  1. Calculate size_ratio = characteristic_size / reference_size

  2. Calculate raw_scale:
     - If adaptive_strength == 0: scale = 1.0 (no adaptation)
     - Else: scale = size_ratio ^ (adaptive_strength * 0.5)

  3. Clamp scale to [min_scale, max_scale]

  4. effective_roughness = base_roughness * scale

  5. Return effective_roughness

Examples (base_roughness=1.0, adaptive_strength=1.0):
  - 10px element:  scale = 0.32, effective = 0.32 (gentler wobble)
  - 50px element:  scale = 0.71, effective = 0.71
  - 100px element: scale = 1.00, effective = 1.00 (reference)
  - 200px element: scale = 1.41, effective = 1.41
  - 500px element: scale = 2.00, effective = 2.00 (capped)
```

#### C.3 CLI Flag

```
--adaptive-strength <float>
  Default: 0.0 (disabled)
  Range: 0.0 - 2.0

  0.0: No adaptation (original behavior)
  0.5: Gentle adaptation
  1.0: Standard adaptation (recommended for mixed-size content)
  2.0: Aggressive adaptation
```

---

### D. Fill Algorithms (Hachure/Crosshatch)

#### D.1 Scanline Hachure Fill

```
Input: Closed path P, fill_angle (theta), fill_gap (d)
Output: Collection of line segments inside P

1. Compute bounding box of P, expand by d
2. Generate scanlines:
   - Rotate coordinate system by theta
   - Create horizontal lines at y = y_min, y_min + d, y_min + 2d, ...

3. For each scanline L:
   a. Find all intersections with P boundary
   b. Sort intersections by x coordinate
   c. Apply even-odd rule: segments [x0-x1], [x2-x3], ... are inside

4. Transform resulting segments back to original coordinates
5. Apply wobble to each hatch line segment
```

#### D.2 Crosshatch Fill

```
Input: Closed path P, primary_angle, secondary_angle_offset (default: 90)
Output: Two layers of hachure

1. Layer 1: Hachure at angle = primary_angle
2. Layer 2: Hachure at angle = primary_angle + secondary_angle_offset
3. Optionally for darker fills: Layer 3 at angle + 45
```

#### D.3 Fill Styles

| Style | Description |
|-------|-------------|
| `Hachure` | Single-direction parallel lines |
| `CrossHatch` | Two-layer perpendicular hatching |
| `Solid` | No hatching, render original fill |
| `ZigZag` | Connected zig-zag pattern |
| `ZigZagLine` | Zig-zag with perpendicular strokes |
| `Dots` | Stippling with small dots |
| `Dashed` | Dashed parallel lines |

---

### E. Stroke Rendering

#### E.1 Stroke Properties

```rust
struct StrokeOptions {
    width: f32,              // Base stroke width
    roughness: f32,          // Line wobble amplitude (0.0-10.0)
    bowing: f32,             // Line curvature bias (0.0-10.0)
    seed: u64,               // Deterministic randomness
    disable_multi_stroke: bool,
}
```

#### E.2 Pressure Simulation (Future)

Use curvature as proxy for drawing speed/pressure:

```
For each point t along curve:
  1. Calculate local curvature k(t)
  2. pressure(t) = 1.0 - (k(t) / max_curvature) * pressure_sensitivity
  3. stroke_width(t) = base_width * pressure(t)
```

---

## Data Structures

### OpSet

Core intermediate representation for sketch operations:

```rust
pub struct OpSet<F: Float> {
    pub op_set_type: OpSetType,  // Path, FillPath, FillSketch
    pub ops: Vec<Op<F>>,          // Drawing operations
    pub size: Option<Point2D<F>>, // Bounding dimensions
    pub path: Option<String>,     // Original SVG path data
}

pub struct Op<F: Float> {
    pub op: OpType,              // Move, LineTo, BCurveTo
    pub data: Vec<F>,            // Coordinates
}

pub enum OpType {
    Move,      // data: [x, y]
    LineTo,    // data: [x, y]
    BCurveTo,  // data: [cp1x, cp1y, cp2x, cp2y, x, y]
}
```

### SketchOptions

Complete configuration for sketch generation:

```rust
pub struct SketchOptions {
    // Geometry
    pub roughness: f32,          // Default: 1.0
    pub bowing: f32,             // Default: 1.0
    pub seed: u64,               // Default: random
    pub curve_step_count: u32,   // Default: 9
    pub curve_fitting: f32,      // Default: 0.95
    pub curve_tightness: f32,    // Default: 0.0

    // Stroke
    pub stroke_width: Option<f32>,
    pub disable_multi_stroke: bool,
    pub disable_multi_stroke_fill: bool,
    pub preserve_vertices: bool,

    // Fill
    pub fill_style: FillStyle,   // Default: CrossHatch
    pub fill_weight: f32,        // Default: 0.5
    pub hachure_angle: f32,      // Default: -41.0 degrees
    pub hachure_gap: f32,        // Default: 4.0px

    // Adaptive (new)
    pub adaptive_strength: f32,  // Default: 0.0 (disabled)
    pub reference_size: f32,     // Default: 100.0px

    // Deduplication (new)
    pub deduplicate_paths: bool, // Default: true
    pub dedup_epsilon: f32,      // Default: 0.1px
}
```

---

## Backend Specifications

### tiny-skia Backend (vruffr-skia)

- **Surface**: CPU pixmap
- **Output**: PNG
- **Path type**: `tiny_skia::Path`
- **Conversion**: `OpSet` -> `Option<Path>` (None for empty/degenerate)

### piet Backend (vruffr-piet)

- **Surfaces**: cairo (PNG/PDF), svg, coregraphics, direct2d, web
- **Path type**: `kurbo::BezPath`

### vello Backend (vruffr-vello)

- **Surface**: GPU scene
- **Output**: PNG, window
- **Path type**: `vello::peniko::kurbo::BezPath`

---

## CLI Parameters

### Input/Output

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `<INPUT>` | path | required | Input SVG file |
| `-o, --output` | path | derived | Output file path |
| `--format` | enum | png | Output format: png, svg, svgpatch |
| `--backend` | enum | skia | Rendering backend |

### Sketch Geometry

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--roughness` | f32 | 1.0 | Line wobble amplitude (0-10) |
| `--bowing` | f32 | 1.0 | Line curvature bias (0-10) |
| `--seed` | u64 | random | Random seed for reproducibility |
| `--adaptive-strength` | f32 | 0.0 | Size-dependent roughness (0-2) |
| `--no-multi-stroke` | bool | false | Disable double-stroke effect |

### Fill

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--fill-style` | enum | crosshatch | hachure, crosshatch, solid, zigzag, dots |
| `--fill-gap` | f32 | 4.0 | Hachure line spacing (px) |
| `--fill-angle` | f32 | -41.0 | Hachure angle (degrees) |
| `--fill-weight` | f32 | 0.5 | Hachure line thickness |
| `--no-fill` | bool | false | Skip fill rendering |

### Stroke

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--stroke-width` | f32 | original | Override stroke width |
| `--no-stroke` | bool | false | Skip stroke rendering |

### Processing

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--deduplicate` | bool | true | Remove duplicate paths |
| `--dedup-epsilon` | f32 | 0.1 | Deduplication tolerance (px) |

### Output

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--scale` | f32 | 1.0 | Output scale factor |
| `--background` | color | white | Background color (hex) |
| `--width` | u32 | auto | Override output width |
| `--height` | u32 | auto | Override output height |

---

## Future Extensions

### Color Manipulation

- Monochrome conversion (multiple algorithms)
- Grayscale, sepia, duotone modes
- Color palette restrictions

### Post-Processing Filters

- Paper texture overlay (Perlin noise alpha modulation)
- Noise/grain effects
- Edge roughening filter
- Smudge/blur effects

### Advanced Shading

- Gradient-to-density hatching
- Weighted Voronoi stippling
- Variable line weight based on luminosity

---

## References

1. Rough.js - https://github.com/rough-stuff/rough
2. rough-rs - https://github.com/orhanbalci/rough-rs
3. "Computational Aesthetics in Vector Graphics" (ref/101gem.md)
4. Perlin Noise for texture synthesis
5. De Casteljau's algorithm for curve subdivision
