# PLAN.md - vruffr Project Plan

## Scope

Transform SVG graphics into hand-drawn sketch-style output with wobbly lines, crosshatch fills, and artistic imperfection. vruffr is a Rust library, CLI tool, and future Python/WASM package.

## Background

This project unifies:
1. **rough-rs fork** (forked from orhanbalci/rough-rs) - core sketch primitives
2. **skesvg** (in extra/) - working SVG-to-sketch CLI tool

The goal is to decouple from upstream, integrate skesvg functionality, and create a polished independent project.

## Architecture

### Current State

```
vruffr/
├── roughr/              # Core sketch primitives (keep)
├── points_on_curve/     # Bezier utilities (keep)
├── svg_path_ops/        # SVG path manipulation (keep)
├── rough_tiny_skia/     # tiny-skia adapter (keep, fix panics)
├── rough_piet/          # piet adapter (keep)
├── rough_vello/         # vello adapter (keep)
├── rough_iced/          # iced adapter (DROP)
├── rough_plotters_svg/  # plotters adapter (DROP)
└── extra/
    ├── src/             # skesvg lib + CLI (INTEGRATE)
    └── ext/             # Reference libraries (READ-ONLY)
```

### Current State (Achieved)

```
vruffr/
├── roughr/              # Core sketch primitives
├── points_on_curve/     # Bezier utilities
├── svg_path_ops/        # SVG path manipulation
├── rough_tiny_skia/     # tiny-skia backend
├── rough_piet/          # piet backends
├── rough_vello/         # vello backend
├── cli/                 # Rust CLI tool (vruffr binary)
├── python/              # Python bindings scaffold (PyO3/maturin)
├── wasm/                # WASM bindings scaffold (wasm-bindgen)
├── examples/            # Example SVGs and Rust code
├── src_docs/            # MkDocs documentation source
├── .github/workflows/   # CI/CD (build, test, release, docs)
├── build.sh             # Master build script
├── demo.sh              # Demo runner
└── extra/               # Reference libraries (READ-ONLY, gitignored)
```

Note: Phase 2 (crate renaming to vruffr-*) deferred as optional polish.

### Rendering Pipeline

```
SVG Input
    ↓ [usvg - parse & normalize]
usvg::Tree
    ↓ [extract paths with transforms]
Path strings + colors + styles
    ↓ [DEDUPLICATION - remove redundant stacked paths]
Unique paths (identical overlapping paths merged)
    ↓ [ADAPTIVE SCALING - compute per-element roughness]
Per-element SketchOptions (size-dependent parameters)
    ↓ [vruffr-core - generate sketch OpSets]
OpSet (move/line/curve operations)
    ↓ [backend adapter - convert to native paths]
Backend-specific paths (tiny-skia Path, vello Scene, piet BezPath)
    ↓ [render]
Output (PNG / SVG / native surface)
```

### Supported Backends

| Backend | Surface | Output Formats | Status |
|---------|---------|----------------|--------|
| vruffr-skia | CPU pixmap | PNG | Working (panics fixed) |
| vruffr-piet | piet-cairo | PNG, PDF | Planned |
| vruffr-piet | piet-svg | SVG | Planned |
| vruffr-piet | piet-web | Canvas | Planned |
| vruffr-vello | GPU scene | PNG, Window | Planned |

## Critical Issues

### Panic in rough_tiny_skia (Line 281) - FIXED

**Location:** `rough_tiny_skia/src/skia_generator.rs:281`

**Status:** FIXED - The `opset_to_shape` function now returns `Option<Path>` instead of panicking. The fix includes:
- Changed return type to `Option<Path>`
- Added `has_drawing_op` tracking to detect empty paths
- Updated `SkiaOpset.ops` field to be `Option<Path>`
- Updated `draw()` method to skip None paths
- Added 9 comprehensive tests for edge cases

**Original Problem:** `PathBuilder::finish()` returned `None` when path was empty or degenerate, causing unwrap() to panic.

**Resolution:** Empty/degenerate paths are now gracefully handled by returning None and skipping during rendering.

## Phases

### Phase 1: Foundation Cleanup - COMPLETE

**Goal:** Fix critical issues and clean up workspace.

1. **Fix rough_tiny_skia panics** - DONE
   - Changed `opset_to_shape` to return `Option<Path>`
   - Updated `SkiaOpset.ops` field to be `Option<Path>`
   - Updated `draw()` method to skip None paths
   - Added 9 comprehensive tests

2. **Drop unused backend crates** - DONE
   - Removed rough_iced from workspace
   - Removed rough_plotters_svg from workspace
   - Deleted both directories
   - Updated root Cargo.toml

3. **Run tests to establish baseline** - PARTIAL
   - `cargo test --lib --tests` passes
   - Example tolerance.rs needs Cairo (platform-specific)

### Phase 1.5: Preprocessing Features - COMPLETE

**Goal:** Add path preprocessing for cleaner, more controlled output.

1. **Duplicate Path Filtering** - DONE
   - Detect paths with identical geometry stacked at same position
   - Compute path signature (bounding box, length, vertex count, command hash)
   - Merge duplicates: same roughened geometry, different strokes/fills
   - Added `--deduplicate` CLI flag (default: false)
   - Added `--dedup-epsilon` tolerance flag (default: 0.1px)
   - Core dedup module in roughr/src/dedup.rs with 12 tests

