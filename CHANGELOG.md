# Changelog

All notable changes to vruffr will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Post-processing color modes**: `--color-mode` flag with options:
  - `grayscale` / `mono` - ITU-R BT.601 luminance conversion
  - `sepia` - Classic sepia tone matrix transform
  - `invert` / `negative` - Color inversion

- **Film grain effect**: `--noise` flag (0.0-1.0) for reproducible noise/grain

- **Edge roughening**: `--edge-roughen` flag (0.0-1.0) for organic boundaries
  - Detects edges via alpha gradient
  - Applies random pixel displacement near edges
  - LCG random for reproducibility

- **Duotone color mode**: `--duotone "#shadow,#highlight"` flag
  - Maps luminance to gradient between two colors
  - Uses ITU-R BT.601 luminance calculation
  - Supports short hex (#RGB) and long hex (#RRGGBB)

- **Short hex color support**: `#RGB` and `#RGBA` formats now work
  - `#fff` → white, `#f00` → red, `#0f0` → green
  - `#f008` → semi-transparent red

- **WASM bindings** (`wasm/`)
  - `render_to_png_base64()`, `render_to_svg()`, `validate_svg()`
  - All options exposed via JS object

- **Python bindings** (`python/`)
  - `render_to_png()`, `render_to_file()`, `render_to_svg()`, `validate_svg()`
  - `SketchOptions` class with all parameters

- **Project infrastructure**
  - GitHub Actions: CI, release, docs workflows
  - `build.sh` master script, `demo.sh` runner, `test.sh` quality checks
  - MkDocs Material documentation scaffold

- **Adaptive roughness scaling**: New `--adaptive-strength` and `--reference-size` CLI flags
  - Automatically scales roughness based on element size
  - Smaller elements get proportionally less roughness to prevent distortion
  - Formula: `effective = base * (size/reference)^(strength*0.5)`
  - Default: disabled (strength=0.0)

- **Path deduplication module** (`roughr::dedup`)
  - `PathSignature` struct for geometric identity (bbox, length, vertex count, command hash, centroid)
  - `deduplicate_paths()` function with bucket-based matching
  - Epsilon-tolerant path equivalence testing
  - Handles overlapping stroke/fill paths from SVG editors

- **CLI as workspace member** (`cli/`)
  - Renamed from skesvg to vruffr
  - Uses path dependencies to roughr and rough_tiny_skia
  - Binary: `vruffr`

### Fixed

- **rough_tiny_skia panic on degenerate paths** (line 281)
  - `opset_to_shape()` now returns `Option<Path>` instead of panicking
  - `SkiaOpset.ops` field changed to `Option<Path>`
  - `draw()` method gracefully skips None paths
  - Added 9 comprehensive tests for edge cases

### Removed

- `rough_iced` backend (unused)
- `rough_plotters_svg` backend (unused)

### Changed

- Workspace now includes cli crate
- roughr Options struct has new fields: `adaptive_strength`, `reference_size`
- **Crate renaming**: `roughr` → `vruffr-core`, `rough_tiny_skia` → `vruffr-skia`

## [0.12.0] - Previous Release

Initial fork from rough-rs with roughr, rough_piet, rough_tiny_skia, rough_vello backends.
