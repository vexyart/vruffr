# TODO.md - vruffr Task List

## Immediate

- [ ] We should be able to use / build / test / run Cairo on our macOS machine.
- [x] Read ./issues/102.md and into ./PLAN.md and ./TODO.md and ./SPEC.md incorporate all these things I postulate.

## Phase 1: Foundation Cleanup - COMPLETE

- [x] Fix rough_tiny_skia panic at line 281 (opset_to_shape unwrap)
- [x] Change opset_to_shape to return Option<Path>
- [x] Update SkiaOpset.ops field to Option<Path>
- [x] Update draw() method to skip None paths
- [x] Add tests for empty/degenerate paths in rough_tiny_skia
- [x] Remove rough_iced from workspace members
- [x] Remove rough_plotters_svg from workspace members
- [x] Delete rough_iced directory
- [x] Delete rough_plotters_svg directory
- [x] Run cargo test --lib --tests to verify baseline

## Phase 1.5: Preprocessing Features - COMPLETE

### Duplicate Path Filtering - COMPLETE

- [x] Create path signature struct (bbox, length, vertex_count, command_hash)
- [x] Implement path signature computation function
- [x] Implement path equivalence test with epsilon tolerance
- [x] Create deduplication filter that groups paths by signature
- [x] Implement stroke/fill merge strategy for duplicate groups
- [x] Add deduplicate field to SketchOptions
- [x] Add dedup_epsilon field to SketchOptions
- [x] Add --deduplicate CLI flag
- [x] Add --dedup-epsilon CLI flag
- [x] Add unit tests for path signature computation
- [x] Add unit tests for deduplication with various cases
- [x] Test with SVGs containing known duplicate paths (covered by unit tests)

### Adaptive Roughness - COMPLETE

- [x] Add adaptive_strength field to Options (roughr/src/core.rs)
- [x] Add reference_size field to Options (roughr/src/core.rs)
- [x] Implement characteristic_size calculation (sqrt of bbox area)
- [x] Implement adaptive scaling function (size_ratio ^ power)
- [x] Integrate adaptive scaling into CLI (cli/src/lib.rs)
- [x] Add --adaptive-strength CLI flag
- [x] Add --reference-size CLI flag
- [x] Add unit tests for adaptive scaling function (7 tests)
- [x] Test with mixed-size SVG elements
- [x] Document adaptive roughness in README

## Phase 2: Rename & Restructure

- [ ] Create crates/ directory structure
- [ ] Rename roughr -> vruffr-core
- [ ] Rename points_on_curve -> merge into vruffr-path
- [ ] Rename svg_path_ops -> merge into vruffr-path
- [ ] Rename rough_tiny_skia -> vruffr-skia
- [ ] Rename rough_piet -> vruffr-piet
- [ ] Rename rough_vello -> vruffr-vello
- [ ] Update workspace Cargo.toml paths
- [ ] Update all internal dependency references
- [ ] Update all use statements across crates
- [ ] Run cargo test --workspace to verify rename

## Phase 3: Integrate CLI - IN PROGRESS

- [x] Move extra/src/ to cli/
- [x] Create cli/Cargo.toml with path dependencies
- [x] Switch from crates.io to workspace path deps
- [x] Rename CLI binary to vruffr
- [x] Remove catch_unwind panic handlers from cli
- [ ] Add backend selection CLI flag
- [ ] Add SVG output format flag
- [ ] Add SVGpatch output mode
- [x] Verify all test-data renders without panics
- [ ] Adapt extra/test-data.sh for new structure
- [ ] Adapt extra/build.sh for new structure

## Phase 4: Documentation

- [x] Rewrite README.md for vruffr project
- [x] Add installation instructions
- [x] Add CLI usage examples
- [x] Add library usage examples
- [x] Write SPEC.md with technical specification
- [ ] Add Rust doc comments to all public APIs
- [x] Create examples/ directory with working examples
- [x] Update CLAUDE.md for new project structure
- [x] Create DEPENDENCIES.md listing all deps with justification
- [x] Create CHANGELOG.md with initial entry

## Phase 5: Python & WASM (Future)

- [ ] Set up python/ directory with maturin/PyO3
- [ ] Create Python wrapper for core functionality
- [ ] Create Fire-based Python CLI
- [ ] Set up wasm/ directory with wasm-pack
- [ ] Create browser demo for WASM
- [ ] Add Python package tests
- [ ] Add WASM integration tests

## Phase 6: Advanced Features (Future)

### Color Manipulation

- [ ] Add monochrome color mode
- [ ] Add grayscale mode
- [ ] Add sepia mode
- [ ] Add duotone mode

### Post-Processing Filters

- [ ] Add paper texture overlay filter
- [ ] Add noise/grain post-processing
- [ ] Add edge roughening filter
- [ ] Research and prototype additional sketch effects

## Cleanup & Polish

- [x] Remove extra/src/ after CLI migration
- [x] Clean up old skesvg artifacts from extra/
- [x] Update .gitignore for new structure
- [~] Run cargo clippy -- -D warnings (svg_path_ops has pre-existing lint issues)
- [x] Run cargo fmt --check
- [x] Ensure all tests pass
- [ ] Tag initial vruffr release
