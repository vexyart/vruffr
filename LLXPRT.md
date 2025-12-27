# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Standard Rust Workflow

```bash
# Format code
cargo fmt

# Lint with strict warnings
cargo clippy -- -D warnings

# Run tests
cargo test

# Build release binary
cargo build --release

# Run the tool (once implemented)
cargo run -- input.svg output.png
```

### Full Quality Check

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

### Working with External Dependencies

External libraries in `ext/` are git submodules or vendored code:
- Do not modify code in `ext/` directly
- Reference them via path dependencies in Cargo.toml
- To update: navigate to the specific ext/ directory and git pull

### Testing Strategy

Per project guidelines (from IDEA.md):
- Unit tests for each function
- Edge cases: empty files, invalid SVG, huge files
- Integration tests: end-to-end SVG → PNG conversion
- Functional examples in `examples/` directory
- Test script: `./test.sh` (create if needed)

## Key Constraints

1. **Minimalism:** Favor existing libraries over custom code
2. **Rust Standards:**
   - Error handling with `Result<T, E>`, using `thiserror` (libs) or `anyhow` (apps)
   - Avoid `panic!` except for unrecoverable errors
   - Minimize `unsafe` code
3. **Testing:** ≥80% coverage target, focus on edge cases
4. **File Size:** Functions ≤20 lines, files ≤200 lines where possible
5. **Documentation:** Update WORK.md, PLAN.md, TODO.md, CHANGELOG.md, DEPENDENCIES.md

## Common Patterns

### Error Handling
```rust
use anyhow::{Context, Result};

fn process_svg(path: &Path) -> Result<Image> {
    let svg_data = std::fs::read(path)
        .context("Failed to read SVG file")?;
    // ...
}
```

### Dependency References
In main Cargo.toml, reference ext/ libraries as path dependencies:
```toml
[dependencies]
roughr = { path = "ext/rough-rs/roughr" }
rough_vello = { path = "ext/rough-rs/rough_vello" }
vello = { path = "ext/vello/vello" }
# or use published versions if compatible
```

## Version Compatibility

Key version constraints from vello_svg:
- vello 0.6 → usvg 0.45
- Monitor compatibility between rough_vello (uses vello 0.5) and latest vello

## Project Documentation

Required files per development guidelines:
- **README.md**: Purpose, installation, usage (≤200 lines)
- **PLAN.md**: Architecture decisions, future goals
- **TODO.md**: Flat checklist with status markers `[ ]` `[~]` `[x]` `[-]` `[!]`
- **WORK.md**: Work log with reasoning, test results, next steps
- **CHANGELOG.md**: Release notes
- **DEPENDENCIES.md**: Package choices with justification

## Reference Materials

Analysis documents in `ref/`:
- `ref/101gem.md`, `ref/102gpt.md`, `ref/103cla.md`, `ref/104sci.md`: Research and analysis
- Review these for additional context on implementation approaches
