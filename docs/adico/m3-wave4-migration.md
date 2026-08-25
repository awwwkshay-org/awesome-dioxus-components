# M3 Wave 4 migration (task 4.5)

Status: complete

This records task 4.5: the collection/selection/navigation-dependent
migration batch from the
[M3 migration queue](m3-migration-queue.md#wave-4--collectionselectionnavigation-dependent-task-45) —
`combobox`, `calendar`, `date-picker`, and `sidebar` — installed into
`registry/` with source metadata, docs metadata, verified installation
fixtures, and representative keyboard/typeahead/selection browser tests.

## A real gap in the M3 plan: Wave 2 was never migrated

The migration queue defines five waves. `tasks.md`'s M3 section only has
checkboxes for `4.3` (Wave 1), `4.4` (Wave 3 — overlay/layer), `4.5` (Wave 4 —
this batch), and `4.6` (Wave 5 — extras). There is no task for Wave 2
(accordion, alert-dialog, aspect-ratio, avatar, checkbox, collapsible, label,
progress, radio-group, scroll-area, slider, switch, tabs, toast, toggle,
toggle-group). This batch only pulled in `separator` — one Wave 2 item —
because `sidebar` has a hard dependency on it; the other sixteen Wave 2 items
remain unmigrated. This needs its own follow-up pass before M3 can be
considered complete; it is not silently folded into this record.

## Scope decisions

- **combobox**: reuses the collection/selectable/listbox primitive
  infrastructure already imported for `select`, needing only a Rust
  2024 `+ use<>` lifetime-capture fix. Registry facade is a pure re-export
  (`ComboboxOption` is generic), matching `select`'s precedent.
- **calendar**: 25+ composable parts. A full Tailwind restyle up front would
  be disproportionate to this migration pass (matching `select`'s "many
  parts, re-export unstyled" precedent) — full default styling is M4's job.
- **date-picker**: composes the owned `calendar` and `popover` primitives
  internally (a Cargo-level composition, the same relationship `sheet`
  already has with `dialog` — no registry dependency edge on `calendar`).
  Also a pure re-export.
- **sidebar**: has no upstream `primitives/src` file at all — see
  `provenance/records/adico-primitives-wave4-collection.json`'s notices.
  Upstream ships it only as a styled preview component (CSS modules, a
  `document::eval` mobile-viewport hook of the exact long-lived/repeatedly-
  firing kind found non-functional during Wave 3 testing). Implemented
  directly as `registry/ui/sidebar.rs`: one CSS-driven collapsible layout
  (`data-state`/`data-collapsible` + Tailwind transitions) instead of a
  silently-broken JS-detected mobile mode. Real viewport-driven mobile sheet
  behavior is deferred to M4/M5.

## A real bug found and fixed by testing in the browser

Playwright caught what `cargo check`/doctests cannot: mounting a `Calendar`
with `CalendarHeader`/`CalendarGrid` children — exactly as this crate's own
primary doc example shows — panicked at first render (a wasm `unreachable`
trap, no page content). Bisecting by rebuilding a live `dx serve` fixture
with progressively smaller compositions isolated it to `CalendarMonthTitle`
alone: it reads a `CalendarViewContext` that only `CalendarView` provides.
`CalendarView`'s own doc comment describes rendering "one... for each month
you want visible," reading as optional for a single month — it is not:
`CalendarGrid` needs it too, and omitting it is a hard panic, not a graceful
default.

Eleven of this crate's own doc examples (inherited verbatim from upstream,
including the primary `Calendar` example) omitted this wrapper. Since
doctests here are compiled but never executed, nothing caught this until an
actual browser mounted the component. This is upstream's own latent bug —
not something this fork's adaptation introduced.

**Fixed**: all eleven affected doc examples now nest their content inside
`CalendarView`, and `CalendarView`'s own doc comment carries an explicit
warning. The wave4-consumer fixture and Playwright suite below reflect the
corrected usage.

## Verification

```
cargo xtask registry build
cargo xtask registry validate
cargo xtask provenance check
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo check -p adico-primitives --locked --no-default-features --features web --target wasm32-unknown-unknown
cargo check -p adico-primitives --locked --features desktop
cargo build -p adico-cli --locked
(cd tests/installation/wave4-consumer && \
  ../../../target/debug/adico init && \
  ../../../target/debug/adico add combobox calendar date-picker sidebar && \
  cargo build && \
  cargo check --target wasm32-unknown-unknown)
(cd tests/installation/wave4-consumer && dx serve --platform web) &
(cd tests/playwright && ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:8080 npm run test:wave4)
```

| Check | Result |
| --- | --- |
| `cargo xtask registry build`/`validate` | 22 item payload(s) (18 existing + 4 new) |
| `cargo xtask provenance check` | 3 imported record(s), 34 source unit(s) |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed (includes `wave4_batch_add_installs_every_collection_item_once`) |
| `adico-primitives` native / web+wasm32 / desktop | all passed, 23 unit tests + 41 doctests |
| `wave4-consumer`: `adico init && adico add …` | plan reported all 4 `@adico/*` addresses plus shared `@adico/cn`; `adico add complete.` |
| `wave4-consumer`: `cargo build` (native) / `cargo check --target wasm32-unknown-unknown` | both succeeded, no warnings under `RUSTFLAGS=-D warnings` |
| `wave4.spec.ts` (5 tests, live `dx serve` + Playwright + axe) | all passed: Combobox typeahead/keyboard selection, Calendar arrow-key navigation and selection, DatePicker popover+embedded Calendar, Sidebar open/close toggle, zero critical axe violations |

Every command above is offline-safe except `wave4-consumer`'s `cargo build`
and the `dx serve`/Playwright pair, matching the existing
`*-consumer`/Playwright fixtures.

## Verification satisfied

Task 4.5's own verification requirement — "verify representative
keyboard/typeahead/selection tests and consumer builds" — is satisfied by
`wave4.spec.ts` running against a live `dx serve` of `wave4-consumer`
(combobox typeahead, calendar arrow-key selection, date-picker composition,
sidebar toggle), plus the offline
`wave4_batch_add_installs_every_collection_item_once` integration test for
installation mechanics on every `cargo test --workspace` run.
