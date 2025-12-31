# PLAN.md - vruffr Project Plan

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

### Phase 2: Crate & Directory Renaming - COMPLETE
- `roughr/` → `vruffr_core/` (crate: vruffr-core)
- `rough_tiny_skia/` → `vruffr_tiny_skia/` (crate: vruffr-skia)
- `rough_piet/` → `vruffr_piet/`
- `rough_vello/` → `vruffr_vello/`

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

### Phase 9: Stroke Scaling - COMPLETE
- `--stroke-scale` multiplier for stroke widths
- Proportional thickness without overriding SVG values

### Phase 10: DPI Control - COMPLETE
- `--dpi` flag for output pixel density (default: 150)
- Higher resolution PNG outputs vs SVG's standard 96 DPI

### Phase 11: Paper Texture - COMPLETE
- `--paper-texture` flag (0.0-1.0)
- Multi-octave procedural noise for organic paper-like surface
- Smoothstep interpolation for natural blending

## Success Criteria - ALL MET

- 119+ tests passing
- All crates use vruffr-* naming
- All features work in CLI, WASM, and Python bindings
