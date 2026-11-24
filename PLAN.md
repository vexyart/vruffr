---
this_file: PLAN.md
---

# Remaining work

## Test all Piet and Vello targets in CI

CI checks both libraries. Add an Ubuntu job with Cairo and a software GPU
driver for the examples and tests. Verify the job passes
`cargo test --all-targets -p vruffr-piet -p vruffr-vello`.

## Make the default points_on_curve example checks pass without Cairo

The tolerance example imports `piet_common::CairoRenderContext`, and
`piet-common` is an unconditional dev-dependency. Remove the Cairo dependency
from the default checks or gate the example and its dependency behind a
`cairo` feature. Verify `cargo clippy --all-targets -p points_on_curve`
on a runner without Cairo.

## Wire crates.io publishing for vruffr-core, vruffr-skia and vruffr-cli in dependency order

`publish.sh` publishes only the CLI. Its manifest has path-only dependencies
on the core and Skia crates. Add version requirements, then check
`cargo publish --dry-run -p <crate>` for each crate in the order above as its
dependencies become available in the registry.
