# WORK.md - vruffr Work Log

## Current Status

All core phases complete. Project production-ready.

### Recent Work (2025-12-31)

- Updated all crate metadata for crates.io publication readiness
- Aligned versions to 2.0.4 across all crates
- Fixed repository/homepage URLs to point to vexyart/vruffr
- Added rust-version = "1.70" for MSRV
- Fixed README crate names table
- **Added paper texture overlay** (`--paper-texture` flag, 0.0-1.0)
  - Multi-octave procedural noise for organic paper-like surface
  - Uses smoothstep interpolation for natural blending
  - Exposed in CLI, WASM, and Python bindings

### Quick Commands

```bash
./test.sh quick    # Format, clippy, tests
./test.sh full     # + build, example, validation
./demo.sh all      # Generate example outputs
./build.sh release # Build release binary
```

### Test Summary

```
cargo test -p vruffr-core -p vruffr-skia -p vruffr-cli
# 119 tests passing
```

### Completed Phases

- **Phase 1**: Foundation cleanup (panic fixes)
- **Phase 1.5**: Preprocessing (dedup, adaptive roughness)
- **Phase 2**: Crate & directory renaming (vruffr_*)
- **Phase 3**: CLI Integration (full-featured)
- **Phase 4**: Documentation (README, SPEC, API docs)
- **Phase 5**: Full WASM/Python rendering pipelines
- **Phase 6**: Color modes (grayscale, sepia, invert), noise/grain
- **Phase 7**: Edge roughening (--edge-roughen)
- **Phase 8**: Duotone mode (--duotone)
- **Phase 9**: Stroke scaling (--stroke-scale)
- **Phase 10**: DPI control (--dpi, default 150)

### Features

| Feature | CLI Flag | Values |
|---------|----------|--------|
| Roughness | `--roughness` | 0.0-10.0 |
| Bowing | `--bowing` | 0.0-10.0 |
| Fill style | `--fill-style` | hachure, crosshatch |
| Color mode | `--color-mode` | color, grayscale, sepia, invert |
| Noise | `--noise` | 0.0-1.0 |
| Adaptive | `--adaptive-strength` | 0.0-2.0 |
| Dedup | `--deduplicate` | flag |
| Background | `--background` | transparent, white, black, #RGB, #RGBA, #RRGGBB, #RRGGBBAA |
| Edge roughen | `--edge-roughen` | 0.0-1.0 |
| Duotone | `--duotone` | "#shadow,#highlight" |
| Stroke scale | `--stroke-scale` | multiplier (e.g., 2.0) |
| DPI | `--dpi` | pixels per inch (default: 150, SVG assumes 96) |
| Paper texture | `--paper-texture` | 0.0-1.0 |

---

See `git log --oneline` for history.
