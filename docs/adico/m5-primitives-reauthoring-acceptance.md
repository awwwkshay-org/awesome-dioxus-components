# Primitives re-authoring acceptance (task 8.4)

Status: complete

This records task 8.4, closing out
[`reauthor-primitives-from-independent-spec`](../../openspec/changes/reauthor-primitives-from-independent-spec/proposal.md).
That change's own `tasks.md` carries the full per-file evidence (spec
derivation, test additions, compat-gap findings, provenance closure) for
every one of the ~49 files it touched across waves A-G; this document
synthesizes that record into an acceptance summary and, per this task's own
text, the list of known gaps carried forward rather than silently dropped.

## What this change did

`adico-primitives` started life as a fork of
`DioxusLabs/dioxus-components`, tracked file-by-file through
`provenance/records/*.json`. This change re-authored every remaining forked
file against its own spec — the relevant WAI-ARIA Authoring Practices Guide
pattern where one exists, the closest applicable APG guidance plus actual
consumer needs otherwise — following a five-step recipe applied uniformly
across all seven waves: derive the spec, write/extend tests externally
(`packages/adico-primitives/tests/test_<stem>.rs`, never inline), re-author
(usually no logic change, since prior M1-M4 work already left most files
independently correct), close any compat gap the spec derivation surfaced,
and drop the file's fork header plus its provenance-record entry.

- **Waves A-F** (`packages/adico-primitives/src/*.rs`, 50 files): every
  primitive module re-authored, following the recipe above. See
  `tasks.md`'s own per-task entries (1.1-1.4, 2.1-2.4, 3.1-3.4, 4.1-4.5,
  5.1-5.9, 6.1-6.3) for file-by-file evidence.
- **Wave G** (`packages/adico-cli/src/css.rs`, task 7.1): evaluated and
  explicitly decided to keep its one narrow `tw-animate-css` attribution
  rather than re-derive — see that task's own evidence and the module's own
  doc comment for the reasoning.
- **Closing** (tasks 8.1-8.3): a `primitive-compat` parity sweep (which
  found and fixed two pre-existing, unrelated xtask bugs — see below), the
  one-file-per-module rule confirmed crate-wide, and a full validation
  sweep including live-browser Playwright verification.

### The task 2.3 menu-family unblock

