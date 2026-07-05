# Changelog

All notable changes to vruffr will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.7] - 2026-07-05

### Fixed

- **CI was broken:** the clippy, test, and docs jobs referenced crate names that
  no longer exist (`roughr`, `rough_tiny_skia`) after the rename to
  `vruffr-core` / `vruffr-skia`. Every gated job would have failed with
  "package ID specification did not match any packages". Renamed to the real
  packages and widened the clippy/check gate to `--workspace --lib`.
- **piet and vello backends did not compile:** both still referenced
  `roughr::core::LineCap` / `LineJoin` (leftover from the rough-rs fork) despite
  depending on `vruffr-core`. Repointed to `vruffr_core::core::*` and cleared the
  residual clippy lints (`unwrap_or_default`, `unnecessary_cast`, `map_clone`,
  `too_many_arguments`). All four rendering backends now build warning-free.
- Corrected stale crate names (`roughr`, `rough_piet`, `rough_vello`) throughout
  the architecture reference docs.

### Changed

- Normalized formatting across the workspace with stable `rustfmt` so
  `cargo fmt --all --check` passes in CI (the tree was previously formatted with
  nightly-only options that CI's stable toolchain does not apply).

### Added

- Project icon at `docs/assets/icon.png` (sourced from `src_docs/md/assets/`),
  wired in as the MkDocs logo and favicon.

## [2.0.6] - 2025-12-31

### Security

- Updated pyo3 from 0.22 to 0.24 to fix RUSTSEC-2025-0020 (buffer overflow risk)

### Changed

- Fixed clippy warnings in vruffr-core: `impl Display` replaces `impl ToString`, derive `Default` for `LineCap`
- Fixed redundant field initialization patterns (`ops: ops` → `ops`)
- Added crate-level clippy allows for legacy Rough.js port patterns in vruffr-core and vruffr-skia
- Moved WASM profile settings to workspace root Cargo.toml (eliminates cargo warning)
- Updated Python bindings to use pyo3 0.24 API (`PyBytes::new` instead of deprecated `new_bound`)

## [2.0.5] - 2025-12-31

### Added

- **Paper texture overlay**: `--paper-texture` flag (0.0-1.0)
  - Multi-octave procedural noise for organic paper-like surface
  - Smoothstep interpolation for natural blending
  - Available in CLI, WASM, and Python bindings

- **CLI help examples**: Added usage examples to `--help` output

### Changed

- Suppressed clippy warnings in forked svg_path_ops code via crate-level allows
- Updated all crate metadata for crates.io publication readiness

## [2.0.4] - 2025-12-28

### Added

- **DPI control**: `--dpi` flag for output pixel density (default: 150)
  - Higher resolution PNG outputs vs SVG's standard 96 DPI
  - Effective scale = user_scale × (dpi / 96)
  - Added to WASM and Python bindings

- **Stroke scaling**: `--stroke-scale` multiplier for stroke widths
  - Proportional thickness without overriding SVG values
  - Default: 1.0 (no scaling)

### Fixed

- Clippy warning: use `unwrap_or_default()` instead of `unwrap_or(Vec::new())`

## [2.0.3] - 2025-12-28

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
