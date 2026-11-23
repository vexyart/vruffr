# WORK.md - vruffr Work Log

## Current Status

v2.0.6 production-ready. All phases complete.

### Quick Commands

`./build.sh docs` builds `src_docs/md/` into `docs/` with fly-january 0.1.1
in strict mode. The Pages workflow runs the same command.

```bash
cargo fmt --check && cargo clippy --workspace --lib -- -D warnings && cargo clippy -p vruffr-cli --all-targets -- -D warnings && cargo test -p vruffr-core -p vruffr-skia -p vruffr-cli
```

### Test Summary

- Theme migration: the strict build passed with zero warnings. It produced nine
  documentation pages, the 404 page and the original standalone playground.
- `./test.sh quick` passed on Rust 1.100 after correcting its crate names.
  `./build.sh test` passed on Rust 1.86. Each ran 120 passing tests with seven
  ignored. On Rust 1.100 the latter command stops on existing Clippy findings
  in `dedup.rs` and `renderer.rs`.

```
cargo test -p vruffr-core -p vruffr-skia -p vruffr-cli
# 120 tests passing
```

---

See `git log --oneline` for history.
