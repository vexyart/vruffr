# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Structure

```
vruffr/
├── cli/                 # CLI tool (vruffr binary)
├── roughr/              # vruffr-core: Core sketch primitives
├── rough_tiny_skia/     # vruffr-skia: tiny-skia rendering backend
├── rough_piet/          # piet rendering backend
├── rough_vello/         # vello GPU backend
├── points_on_curve/     # Bezier curve utilities
├── svg_path_ops/        # SVG path manipulation
└── extra/               # Reference code (read-only)
```

## Development Commands

```bash
# Build and run CLI
cargo build --release
./target/release/vruffr input.svg -o output.png

# Run tests
cargo test -p vruffr-core -p vruffr-skia -p vruffr-cli

# Format and lint
cargo fmt
cargo clippy -p vruffr-core -p vruffr-skia -p vruffr-cli -- -D warnings

# Full quality check
cargo fmt --check && cargo test -p vruffr-core -p vruffr-skia -p vruffr-cli
```

Note: `svg_path_ops` has pre-existing clippy warnings from the original fork.

## Key Crates

| Crate | Purpose | Entry Point |
|-------|---------|-------------|
| `vruffr-cli` | CLI binary and library | `cli/src/main.rs` |
| `vruffr-core` | Core primitives, dedup module | `roughr/src/lib.rs` |
| `vruffr-skia` | Rendering to PNG via tiny-skia | `rough_tiny_skia/src/lib.rs` |

## Testing Strategy

- Unit tests in each crate's `src/` with `#[cfg(test)]`
- Integration tests: `cargo test -p vruffr-cli`
- Manual testing: `./target/release/vruffr extra/test-data/*.svg -o /tmp/out.png`

## Key Patterns

### Adaptive Roughness
```rust
// In cli/src/lib.rs
fn compute_effective_roughness(path: &usvg::Path, options: &SketchOptions) -> f32 {
    let size = (bbox.width() * bbox.height()).sqrt();
    let scale = (size / reference_size).powf(strength * 0.5);
    base_roughness * scale.clamp(0.2, 2.0)
}
```

### Option<Path> Pattern
```rust
// rough_tiny_skia returns Option<Path> to handle degenerate paths gracefully
fn opset_to_shape(&self, o: &OpSet<F>) -> Option<Path> {
    // Returns None for empty/degenerate paths instead of panicking
}
```

## Constraints

1. **Minimalism:** Favor existing libraries over custom code
2. **Error Handling:** Use `anyhow` for CLI, `thiserror` for libraries
3. **No Panics:** Return `Option` or `Result` instead
4. **Testing:** Cover edge cases (empty paths, degenerate geometry)
5. **Documentation:** Keep WORK.md, TODO.md, CHANGELOG.md updated

## Reference Materials

- `extra/` contains skesvg reference code (read-only, don't modify)
- `PLAN.md` has architecture decisions and phase roadmap
- `SPEC.md` has technical specification