The one task that spanned most of this session's active work: `menu.rs` (a
new, unified `Menu`/`MenuTrigger`/`MenuContent`/`MenuItem`/... anatomy) had
sat unconsumed because migrating `dropdown_menu.rs`/`context_menu.rs`/
`menubar.rs` onto it was blocked on a layer-stack design question. Per user
direction to check Base UI's actual source before designing a fix (a
standing instruction now worth keeping for future architectural
dilemmas), `crate::layer::use_layer` was changed to join the shared
overlay stack only while its caller's `open` state is `true` — matching
Floating UI/Base UI's `useDismiss` model — rather than while merely
mounted (task 6.3). This fixed a real bug (a closed-but-still-mounted
overlay could shadow a sibling's `is_topmost()` check) and unblocked task
2.3, which then, per further Base UI source research:

- Made `dropdown_menu.rs` a direct re-export of `menu.rs`'s components
  (Base UI has no separate dropdown-menu component — `Menu` *is* the
  dropdown menu), fixing a real ARIA bug along the way
  (`role="listbox"`/`"option"` → the WAI-ARIA APG Menu Button pattern's
  `role="menu"`/`"menuitem"`).
- Kept `menubar.rs` and `context_menu.rs` independent of `menu.rs`,
  matching Base UI's own split (Menubar shares only a roving-focus
  container with Menu; Context Menu's Base UI equivalent delegates to
  `Menu.Root` only because Base UI's `Menu.Popup` is `Positioner`-anchored
  from the start, which `crate::menu::MenuContent` isn't). Both decisions
  are recorded with their Base UI source evidence in `tasks.md`.
- Surfaced and fixed a real correctness bug in `menubar.rs`: `MenubarTrigger`
  computed a `disabled` boolean but never rendered it. The first fix
  attempt used a native `disabled` attribute (wrong — that removes a
  roving-focus member from the tab order, which the ARIA APG Menubar
  pattern doesn't want); caught by review and corrected to `aria-disabled`.
- Was verified live: `tests/playwright/wave3.spec.ts`'s DropdownMenu test
  still asserted the pre-fix `listbox`/`option` roles and needed updating;
  after the fix, DropdownMenu/ContextMenu/Menubar and the batch's axe
  accessibility check all pass against `tests/installation/wave3-consumer`.

## Tests added

`packages/adico-primitives/tests/` now has 44 external test files (one per
source module needing behavioral coverage, per this change's
test-placement convention — every test lives under `tests/`, never inline
`#[cfg(test)]` in `src/*.rs`, with private items widened to `pub` and
doc-commented as test-only where a test needs to reach them).

`cargo test -p adico-primitives` totals 409 passing tests: 23 crate-root
unit tests, 306 across the 44 external test files, 80 doc tests. The
menu-family work alone added/moved: `test_menu.rs` (5, externalized from a
prior inline module, +1 new), `test_dropdown_menu.rs` (3, new),
`test_menubar.rs` (4, new — first coverage this file has ever had),
`test_context_menu.rs` (4, new — same).

## Parity/compat gaps closed this change

Beyond the menu-family ARIA fixes above:

- `statics/primitive_compatibility.json` had gone stale (by design,
  deferred until source stabilized) and, on re-sync, exposed two
  independent xtask bugs unrelated to any single primitive: two
  hand-maintained `UPSTREAM_COMPONENTS` entries (`combobox`, `select`)
  still pointed at pre-flattening directory paths (`"combobox/"`,
  `"select/"`), silently zeroing their introspected props/hooks/components
  on sync; and the `dioxus_primitives` axis's module-to-file lookup had no
  way to know `VirtualList`'s upstream module name (`virtual`) is a
  parent-namespace catalog artifact, not the real component (`virtual_list`,
  which already resolved correctly on its own name). Both fixed —
  see task 8.1's evidence for the full self-correction record (an initial
  fix attempt for the `virtual` case was wrong and reverted after review).

## Known gaps carried forward

Per this task's own text ("known gaps carried forward"), not fixed here —
each was found, confirmed live where possible, and left as a named,
attributed gap rather than a silent omission:

1. **`positioner.rs`'s `document::eval` DOM measurement never completes in
   this Dioxus web runtime.** Confirmed live this session in two
   independent fixtures (`wave3-consumer`, `select-consumer`): every
   `Positioner`-rendered surface (`tooltip`, `popover`, `hover_card`,
   `select`, `combobox`) opens logically (`data-state="open"`, correct
   ARIA) but stays visually `visibility: hidden` — the measurement pass
   that should flip it never runs. This is the single largest gap in this
   change's own live-browser verification and blocks real interaction
   testing for five components.
2. **The long-lived `document::eval` listener pattern never registers its
   `addEventListener` call**, a related but distinct symptom of the same
   underlying `document::eval` defect (found 2026-08-25, recorded in
   `adico-primitives-wave3-overlays.json`'s git history before that record
   was fully closed and removed). This is why `context_menu.rs`'s
   `use_outside_dismiss` — and by extension `Context Menu`'s
   `primitive_compatibility.json` status — stays `Partial`, not `Built`,
   despite being fully re-authored and ARIA-correct.
   Both (1) and (2) are almost certainly one root cause with two symptoms
   in this crate's `document::eval` usage pattern under the pinned Dioxus
   version, and are the natural next investigation — scoped as a follow-up
   change, not fixed inline here.
3. `hover_card.rs`'s `force_mount` prop is not honored on any target
   (SSR or web/native) despite its own doc comment — found and documented
   in task 5.5, not fixed (fixing risks altering unrelated animation-close
   timing untested).
4. `menu.rs` is not yet composed on `positioner::Positioner`, not wired to
   `use_typeahead`, and `MenuSubmenuTrigger`'s hover-intent-delay opening
   is unimplemented (click and arrow-key open/close only).
5. `Menubar` has no `default_open`-equivalent prop — which menu is open
   derives entirely from roving focus, itself driven by events a bare
   `VirtualDom::rebuild_in_place()` never dispatches — so "a menu is open
   and its content renders" is untestable without a live browser; named in
   `test_menubar.rs`'s own header.
6. `tests/playwright/wave2-*.spec.ts`, `wave4.spec.ts`, `wave5-*.spec.ts`,
   `mode-toggle.spec.ts`, `theme-switcher.spec.ts`, and `fullstack.spec.ts`
   were **not** re-run during this change's closing sweep — none of those
   specs' primitives were touched by any wave here (only `layer.rs` and
   the menu family have live-browser-relevant changes), so re-running them
   would have exercised nothing this change could have affected.
