# WORK.md - vruffr Work Log

- The fly-january 0.2.1 strict build passed with zero warnings (10 themed pages
  and the standalone playground).
- `./test.sh quick` on Rust 1.100 passed 120 tests with seven ignored.
- `cargo clippy --all-targets -p points_on_curve` exited 101 on macOS. The
  tolerance example imports `piet_common::CairoRenderContext`, which is
  unavailable on this target.
- `cargo package --offline --no-verify -p vruffr-cli` exited 101 because the
  CLI's `vruffr-core` dependency lacks a version requirement.
- The last `./build.sh test` passed on Rust 1.86. On Rust 1.100 its broader
  Clippy check stopped in `dedup.rs` and `renderer.rs`.

See [PLAN.md](PLAN.md) for the three remaining tasks.
