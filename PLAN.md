# PLAN.md - vruffr Project Plan

## Scope

Transform SVG graphics into hand-drawn sketch-style output with wobbly lines, crosshatch fills, and artistic imperfection. vruffr is a Rust library, CLI tool, and Python/WASM package.

## Architecture

```
vruffr/
├── roughr/              # vruffr-core: Core sketch primitives
├── rough_tiny_skia/     # vruffr-skia: tiny-skia backend
├── rough_piet/          # piet backends
├── rough_vello/         # vello backend
├── cli/                 # Rust CLI tool (vruffr binary)
├── python/              # Python bindings (PyO3/maturin)
├── wasm/                # WASM bindings (wasm-bindgen)
├── points_on_curve/     # Bezier utilities
├── svg_path_ops/        # SVG path manipulation
└── extra/               # Reference libraries (READ-ONLY)
```

## Completed Phases

### Phase 1: Foundation - COMPLETE
- Panic fixes for empty/degenerate paths
- Option<Path> pattern for graceful handling

### Phase 1.5: Preprocessing - COMPLETE
- Path deduplication (`--deduplicate`)
- Adaptive roughness (`--adaptive-strength`)

### Phase 2: Crate Renaming - COMPLETE
- `roughr` → `vruffr-core`
- `rough_tiny_skia` → `vruffr-skia`

### Phase 3: CLI Integration - COMPLETE
- Full-featured CLI with all options
- PNG, SVG, SVG-plain output formats

### Phase 4: Documentation - COMPLETE
- README, SPEC, API docs
- MkDocs site

### Phase 5: Bindings - COMPLETE
- Full WASM rendering pipeline
- Full Python rendering pipeline

### Phase 6: Post-Processing - COMPLETE
- Color modes: grayscale, sepia, invert
- Noise/grain effect

### Phase 7: Edge Roughening - COMPLETE
- `--edge-roughen` for organic boundaries
- Alpha gradient edge detection

### Phase 8: Duotone Mode - COMPLETE
- `--duotone "#shadow,#highlight"`
- Luminance-based color mapping

## Success Criteria - ALL MET

- 118 tests passing
- All crates use vruffr-* naming
- All features work in CLI, WASM, and Python bindings
