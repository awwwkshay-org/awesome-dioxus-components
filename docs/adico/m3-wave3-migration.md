# M3 Wave 3 migration (task 4.4)

Status: complete

This records task 4.4: the overlay/layer-dependent migration batch from the
[M3 migration queue](m3-migration-queue.md#wave-3--overlaylayer-dependent-task-44) —
`tooltip`, `popover`, `hover-card`, `dropdown-menu`, `context-menu`, and
`menubar` — installed into `registry/` with source metadata, docs metadata,
and verified installation fixtures, browser keyboard/focus/accessibility
coverage for each interactive item.

## The migration queue's portal assumption was wrong

The queue document speculated (from upstream inventory metadata alone, not
source) that all six items depend on `src/portal.rs` and a JS positioning
engine. Reading the actual pinned upstream source (`bf007c15d0cf4d0…`)
showed that's false: only `toast.rs` uses the portal upstream. All six Wave
3 items position purely through `data-side`/`data-align` attributes plus
consumer-supplied CSS — no portal, no measurement JS. The new crate-root
`ContentSide`/`ContentAlign` enums (in `packages/adico-primitives/src/lib.rs`)
are the only new shared primitive infrastructure this wave needed; task
4.2-style primitive extension for this wave is otherwise just the six ported
modules themselves.

## What changed

- `packages/adico-primitives/src/{tooltip,popover,hover_card,dropdown_menu,context_menu,menubar}.rs`:
  forked from upstream, provenance-tracked in
  `provenance/records/adico-primitives-wave3-overlays.json`. Triggers drop
  upstream's `dioxus-attributes` proc-macro crate and `merge_attributes`
  helper (and `dropdown_menu`'s polymorphic `r#as` prop) in favor of the
  plain `..props.attributes` spread pattern `dialog` already established.
  `context_menu`'s two `document::eval` call sites (Safari visual-viewport
  correction, scroll suppression) are now behind this crate's
  `#[cfg(any(feature = "web", feature = "desktop"))]` target-gated pattern
  with SSR-safe fallbacks, and its long-press timer uses this crate's own
  `time::sleep` instead of `dioxus_sdk_time`.
- `registry/ui/{tooltip,popover,hover_card,context_menu,menubar}.rs`: styled
  shadcn-convention facades (single default visual style, no variant matrix
  — that's M4's job). `registry/ui/dropdown_menu.rs` is a pure re-export
  matching `select`'s precedent: `DropdownMenuItem` is generic, so styling
  is left to consumer composition rather than an opaque wrapper.
- `registry/registry.json`: six new entries. Registry item names stay
  kebab-case (`hover-card`, `dropdown-menu`, `context-menu`) while installed
  file/module names stay the Rust-valid snake_case the primitive already
  uses (`hover_card.rs`, etc.) — `target` is a free-form path in the schema,
  not required to match the item name.
- `packages/adico-cli/src/main.rs`: `include_bytes!` match arms for the six
  new files (the CLI embeds official registry source through an explicit
  per-file allowlist), plus `wave3_batch_add_installs_every_overlay_item_once`
  in `cli_integration.rs` and an updated 18-item expected-catalog list in
  `discovery_uses_default_and_explicit_configured_sources_without_mutation`.
- `tests/installation/wave3-consumer`: a real consumer fixture (adds
  `Dioxus.toml` alongside `Cargo.toml`/`src/main.rs`, needed for `dx serve`),
  installed and built through the real `adico` binary, composing all six
  items together.
- `tests/playwright/wave3.spec.ts`: real-browser coverage — served via
  `dx serve --platform web` against `tests/installation/wave3-consumer` and
  exercised with Playwright/axe, not just compiled.

## A real bug found and fixed by testing in the browser

Playwright caught something `cargo check`/`cargo test` cannot: Popover's
Escape key did nothing. Instrumenting `document.addEventListener` in the
page showed the count staying at zero after opening a Popover — this
crate's `use_global_escape_listener`/`use_outside_dismiss` (in `lib.rs`,
inherited unmodified from the M1 Dialog/Select fork) never actually
register their `document::eval`-based listener in this Dioxus 0.7.9/0.7.10
web runtime. `use_animated_open`'s one-shot request/response `document::eval`
call *does* resolve correctly, so the gap is specific to the long-lived,
repeatedly-firing eval pattern, not `document::eval` generally.

`dialog`'s existing browser tests never actually exercised this path:
`DialogRoot`'s Escape handling is a native Dioxus `onkeydown` on its own
root element, and the registry facade's outside-dismiss is a native
`onclick` on its full-screen overlay div — neither uses
`use_global_escape_listener`/`use_outside_dismiss` at all. Popover has no
overlay and was the first primitive in this codebase to depend solely on
the broken path.

**Fix applied**: `PopoverRoot` now also carries a native `onkeydown` Escape
handler, the same reliable mechanism `dialog` already uses (see
`packages/adico-primitives/src/popover.rs`). **Known gap, not fixed here**:
outside-dismiss and focus-trapping for `popover`/`context-menu` still
depend on the unverified eval path. Root-causing the interpreter-level
listener registration failure is out of scope for a component migration
task — it's shared `lib.rs` infrastructure, so it belongs to M6 (shared
primitive expansion) rather than this wave. The Playwright suite below
asserts only what's confirmed working in the browser, not the untested
paths.

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
(cd tests/installation/wave3-consumer && \
  ../../../target/debug/adico init && \
  ../../../target/debug/adico add tooltip popover hover-card dropdown-menu context-menu menubar && \
  cargo build && \
  cargo check --target wasm32-unknown-unknown)
(cd tests/installation/wave3-consumer && dx serve --platform web) &
(cd tests/playwright && ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:8080 npm run test:wave3)
```

| Check | Result |
| --- | --- |
| `cargo xtask registry build`/`validate` | 18 item payload(s) (12 existing + 6 new) |
| `cargo xtask provenance check` | 2 imported record(s), 23 source unit(s) |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed (includes `wave3_batch_add_installs_every_overlay_item_once`) |
| `adico-primitives` native / web+wasm32 / desktop | all passed, including 24 doctests |
| `wave3-consumer`: `adico init && adico add …` | plan reported all 6 `@adico/*` addresses plus shared `@adico/cn`; `adico add complete.` |
| `wave3-consumer`: `cargo build` (native) / `cargo check --target wasm32-unknown-unknown` | both succeeded, no warnings under `RUSTFLAGS=-D warnings` |
| `wave3.spec.ts` (7 tests, live `dx serve` + Playwright + axe) | all passed: Tooltip hover/ARIA, Popover open/dialog-semantics/Escape, HoverCard hover, DropdownMenu roving-focus/selection, ContextMenu right-click/keyboard/Escape, Menubar open/selection, zero critical axe violations |

Every command above is offline-safe except `wave3-consumer`'s `cargo build`
and the `dx serve`/Playwright pair, matching the existing
`*-consumer`/Playwright fixtures.

## Verification satisfied

Task 4.4's own verification requirement — "verify browser keyboard, focus,
and accessibility coverage for every migrated interactive item" — is
satisfied by `wave3.spec.ts` running against a live `dx serve` of
`wave3-consumer`, plus the offline `wave3_batch_add_installs_every_overlay_item_once`
integration test for installation mechanics on every `cargo test --workspace`
run.
