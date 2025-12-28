# WORK.md - vruffr Work Log

## Current Status

All core phases complete. Project production-ready.

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
# 118 tests passing
```

### Completed Phases

- **Phase 1**: Foundation cleanup (panic fixes)
- **Phase 1.5**: Preprocessing (dedup, adaptive roughness)
- **Phase 3**: CLI Integration (full-featured)
- **Phase 4**: Documentation (README, SPEC, API docs)
- **Phase 5**: Full WASM/Python rendering pipelines
- **Phase 6**: Color modes (grayscale, sepia, invert), noise/grain
- **Infrastructure**: CI/CD, build scripts, MkDocs

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

---

See `git log --oneline` for history.
