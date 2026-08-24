# M3 Wave 1 migration (task 4.3)

Status: complete

This records task 4.3: the first independent migration batch from the
[M3 migration queue](m3-migration-queue.md#wave-1--zero-new-primitive-reuses-only-what-m2-already-owns) —
`badge`, `card`, `input`, `item`, `pagination`, `skeleton`, `textarea`, and
`sheet` — installed into `registry/` with source metadata, docs metadata, and
verified installation fixtures. Per task 4.2's record, none of these items
needed a new `adico-primitives` import; `sheet` composes the already-owned
`dialog` primitive directly.

## What changed

- `registry/ui/{badge,card,input,item,pagination,skeleton,textarea,sheet}.rs`:
  source-owned styled components, matching Button's minimal-props convention
  (no variant matrix yet — that is M4's job, not M3's).
- `registry/registry.json`: one entry per item (files/checksum,
  `registryDependencies`, `cargoDependencies`, `style`, `moduleExports`,
  `documentation`). `skeleton` intentionally omits the `cn` registry
  dependency and the `cn` style utility, per the migration queue's note that
  its classes are static enough not to need the composition helper.
- `packages/adico-cli/src/main.rs`: added the required `include_bytes!` match
  arm for each new file in `ConfiguredRegistryReader::read` (the CLI embeds
  official registry source through an explicit per-file allowlist, not a
  directory scan) and extended
  `discovery_uses_default_and_explicit_configured_sources_without_mutation`'s
  expected official item list from 4 to 12 addresses.
- `packages/adico-cli/tests/cli_integration.rs`: added
  `wave1_batch_add_installs_every_migrated_item_once`, installing all 8 items
  in one `adico add` call and asserting the plan report, installed files,
  single shared `cn` lock entry, and the `adico-primitives` Cargo edit that
  `sheet` requires.
- `tests/installation/wave1-consumer`: a new consumer-style fixture (mirrors
  `button-consumer`/`dialog-consumer`), installed and built through the real
  `adico` binary against real Dioxus, exercising every Wave 1 item together
  (`Card` wrapping `Badge`, `Input`, `Textarea`, `Skeleton`, an `ItemGroup`,
  a `Pagination` control cluster, and a `Sheet` with signal-driven
  `open`/`on_open_change`, matching Dialog's controlled-state pattern).

Deliberately deferred to task 4.7 (not part of 4.3's scope): `parity.json`
entries for these 8 items. `parity.json` currently only tracks the M2
vertical slice (`button`, `dialog`, `select`); task 4.7 is where every M3
migration batch gets its parity-manifest sweep, after batches 4.3–4.6 have
all landed.

## Verification

```
cargo xtask registry build
cargo xtask registry validate
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build -p adico-cli --locked
(cd tests/installation/wave1-consumer && \
  ../../../target/debug/adico init && \
  ../../../target/debug/adico add badge card input item pagination skeleton textarea sheet && \
  cargo build && \
  cargo check --target wasm32-unknown-unknown)
```

| Check | Result |
| --- | --- |
| `cargo xtask registry build` | 12 item payload(s) generated (4 existing + 8 new), deterministic |
| `cargo xtask registry validate` | passed, 12 item payload(s) in `@adico` |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed (includes the new `wave1_batch_add_installs_every_migrated_item_once` integration test) |
| `wave1-consumer`: `adico init && adico add …` | plan reported all 8 `@adico/*` addresses plus the shared `@adico/cn` dependency; `adico add complete.` |
| `wave1-consumer`: `cargo build` (native) | succeeded against real `dioxus =0.7.9` and `adico-primitives` (path-patched), no warnings under `RUSTFLAGS=-D warnings` |
| `wave1-consumer`: `cargo check --target wasm32-unknown-unknown` | succeeded, confirming `sheet`'s `adico-primitives` `web` feature compiles for the browser target |

Every command above is offline-safe except `wave1-consumer`'s `cargo build`,
which resolves real crates.io dependencies exactly like the other
`tests/installation/*-consumer` fixtures.

## Verification satisfied

Task 4.3's own verification requirement — "verify each can be installed by
`adico add` into a clean consumer fixture" — is satisfied by the
`wave1-consumer` fixture above (all 8 items, one clean fixture) and by the
`wave1_batch_add_installs_every_migrated_item_once` integration test, which
covers the same batch offline on every `cargo test --workspace` run.
