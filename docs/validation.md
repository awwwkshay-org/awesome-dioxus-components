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
| Desktop | Explicit desktop feature check/smoke test on a supported runner | Proven for Button in M2 (`examples/desktop`); Dialog/Select desktop feature-selection is a tracked gap, see `parity.json` |
| Browser interaction/a11y | Playwright keyboard and accessibility suite | Harness selected in M0, added in M2 (`tests/playwright/dialog.spec.ts`, `select.spec.ts`) |
| Visual parity | Approved visual-regression suite | Harness selected in M0, added incrementally starting M4 |
| External registry | Local/static-HTTPS source and lock-refresh fixtures | Proven in M2 (`tests/installation/awwwkshay-consumer`, registry-core local/HTTPS parity test) |

Reports must name target checks that did not run and the reason; skipped checks
are not parity passes.
