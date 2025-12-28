# Contributing to vruffr

## Quick Start

```bash
# Clone and build
git clone https://github.com/vexyart/vruffr.git
cd vruffr
cargo build

# Run tests
./test.sh quick

# Try it
./target/debug/vruffr examples/sag.svg -o /tmp/test.png
```

## Development Workflow

### Before Making Changes

1. Run `./test.sh quick` to verify baseline
2. Create a feature branch: `git checkout -b feature-name`

### Making Changes

1. Write tests first (TDD)
2. Make minimal, focused changes
3. Run `./test.sh quick` frequently
4. Update docs if adding features

### Before Committing

```bash
# Format, lint, test
./test.sh quick

# For larger changes
./test.sh full
```

## Project Structure

| Directory | Purpose |
|-----------|---------|
| `cli/` | CLI tool and main library |
| `vruffr_core/` | Core sketch primitives |
| `vruffr_tiny_skia/` | CPU rendering backend |
| `examples/` | Example SVGs and code |
| `src_docs/` | MkDocs documentation |

## Code Style

- Follow existing patterns
- Use `cargo fmt` before committing
- Keep functions under 20 lines where possible
- Prefer `Option<T>` over panics

## Testing

```bash
# Quick check
./test.sh quick

# Full check (includes build + examples)
./test.sh full

# Run demos
./demo.sh all
```

## Adding Features

1. Add CLI flag in `cli/src/main.rs`
2. Add option to `SketchOptions` in `cli/src/lib.rs`
3. Implement in library code
4. Add tests
5. Update CLI help text
6. Update README if user-facing

## Commit Messages

```
Short summary (50 chars or less)

Longer description if needed. Explain what and why,
not how (the code shows how).
```

## Pull Requests

1. One feature or fix per PR
2. Include tests
3. Update relevant docs
4. Run `./test.sh full` before submitting

## Questions?

Open an issue at https://github.com/vexyart/vruffr/issues