2. **Adaptive Roughness (Size-Dependent)** - DONE
   - Calculate characteristic size for each element (sqrt of bbox area)
   - Scale roughness based on element size vs reference size (100px)
   - Small elements get reduced roughness (stay legible)
   - Large elements can have increased roughness
   - Added `--adaptive-strength` CLI flag (0.0-2.0, default: 0.0 disabled)
   - Added `--reference-size` CLI flag (default: 100px)
   - 7 unit tests in roughr/src/core.rs

### Phase 2: Rename & Restructure

**Goal:** Create vruffr branding and crate structure.

1. **Rename crates**
   - roughr → vruffr-core
   - points_on_curve + svg_path_ops → vruffr-path
   - rough_tiny_skia → vruffr-skia
   - rough_piet → vruffr-piet
   - rough_vello → vruffr-vello

2. **Create crates/ directory**
   - Move all crates under crates/
   - Update workspace paths

3. **Update all internal references**
   - Cargo.toml dependencies
   - `use` statements
   - Documentation

### Phase 3: Integrate CLI - COMPLETE (core features)

**Goal:** Make vruffr the main CLI using local workspace crates.

1. **Move skesvg to cli/** - DONE
   - Created cli/Cargo.toml with path dependencies
   - Renamed to vruffr (library and binary)
   - Uses workspace path deps for roughr and rough_tiny_skia

2. **Remove catch_unwind workarounds** - DONE
   - Panic handlers removed from render_path() and collect_path_elements()
   - Clean code now that Option<Path> handles degenerate paths

3. **Expand CLI features** - DONE (core)
   - All SketchOptions exposed as flags (DONE)
   - SVG output mode (plain SVG) - DONE
   - CLI integration tests (8 tests) - DONE
   - SVG patch mode (embed into original) - DEFERRED (complex, low priority)
   - Backend selection flag - DEFERRED (only one backend currently)

### Phase 4: Documentation - COMPLETE

**Goal:** Complete project documentation.

1. **README.md** - DONE (new project introduction with all options)
2. **SPEC.md** - DONE (technical specification)
3. **API docs** - DONE (Rust doc comments on all public APIs)
4. **Examples** - DONE (cli/examples/basic.rs)
5. **CLAUDE.md** - DONE (updated for new structure)
6. **DEPENDENCIES.md** - DONE (package justifications)
7. **CHANGELOG.md** - DONE (initial entry)

### Phase 5: Python & WASM - PARTIAL

**Goal:** Language bindings for broader adoption.

1. **Scaffolding** - COMPLETE
   - python/ directory with PyO3/maturin setup
   - wasm/ directory with wasm-bindgen setup
   - Playground HTML page for WASM demo

2. **Rendering Pipelines** - TODO
   - Complete WASM render_to_svg implementation
   - Complete Python render_to_png/svg implementation
   - Fire-based Python CLI wrapper

### Phase 6: Advanced Features (Future)

**Goal:** Enhanced sketch effects.

1. **Color manipulation**
   - Monochrome modes (grayscale, sepia, duotone)
   - Color palette restrictions

2. **Post-processing filters**
   - Paper texture overlay
   - Noise/grain effects
   - Edge roughening

## Key Parameters

### SketchOptions (from skesvg)

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| roughness | f64 | 1.0 | Line perturbation (0-10) |
| bowing | f64 | 1.0 | Line curvature (0-10) |
| seed | u64 | 42 | Random seed for reproducibility |
| fill_style | enum | CrossHatch | Hachure, CrossHatch |
| hachure_angle | f32 | -41.0 | Angle of hachure lines |
| hachure_gap | f32 | 4.0 | Gap between hachure lines |
| fill_weight | f32 | 0.5 | Thickness of fill lines |
| stroke_width | Option<f32> | None | Override stroke width |
| background | Option<[u8;4]> | white | Background color |
| no_fill | bool | false | Skip fill rendering |
| no_stroke | bool | false | Skip stroke rendering |
| scale | f32 | 1.0 | Output scale factor |

### New Preprocessing Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| adaptive_strength | f32 | 0.0 | Size-dependent roughness scaling (0=off, 1=normal, 2=aggressive) |
| reference_size | f32 | 100.0 | Reference element size in pixels for adaptive scaling |
| deduplicate_paths | bool | true | Remove duplicate stacked paths before roughening |
| dedup_epsilon | f32 | 0.1 | Tolerance in pixels for path deduplication |

## Dependencies

### Core (Published Crates)
- usvg 0.45 - SVG parsing
- tiny-skia 0.11 - CPU rendering
- palette 0.7 - Color handling
- euclid 0.22 - 2D geometry
- rand 0.8 - Randomness

### CLI
- clap 4 - Argument parsing
- anyhow 1 - Error handling
- fontdb 0.23 - Font loading

### Backends (Optional)
- piet 0.8 - Vector graphics abstraction
- vello 0.5-0.6 - GPU rendering

## Success Criteria

1. `cargo test --workspace` passes with no panics
2. CLI can render extra/test-data/*.svg without crashes
3. All public APIs documented
4. README describes project clearly as independent (not a fork)
5. Examples work out of the box
