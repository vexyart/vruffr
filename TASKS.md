# vruffr Architecture

## Scope

Transform SVG graphics into hand-drawn sketch-style output with wobbly lines, crosshatch fills, and artistic imperfection. vruffr is a Rust library, CLI tool, and Python/WASM package.

## Architecture

```
vruffr/
├── vruffr_core/         # vruffr-core: Core sketch primitives
├── vruffr_tiny_skia/    # vruffr-skia: tiny-skia backend
├── vruffr_piet/         # piet backend
├── vruffr_vello/        # vello GPU backend
├── cli/                 # Rust CLI tool (vruffr binary)
├── python/              # Python bindings (PyO3/maturin)
├── wasm/                # WASM bindings (wasm-bindgen)
├── points_on_curve/     # Bezier utilities
└── svg_path_ops/        # SVG path manipulation
```

See [PLAN.md](PLAN.md) for remaining work and [CHANGELOG.md](CHANGELOG.md) for completed phases.
