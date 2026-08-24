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

| Surface | Required check when changed | Status in M0 |
| --- | --- | --- |
| Registry schema/resolution | Registry-core unit tests and deterministic build validation | Harness added in M2 |
| CLI installation | Installation fixtures plus conflict/idempotency tests | Harness added in M2 |
| Web/wasm | `cargo check --target wasm32-unknown-unknown` for affected Dioxus apps | Required once components exist |
| SSR/hydration | Server-feature build and hydration fixture | Required for affected components |
| Desktop | Explicit desktop feature check/smoke test on a supported runner | Required for affected components |
| Browser interaction/a11y | Playwright keyboard and accessibility suite | Harness selected in M0, added in M2 |
| Visual parity | Approved visual-regression suite | Harness selected in M0, added incrementally |
| External registry | Local/static-HTTPS source and lock-refresh fixtures | Required in M2 |

Reports must name target checks that did not run and the reason; skipped checks
are not parity passes.
