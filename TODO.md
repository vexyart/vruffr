# TODO.md - vruffr Task List

All planned phases complete. Project is production-ready at v2.0.7.

## Future Ideas (Optional)

- [x] Paper texture overlay
- [x] Crates.io metadata (versions aligned, ready for `cargo publish`)
- [ ] Extend CI to build/test the piet and vello backends. They compile clean at
      the library level now, but their dev-dependencies (piet-common/Cairo,
      bevy/wgpu, the `text2v` git dep) and examples need system packages the
      current ubuntu runner does not install. Add a dedicated job that provisions
      Cairo + a GPU/software rasterizer, then run `--all-targets` for these crates.
- [ ] Fix `points_on_curve/examples/tolerance.rs` (uses `piet_common::CairoRenderContext`)
      so `cargo clippy --all-targets -p points_on_curve` passes without Cairo,
      or gate the example behind a `cairo` feature.
- [ ] Wire `cargo publish` for `vruffr-core`, `vruffr-skia`, and `vruffr-cli` into
      `publish.sh` in dependency order (currently only publishes `vruffr-cli`).
