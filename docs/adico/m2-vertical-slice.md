# M2 registry/CLI vertical-slice acceptance record

Status: accepted for implementation

## Pipeline proven end to end

M2 proves the complete `registry source → metadata → resolution → CLI →
Cargo/module/CSS → consumer build/runtime` pipeline for **Button, Dialog, and
Select** — the M1-selected vertical slice — from both the embedded official
registry (`@adico`) and a configured organization registry (`@awwwkshay`).

```text
registry/registry.json (source + metadata)
  -> adico-registry-core (resolution, dependency graph, checksums)
  -> adico-cli (plan, Cargo.toml edits, mod.rs marker regions, theme CSS)
  -> consumer project (installed source, adico.lock, components.json)
  -> build/runtime (web, desktop, SSR + hydration)
```

## Official registry: Button, Dialog, Select

| Stage | Evidence |
| --- | --- |
| Source/metadata | [`../../registry/registry.json`](../../registry/registry.json), [`../../registry/ui`](../../registry/ui) |
| Resolution | `packages/adico-registry-core` unit tests (dependency graph, shared-dependency dedup, checksum validation) |
| CLI plan/apply | `packages/adico-cli/tests/cli_integration.rs` (multi-item add, repeated add, shared dependencies, Cargo conflicts, file conflicts, malformed modules, dry-run, source-lock refresh) |
| Consumer build (web) | [`../../tests/installation/button-consumer`](../../tests/installation/button-consumer), [`dialog-consumer`](../../tests/installation/dialog-consumer), [`select-consumer`](../../tests/installation/select-consumer), [`../../examples/basic-spa`](../../examples/basic-spa) — each installed and built through the real `adico` binary |
| Browser interaction/accessibility | [`../../tests/playwright/dialog.spec.ts`](../../tests/playwright/dialog.spec.ts), [`select.spec.ts`](../../tests/playwright/select.spec.ts) |
| SSR/hydration | [`../../examples/basic-ssr`](../../examples/basic-ssr) (server-feature and web-feature builds), [`../../tests/playwright/fullstack.spec.ts`](../../tests/playwright/fullstack.spec.ts) — server-rendered HTML for all three components, client hydration confirmed by real interaction (Dialog opens, Select lists options) with zero console errors |
| Desktop | Removed by the `consolidate-examples` change (2026-08-30); `examples/desktop` no longer exists. Native desktop-target build validation has no fixture and is a named, tracked gap (`parity.json` records `desktop: passed=false` for Button with a note; Dialog/Select desktop installs were already a tracked gap for the unrelated per-platform feature-selection reason below) |
| Parity record | [`../../parity.json`](../../parity.json) — first entries for `button`, `dialog`, `select` (`status: in_progress`); `source`, `cli`, `cargo`, `web`, `examples`, `keyboard`, `accessibility`, and `ssrHydration` pass with evidence; `api`, `visual`, `variants`, `states`, `darkMode`, `rtl`, `responsive`, and `docs` are explicitly deferred to M4/M5 with a note, not silently omitted |

## Organization registry: Awwwkshay

| Stage | Evidence |
| --- | --- |
| Source/metadata | [`../../tests/installation/awwwkshay-consumer/awwwkshay-registry`](../../tests/installation/awwwkshay-consumer/awwwkshay-registry) — a real, checked-in `@awwwkshay` registry with a `card` item that explicitly cross-registry-depends on `@adico/cn` |
| Resolution (local and static-HTTPS parity) | `packages/adico-registry-core` `awwwkshay_registry_fixture_resolves_identically_over_local_and_https_sources` — the same checked-in manifest resolves identically whether configured as a local path or loaded through the static-HTTPS transport |
| CLI plan/apply, mixed bare + explicit resolution | `packages/adico-cli/tests/cli_integration.rs` `bare_company_default_and_explicit_official_items_install_together` — a bare `card` request resolves through the configured `@awwwkshay` default while an explicit `@adico/button` request resolves through the official registry in the same `adico add` |
| Consumer build | [`../../tests/installation/awwwkshay-consumer`](../../tests/installation/awwwkshay-consumer) — `@awwwkshay` set as `defaultRegistry`, `adico add card @adico/button` installs Card, Button, and the transitively-required `cn`, and the fixture's `cargo build` succeeds against real Dioxus |

## Reproducing this record

From the repository root:

```sh
cargo fmt --all --check
cargo check --workspace --locked
cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings
cargo test --locked --workspace

# Official-registry vertical slice, rebuilt through the real CLI binary:
cargo build -p adico-cli --locked
(cd tests/installation/button-consumer && ../../../target/debug/adico init && ../../../target/debug/adico add button && cargo build)
(cd tests/installation/dialog-consumer && ../../../target/debug/adico init && ../../../target/debug/adico add dialog && cargo build)
(cd tests/installation/select-consumer && ../../../target/debug/adico init && ../../../target/debug/adico add select && cargo build)

# Organization-registry vertical slice:
(cd tests/installation/awwwkshay-consumer && ../../../target/debug/adico init --default-registry @awwwkshay --registry @awwwkshay=awwwkshay-registry && ../../../target/debug/adico add card @adico/button && cargo build)
cargo xtask registry validate --source tests/installation/awwwkshay-consumer/awwwkshay-registry/registry.json

# SSR/hydration (requires the Dioxus CLI, `dx`):
cargo check -p adico-example-basic-ssr --no-default-features --features server --locked
cargo check -p adico-example-basic-ssr --no-default-features --features web --target wasm32-unknown-unknown --locked
(cd examples/basic-ssr && dx serve --platform web) &
(cd tests/playwright && ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:8080 npm run test:fullstack)

# Desktop: examples/desktop was removed by the consolidate-examples change
# (2026-08-30); native desktop-target build validation has no fixture and is
# a named, tracked gap (see parity.json).

openspec validate build-adico-component-ecosystem --strict
```

Every command above is offline-safe except the `cargo build`/`cargo check`
invocations that resolve real `dioxus`/`adico-primitives` crates for the first
time on a machine without a warm local registry cache; normal CI and repeated
local runs use the already-resolved `Cargo.lock`.

## M2 acceptance evidence

This record is accepted when the vertical-slice pipeline above passes for
Button, Dialog, Select, and the Awwwkshay organization registry, and the
commands in this file are reproducible from a clean checkout.

| Evidence | Artifact |
| --- | --- |
| Registry manifest schema, resolution, install planning | `packages/adico-registry-core` (20 unit tests) |
| CLI `init`/`add`/`list`/`view` | `packages/adico-cli` (unit tests + 10-case `cli_integration.rs`) |
| Official vertical-slice consumer fixtures | `tests/installation/{button,dialog,select}-consumer`, `examples/basic-spa` |
| Organization vertical-slice consumer fixture | `tests/installation/awwwkshay-consumer` |
| SSR/hydration fixture | `examples/basic-ssr`, `tests/playwright/fullstack.spec.ts` |
| Desktop fixture | None — removed by `consolidate-examples` (2026-08-30); tracked as a named gap in `parity.json` |
| Parity tracking | [`../../parity.json`](../../parity.json) and [`parity.md`](parity.md) |
| Validation matrix | [`../validation.md`](../validation.md) |
