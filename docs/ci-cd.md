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

`release.yml` is the CLI release workflow: it triggers on a `v[0-9]+.[0-9]+.[0-9]+`
tag push, builds `adico` on native runners for macOS (arm64/x64), Linux
(arm64/x64), and Windows (x64) — no cross-compilation and no musl target (the
Tailwind standalone CLI `adico` downloads at runtime is only published for
glibc/macOS/Windows hosts; see `packages/adico-cli/src/css_build.rs`) —
publishes the archives and SHA-256 checksums as GitHub Release assets via
`gh release create --verify-tag`, and then updates the `adico` formula in
`awwwkshay-org/homebrew-tap` using `.github/scripts/render_homebrew_formula.py`
(unit-tested by `ci.yml`). It does not publish to crates.io; that remains a
separate, not-yet-scheduled workstream because `adico-primitives` and
`adico-registry-core` must publish ahead of `adico-cli` with version
requirements instead of the workspace's local `[patch.crates-io]` override.

A future `cd.yml` may build the hosted site image (`apps/web` + the
generated registry, served by nginx), push it to
`ghcr.io/awwwkshay-org/adico-web`, and open a deployment PR against the
`awwwkshay-infra` repository — not yet built. Update this file when it lands.

Network-dependent upstream synchronization (`cargo xtask catalog fetch`)
remains an explicit maintainer action; ordinary CI uses checked-in snapshots
and never touches the network.