7. `tests/playwright/dialog.spec.ts`'s pre-existing focus-trap failure
   ("traps Tab focus... wraps at both ends", 1 of 4 tests, found during
   task 6.3's own live verification) is unrelated to and not touched by
   this change.
8. `cargo check -p adico-primitives --features native` was **compile-checked
   only** throughout this change, not exercised — there is no native/desktop
   fixture in this repo to run it against, which task 8.3's own text
   already anticipated as a known gap, not something to newly claim
   coverage of here.

## Verification

```
cargo fmt --all --check
cargo check --locked --workspace
cargo check --target wasm32-unknown-unknown -p adico-primitives --features web
cargo check --locked -p adico-primitives --features native
cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings
cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask
cargo run -p adico-xtask -- registry validate
cargo run -p adico-xtask -- provenance check
cargo run -p adico-xtask -- primitive-compat check
openspec validate reauthor-primitives-from-independent-spec --strict
```

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `--target wasm32-unknown-unknown --features web` (`adico-primitives`) | passed |
| `--features native` (`adico-primitives`, compile only) | passed |
| `cargo clippy ... -- -D warnings` (5 packages, all targets) | passed |
| `cargo test --locked` (5 packages) | passed — 409 tests in `adico-primitives` alone |
| `cargo xtask registry validate` | passed: 46 item payload(s) in `@adico` |
| `cargo xtask provenance check` | passed: 1 imported record(s), 1 source unit(s) |
| `cargo xtask primitive-compat check` | passed: up to date |
| `openspec validate ... --strict` | passed |
| `tests/playwright/wave3.spec.ts` (menu family + overlays) | 4/7 pass — see known gap 1 above for the other 3 |
| `tests/playwright/select.spec.ts` | 0/2 pass — same known gap 1, confirmed independently |

## Provenance close-out

`adico-primitives` is now down to **zero real provenance records** — every
`adico-primitives-*.json` record has been closed and removed; the last one
(`adico-primitives-wave3-overlays.json`, `context_menu.rs`'s final source
unit) was removed in task 2.3's closing commit. `provenance/records/` has
two files remaining: `adico-cli-theme-animation-utilities.json` (`adico-cli`,
kept by the explicit, documented decision in task 7.1 — not an oversight)
and `example-dioxus-components.json`, an M0 schema fixture with a
deliberate all-zero placeholder revision and a `localPaths` entry
(`packages/adico-primitives/src/example.rs`) that has never existed as a
real file — `cargo xtask provenance check`'s own source explicitly excludes
any record with that all-zero revision sentinel from its count ("does not
represent imported code"), so this file is not a lingering `adico-primitives`
obligation despite its path. `cargo xtask provenance check` therefore
correctly reports `1 imported record(s), 1 source unit(s)`: the one real,
intentionally-kept `adico-cli` record.

`openspec/changes/remove-provenance-tracking` (if that change exists or is
proposed) may now begin against `adico-primitives`'s clean state.

## Acceptance statement

Every file `reauthor-primitives-from-independent-spec` set out to
re-author (all of `packages/adico-primitives/src/*.rs`, plus
`packages/adico-cli/src/css.rs`'s explicit keep-decision) has been: given
its own derived spec, given test coverage where none existed, verified to
have no unintended logic change (or, where a real gap was found, fixed and
documented), and had its fork attribution dropped. `adico-primitives`
carries zero provenance records. The eight known gaps above are named,
evidenced, and — where a root cause could be identified — attributed to a
single underlying `document::eval` defect worth its own follow-up change
rather than eight independent mysteries. This change is complete;
`reauthor-primitives-from-independent-spec` is ready to archive.
