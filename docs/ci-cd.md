# CI and delivery

`ci.yml` validates Rust formatting, the workspace, Clippy across the five core
crates, `cargo test --locked --workspace`, and the eight offline `adico-xtask`
staleness gates (`registry validate`, `provenance check`, `primitive-compat
check`, `component-compat check`, `primitive-usage check`, `styling-usage
check`, `playground-controls check`, `prop-parity check`). See
[`development.md`](development.md) for what each gate checks and
[`validation.md`](validation.md) for the full per-surface matrix, including
surfaces with an authored harness that CI does not yet run (wasm32,
Playwright, Windows).

`cd.yml` and a CLI release workflow are being built to support the public
v0.1.0 release: `cd.yml` will build the hosted site image (`apps/docs` +
`apps/playground` + the generated registry, served by nginx), push it to
`ghcr.io/awwwkshay-org/adico-web`, and open a deployment PR against the
`awwwkshay-infra` repository. A separate tag-triggered release workflow will
build native `adico-cli` binaries, publish GitHub Release assets, update the
`awwwkshay-org/homebrew-tap` formula, and publish
`adico-cli`/`adico-primitives`/`adico-registry-core` to crates.io. Until both
land, update this file to describe what actually runs.

Network-dependent upstream synchronization (`cargo xtask catalog fetch`)
remains an explicit maintainer action; ordinary CI uses checked-in snapshots
and never touches the network.
