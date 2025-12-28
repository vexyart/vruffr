# TODO.md - vruffr Task List

## Phase 2: Crate Renaming - COMPLETE

### 2.1 Rename roughr -> vruffr-core
- [x] Update roughr/Cargo.toml name field
- [x] Update rough_tiny_skia/Cargo.toml dependency
- [x] Update rough_piet/Cargo.toml dependency
- [x] Update rough_vello/Cargo.toml dependency
- [x] Update cli/Cargo.toml dependency
- [x] Update all `use roughr::` -> `use vruffr_core::`

### 2.2 Rename rough_tiny_skia -> vruffr-skia
- [x] Update rough_tiny_skia/Cargo.toml name field
- [x] Update cli/Cargo.toml dependency
- [x] Update all `use rough_tiny_skia::` -> `use vruffr_skia::`

### 2.3 Update Documentation
- [x] Update CLAUDE.md references
- [x] Update WORK.md references

### 2.4 Verification
- [x] Run cargo build --workspace
- [x] Run cargo test -p vruffr-core -p vruffr-skia -p vruffr-cli
- [x] Verify CLI works

## Phase 7: Edge Roughening Filter

- [ ] Add `--edge-roughen` CLI flag (0.0-1.0)
- [ ] Add edge_roughen field to RenderOptions
- [ ] Implement apply_edge_roughening function in cli/src/lib.rs
- [ ] Add edge_roughen to WASM bindings
- [ ] Add edge_roughen to Python bindings
- [ ] Add tests for edge roughening

## Phase 8: Duotone Mode

- [ ] Add `--duotone` CLI flag (shadow,highlight hex colors)
- [ ] Add ColorMode::Duotone variant
- [ ] Implement duotone color mapping in apply_color_mode
- [ ] Parse duotone colors from CLI argument
- [ ] Add duotone to WASM bindings
- [ ] Add duotone to Python bindings
- [ ] Add tests for duotone mode
