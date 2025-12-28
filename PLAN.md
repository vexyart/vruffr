# PLAN.md - vruffr Project Plan

## Scope

Transform SVG graphics into hand-drawn sketch-style output with wobbly lines, crosshatch fills, and artistic imperfection. vruffr is a Rust library, CLI tool, and Python/WASM package.

## Current Architecture

```
vruffr/
├── roughr/              # Core sketch primitives
├── points_on_curve/     # Bezier utilities
├── svg_path_ops/        # SVG path manipulation
├── rough_tiny_skia/     # tiny-skia backend
├── rough_piet/          # piet backends
├── rough_vello/         # vello backend
├── cli/                 # Rust CLI tool (vruffr binary)
├── python/              # Python bindings (PyO3/maturin)
├── wasm/                # WASM bindings (wasm-bindgen)
└── extra/               # Reference libraries (READ-ONLY)
```

## Phase 2: Crate Renaming

**Goal:** Rename crates to vruffr-* branding for consistency.

### 2.1 Rename roughr -> vruffr-core

1. Update `roughr/Cargo.toml`:
   - Change `name = "roughr"` to `name = "vruffr-core"`
   - Keep same version

2. Update all dependents:
   - `rough_tiny_skia/Cargo.toml`: `roughr = { path = "../roughr" }` -> `vruffr-core = { path = "../roughr" }`
   - `rough_piet/Cargo.toml`: same change
   - `rough_vello/Cargo.toml`: same change
   - `cli/Cargo.toml`: same change
   - `wasm/Cargo.toml`: same change
   - `python/Cargo.toml`: same change

3. Update `use` statements in all source files:
   - `use roughr::` -> `use vruffr_core::`

4. Update root `Cargo.toml` workspace member (path stays same, just package name changes)

### 2.2 Rename rough_tiny_skia -> vruffr-skia

1. Update `rough_tiny_skia/Cargo.toml`:
   - Change `name = "rough_tiny_skia"` to `name = "vruffr-skia"`

2. Update dependents:
   - `cli/Cargo.toml`: `rough_tiny_skia = { path = "../rough_tiny_skia" }` -> `vruffr-skia = { path = "../rough_tiny_skia" }`
   - `wasm/Cargo.toml`: same change
   - `python/Cargo.toml`: same change

3. Update `use` statements:
   - `use rough_tiny_skia::` -> `use vruffr_skia::`

### 2.3 Update Documentation

1. Update README.md references
2. Update CLAUDE.md references
3. Update any doc comments

### 2.4 Verification

1. `cargo build --workspace`
2. `cargo test -p vruffr-core -p vruffr-skia -p vruffr-cli`
3. Verify CLI still works

## Phase 7: Edge Roughening Filter

**Goal:** Add post-processing filter that roughens the edges of rendered output.

### 7.1 Design

Edge roughening applies a displacement effect to pixels near edges (where alpha or color changes sharply). This creates a more organic, hand-drawn boundary.

Algorithm:
1. Detect edges using alpha gradient or Sobel operator
2. For pixels near edges, apply small random displacement
3. Use seed for reproducibility
4. Intensity parameter controls displacement magnitude

### 7.2 Implementation

1. Add `--edge-roughen <INTENSITY>` CLI flag (0.0-1.0, default: 0.0 = disabled)
2. Add `edge_roughen_intensity: f32` to RenderOptions
3. Implement `apply_edge_roughening(pixmap: &mut Pixmap, intensity: f32, seed: u64)`:
   - Scan for edge pixels (alpha gradient > threshold)
   - Apply controlled random displacement to nearby pixels
   - Use LCG random (same as noise) for reproducibility

4. Add to WASM/Python bindings
5. Add tests

### 7.3 Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| edge_roughen | f32 | 0.0 | Edge displacement intensity (0=off, 1=max) |

## Phase 8: Duotone Mode

**Goal:** Add duotone color mode that maps grayscale to two colors.

### 8.1 Design

Duotone maps the luminance of each pixel to a gradient between two colors:
- Dark pixels -> shadow color
- Light pixels -> highlight color
- Mid-tones -> blend between the two

### 8.2 Implementation

1. Add `--duotone <SHADOW,HIGHLIGHT>` CLI flag (e.g., `--duotone "#1a1a2e,#edf2f4"`)
2. Add `ColorMode::Duotone { shadow: [u8; 3], highlight: [u8; 3] }` variant
3. Implement duotone mapping:
   ```
   luminance = 0.299*R + 0.587*G + 0.114*B (ITU-R BT.601)
   t = luminance / 255.0
   output_r = shadow_r + t * (highlight_r - shadow_r)
   output_g = shadow_g + t * (highlight_g - shadow_g)
   output_b = shadow_b + t * (highlight_b - shadow_b)
   ```

4. Parse duotone colors from CLI (comma-separated hex)
5. Add to WASM/Python bindings
6. Add tests

### 8.3 Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| duotone | String | None | Two hex colors: "shadow,highlight" |

## Success Criteria

1. `cargo test --workspace` passes (excluding platform-specific examples)
2. All crates use vruffr-* naming
3. Edge roughening produces visible organic edge effects
4. Duotone produces proper two-color output
5. All features work in CLI, WASM, and Python bindings
