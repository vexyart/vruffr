# DEPENDENCIES.md

Dependencies used in vruffr with justification.

## CLI (vruffr-cli)

| Dependency | Version | Purpose |
|------------|---------|---------|
| `clap` | 4.x | CLI argument parsing (derive macros) |
| `anyhow` | 1.x | Error handling with context |
| `usvg` | 0.45 | SVG parsing and normalization |
| `fontdb` | 0.23 | System font loading for text |
| `tiny-skia` | 0.11 | CPU rasterization |
| `palette` | 0.7 | Color manipulation (HSL/RGB) |

## Core (roughr)

| Dependency | Version | Purpose |
|------------|---------|---------|
| `euclid` | 0.22 | 2D geometry types |
| `rand` | 0.8 | Random number generation for sketch effects |
| `svgtypes` | 0.15 | SVG path parsing |
| `num-traits` | 0.2 | Generic numeric traits |

## Rendering (rough_tiny_skia)

| Dependency | Version | Purpose |
|------------|---------|---------|
| `tiny-skia` | 0.11 | Path building and rendering |

## Other Backends

### rough_piet
| Dependency | Version | Purpose |
|------------|---------|---------|
| `piet` | 0.8 | Vector graphics abstraction |
| `piet-common` | 0.8 | Common piet implementations |

### rough_vello
| Dependency | Version | Purpose |
|------------|---------|---------|
| `vello` | 0.5 | GPU vector graphics |
| `peniko` | 0.3 | Color/brush types for vello |

## Version Constraints

- **usvg 0.45** required for compatibility with tiny-skia 0.11
- **vello 0.5** in rough_vello (may need update for vello 0.6+)

## Dev Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `tempfile` | 3.x | Temporary files in tests |
