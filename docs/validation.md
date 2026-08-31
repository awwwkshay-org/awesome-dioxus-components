# adico validation matrix

This matrix distinguishes checks that every change must run from platform and
tooling checks that become required when a change touches their surface.

## Required baseline

Run these from the repository root for every Rust/package change:

```sh
cargo fmt --all --check
cargo check --workspace --locked
cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings
cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask
openspec validate build-adico-component-ecosystem --strict
```

The pre-existing API/UI template retains its documented PostgreSQL validation
until its retirement is explicitly approved.

## Surface-specific validation

| Surface | Required check when changed | Status after M2 |
| --- | --- | --- |
| Registry schema/resolution | Registry-core unit tests and deterministic build validation | Harness added in M2; see [`adico/m2-vertical-slice.md`](adico/m2-vertical-slice.md) |
| CLI installation | Installation fixtures plus conflict/idempotency tests | Harness added in M2 (`packages/adico-cli/tests/cli_integration.rs`, `tests/installation/`) |
| Web/wasm | `cargo check --target wasm32-unknown-unknown` for affected Dioxus apps | Proven for Button/Dialog/Select in M2 |
| SSR/hydration | Server-feature build and hydration fixture | Proven for Button/Dialog/Select in M2 (`tests/playwright/fullstack.spec.ts`); required for later components |
| Desktop / mobile (native) | Explicit `native`-feature check/smoke test on a supported runner. This row covers Android and iOS as well as desktop: Dioxus's mobile renderer embeds the same platform WebView family (`wry`/`tao`) as desktop rather than rendering native OS widgets, so `adico-primitives`' `native` Cargo feature and its `#[cfg(any(feature = "web", feature = "native"))]` gates cover both (see design.md §8a) — there is no separate `mobile` feature or gate. | `examples/desktop` (Button's only evidence) was removed by `consolidate-examples` (2026-08-30); native desktop-target validation has no fixture and is a tracked gap for all of Button/Dialog/Select (`parity.json`'s completion-tracking manifest was removed 2026-08-31; this gap is now recorded here instead). Android/iOS have the same gap: no fixture has ever been built for either, so mobile is untested, not verified-and-passing. |
| Browser interaction/a11y | Playwright keyboard and accessibility suite | Harness selected in M0, added in M2 (`tests/playwright/dialog.spec.ts`, `select.spec.ts`) |
| Visual parity | Approved visual-regression suite | Harness selected in M0, added incrementally starting M4 |
| External registry | Local/static-HTTPS source and lock-refresh fixtures | Proven in M2 (`tests/installation/awwwkshay-consumer`, registry-core local/HTTPS parity test) |

Reports must name target checks that did not run and the reason; skipped checks
are not parity passes.
