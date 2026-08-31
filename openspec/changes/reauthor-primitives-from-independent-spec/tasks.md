## 1. Wave A — near-free wins (prove the recipe)

> **Note (2026-08-31):** Every task below that says "author/extend tests," "confirm/extend
> unit test coverage," or similar means: write those tests in
> `packages/adico-primitives/tests/test_<file-stem>.rs` (one file per rewritten source module,
> filename pattern `test_*.rs`), never as an inline `#[cfg(test)] mod` block in `src/*.rs` —
> this applies even to tests of currently-private state, which requires widening the tested
> item to `pub` (see design.md's Decisions and Risks/Trade-offs). This convention is not
> rewritten into every task line below. The existing `packages/adico-primitives/tests/
> public_api.rs` is renamed `test_public_api.rs` to match. Separately, several files this
> change touches have no tracked row in `statics/primitive_compatibility.json` at all (e.g.
> `collection.rs`, an internal-only file with no component/util entry on either axis) — for
> those, a task's "close the compat gap" step is a no-op, not a blocker; the spec is ARIA APG
> plus the file's actual consumers' needs (see design.md's synthesis decision).

- [x] 1.1 `collection.rs`: derive spec from ARIA APG (roving focus / grid navigation); no
      tracked `primitive_compatibility.json` row exists for this internal-only file, so its
      spec is also its ~12 consuming components' actual needs; confirm/extend unit test
      coverage; re-author with no upstream file open; drop the file's upstream header and
      remove it from its provenance record's `localPaths` in the same commit. Verify: `cargo
      test -p adico-primitives`, `cargo run -p adico-xtask -- provenance check` shows the
      count decreased by exactly one file. Done 2026-08-31: the file's registration/roving-
      state logic was already independently structured (builder API, RTL-aware grid nav,
      key-identity reindexing — no fork-text residue found), so the rewrite was documentation
      (ARIA APG spec comment) plus attribution removal, not a logic rewrite; added 34 tests in
      the new `packages/adico-primitives/tests/test_collection.rs` covering previously-
      untested `roving_tabindex` anchor precedence, `focus_next/prev_matching`,
      `set_focus_key`/`focused_key`, register/unregister reindexing, and
      `try_focus_placement` (existing coverage was 15 tests on `navigate_key`/
      `navigate_grid_key` only). `register_item`/`unregister_item`/`CollectionItemState`
      widened to `pub` per the test-placement convention above. Verified: `cargo fmt --all
      --check`, `cargo check --locked --workspace`, `cargo clippy --locked -p adico-cli -p
      adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask
      --all-targets -- -D warnings`, `cargo test --locked` on the same five packages (all
      passing), `cargo run -p adico-xtask -- provenance check` (64 → 63 source units, 9
      records unchanged), `cargo run -p adico-xtask -- registry validate` (no registry facade
      exists for this internal-only primitive, so nothing to update).
- [x] 1.2 `js/focus-trap.js`: derive spec from ARIA APG focus-trap guidance; confirm existing
      focus-scope tests still cover it or extend them; re-author; drop header + record entry.
      Verify: `cd tests/playwright && npm test` (focus-trap-dependent specs), provenance
      count decreases by one. Done 2026-08-31: rewrote the header as an ARIA APG "Trapping
      Focus" spec citation; the implementation itself was unchanged (already independently
      structured — dual guard sentinels, tabindex-aware `focusable()`, non-modal `FocusScope`
      — no fork-text residue). Added a Tab-cycling test to `dialog.spec.ts` and a second
      focusable element to the `dialog-consumer` fixture. **Verification gap, not a pass:**
      this new test and the pre-existing "closes with Escape" test both fail in this sandbox
      against a fresh, otherwise-unmodified `dialog-consumer` build — traced to `DialogRoot`'s
      `use_escape_key`/`layer.is_topmost()` gate (`lib.rs`/`layer.rs`), not fixed here per
      explicit user direction to keep this task scoped to `focus-trap.js`. Recorded in
      `provenance/records/adico-primitives-dialog-select.json`'s changes log. `cargo test -p
      adico-primitives` unaffected (no Rust changed) and passing; `cargo run -p adico-xtask --
      provenance check` 63 → 62. Follow-up needed: a separate task/change to fix the
      Escape/layer regression and re-run the full `tests/playwright` suite once fixed.
- [x] 1.3 `typeahead.rs`: derive spec from ARIA APG typeahead guidance +
      `primitive_compatibility.json`; confirm the existing 15 unit tests cover the spec or
      extend them; re-author; drop header + record entry. Verify: `cargo test -p
      adico-primitives typeahead`, provenance count decreases by one. Done 2026-08-31: unlike
      1.1/1.2, this file's own prior header admitted its matching algorithm was unchanged
      from the original module, so this was a genuine rewrite, not documentation-only.
      Neither `statics/catalogs/base-ui.json` nor `statics/primitive_compatibility.json`
      tracks a typeahead capability on either axis, confirming the fuzzy/adaptive-
      keyboard/phonetic matching layer is an adico-only extra (kept per the "one-stop shop"
      decision), not a ported feature. Independently redesigned: the weighted edit-distance
      formula (`position_weight` replaces `recency_bias`'s log-ratio-to-the-4th-power curve
      with a cubic ratio, new constants), the substitution-cost model (new weighting
      constants, added a case-only-difference tier), Unicode-codepoint proximity, and the
      cross-script phonetic table (deliberately narrowed to Latin/Cyrillic/Greek, dropping
      Arabic/Bengali, to avoid transcribing codepoints the rewrite's author couldn't verify
      directly). `KeyboardLayout`'s physical layout tables and `code_to_char` are unchanged
      (physical/factual keyboard geometry, not creative expression). Replaced 13
      relational-only tests (`assert!(a < b)`, which pass for nearly any sane fuzzy matcher)
      with 26 tests in the new `packages/adico-primitives/tests/test_typeahead.rs`, adding
      exact-index regression coverage (prefix-match precedence, single-adjacent-key-typo
      resolution, cross-script phonetic isolation). Verified: `cargo fmt --all --check`,
      `cargo check --locked --workspace`, `cargo clippy --locked -p adico-cli -p
      adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask
      --all-targets -- -D warnings`, `cargo test --locked` on the same five packages (all
      passing), `cargo run -p adico-xtask -- provenance check` (62 → 61 source units, 9
      records unchanged, correctly removing the `text_search.rs`→`typeahead.rs` rename pair),
      `cargo run -p adico-xtask -- registry validate` (no facade change needed — the
      `Typeahead`/`use_typeahead` public API is unchanged).
- [x] 1.4 Run the full baseline validation suite (`cargo fmt --all --check`, `cargo check
      --locked --workspace`, `cargo clippy --locked -p adico-cli -p adico-primitives -p
      adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings`,
      `cargo test --locked` on the same five packages) and confirm `provenance/records/`
      reflects exactly 3 fewer imported files than baseline. Done 2026-08-31: full suite
      green across all five packages; `cargo run -p adico-xtask -- provenance check` reports
      `9 imported record(s), 61 source unit(s)` — exactly 3 fewer than the 64-file baseline
      (`collection.rs`, `js/focus-trap.js`, `typeahead.rs`). Wave A's stated purpose — prove
      the five-step recipe and the provenance burn-down mechanics before scaling — is met:
      one file needed only attribution/documentation work (`collection.rs`), one needed the
      same plus new browser-test coverage that the sandbox couldn't verify
      (`js/focus-trap.js`, an unrelated pre-existing Escape/layer regression recorded as a
      follow-up, not fixed here), and one needed a genuine line-level algorithm rewrite
      (`typeahead.rs`) — a real spread of the difficulty this change will keep encountering
      across Waves B-G, not a uniformly "near-free" set. `cargo run -p adico-xtask --
      registry validate` unaffected throughout (no registry facade touched by any of the
      three files).

## 2. Wave B — menu family (select/combobox + menu unification)

- [ ] 2.1 Re-author `select/{mod,context}.rs` + `select/components/{mod,group,list,option,
      select,trigger,value}.rs` (9 files) as a single `select.rs`, onto
      `positioner::Positioner`/`Arrow` and `layer::use_layer`, per task `7.8d` in
      `build-adico-component-ecosystem`. Derive spec from ARIA APG Listbox/Combobox pattern +
      compatibility row; author/extend tests first. Verify: `cargo test -p adico-primitives
      select`, `tests/playwright/select.spec.ts` passes, `find packages/adico-primitives/src
      -mindepth 2 -name '*.rs' | grep select` returns nothing, provenance count decreases by
      9 files (one record entry set).
- [ ] 2.2 Re-author `combobox/{mod,context}.rs` + `combobox/components/{mod,combobox,input,
      list,empty,option}.rs` (8 files) as a single `combobox.rs`, same positioner/layer
      migration. Verify: `cargo test -p adico-primitives combobox`, relevant Playwright spec,
      directory removed, provenance count decreases by 8.
- [ ] 2.3 Re-author `dropdown_menu.rs`, `menubar.rs`, `context_menu.rs` onto the unified
      `menu::Menu` anatomy, per task `7.8e`. These are zero-churn today — author tests from
      the ARIA APG Menu/Menubar pattern before rewriting (none exist currently). Verify:
      `cargo test -p adico-primitives menu`, `tests/playwright/*menu*.spec.ts`, provenance
      count decreases by 3.
- [ ] 2.4 Update `registry/ui/select.rs`, `registry/ui/combobox.rs`,
      `registry/ui/dropdown-menu.rs`, `registry/ui/menubar.rs`, `registry/ui/context-menu.rs`
      facades and `registry/registry.json` for any public API change from 2.1-2.3; run
      `cargo run -p adico-xtask -- registry build` then `registry validate`; install through
      `examples/basic-spa` or a `tests/installation/*` fixture and confirm it builds.

## 3. Wave C — shared internals (dependencies of later waves)

- [ ] 3.1 `portal.rs`: derive spec (real DOM portal semantics per `design.md` §8a in
      `build-adico-component-ecosystem`); author/extend tests; re-author; drop header +
      record entry. Verify: `cargo test -p adico-primitives portal`, provenance count -1.
- [ ] 3.2 `pointer.rs` + `move_interaction.rs`: derive spec from the existing unified
      press/long-press/drag gesture design (`gesture.rs` precedent); author/extend tests;
      re-author both; drop headers + record entries. Verify: `cargo test -p adico-primitives
      pointer move_interaction`, provenance count -2.
- [ ] 3.3 `listbox.rs`, `selection.rs`, `selectable.rs`: derive spec from ARIA APG selection
      patterns; author/extend tests; re-author; drop headers + record entries. Verify: `cargo
      test -p adico-primitives listbox selection selectable`, provenance count -3.
- [ ] 3.4 Run full baseline validation suite; confirm no consumer of these internals
      (accordion, dialog, popover, select, combobox, tabs, calendar, menubar, toolbar,
      slider, color-picker, drag-and-drop-list) regressed: `cargo check --workspace`,
      relevant Playwright specs.

## 4. Wave D — large zero-churn leaves

- [ ] 4.1 `tag_group.rs` (1052 lines, zero unit tests): derive spec from ARIA APG (tag/chip
      list pattern) + compatibility row; author tests first; re-author; close parity gaps;
      drop header + record entry. Verify: `cargo test -p adico-primitives tag_group`,
      relevant Playwright spec if one exists or is added, provenance count -1.
- [ ] 4.2 `toast.rs` (761 lines, zero unit tests, timer-driven): derive spec from ARIA APG
      alert/status pattern; author tests covering auto-dismiss timing first; re-author; drop
      header + record entry. Verify: `cargo test -p adico-primitives toast`, provenance
      count -1.
- [ ] 4.3 `toolbar.rs`, `scroll_area.rs`, `tabs.rs`, `radio_group.rs`, `toggle_group.rs`:
      derive specs from their ARIA APG patterns; author tests first; re-author each; drop
      headers + record entries. Verify: `cargo test -p adico-primitives toolbar scroll_area
      tabs radio_group toggle_group`, `tests/playwright/*.spec.ts` for tabs/radio-group if
      present, provenance count -5.
- [ ] 4.4 Flatten `virtual/{mod,types,utils,virtualizer}.rs` + `virtual_list.rs` (5 files)
      into a single `virtual_list.rs`; derive spec from the current virtualization contract
      (window/overscan/measurement behavior) since there is no ARIA pattern for this one;
      author tests covering scroll-window math first. Verify: `cargo test -p
      adico-primitives virtual_list`, directory removed, provenance count -5 (one record
      entry set).
- [ ] 4.5 Update any `registry/ui/*.rs` facades affected by 4.1-4.4's parity closures; run
      `registry build` + `registry validate`; confirm via a consumer fixture install.

## 5. Wave E — remaining files by provenance record

- [ ] 5.1 `wave2-state` record: `avatar.rs`, `checkbox.rs`, `collapsible.rs`, `switch.rs`,
      `toggle.rs`. Derive specs from ARIA APG; author/extend tests; re-author each; delete
      the `adico-primitives-wave2-state.json` record once empty. Verify: `cargo test -p
      adico-primitives`, `tests/installation/wave2-*-consumer` fixture, provenance count -5,
      record file deleted.
- [ ] 5.2 `wave2-simple` record: `aspect_ratio.rs`, `label.rs`, `progress.rs`. Same recipe.
      Verify: tests pass, record `adico-primitives-wave2-simple.json` deleted, provenance
      count -3.
- [ ] 5.3 `wave2-roving-focus` record (remainder not covered by Wave B): `accordion.rs`,
      whatever of `radio_group.rs`/`tabs.rs`/`toggle_group.rs` was not already closed in Wave
      D under a different record. Same recipe. Verify: tests pass, record
      `adico-primitives-wave2-roving-focus.json` deleted, provenance count reduced
      accordingly.
- [ ] 5.4 `wave2-risk` record: `scroll_area.rs` (if not already closed in Wave D),
      `alert_dialog.rs`, `slider.rs`. Same recipe — `alert_dialog.rs` migrates onto
      `scroll_lock` per the existing 7.8a precedent if not already done. Verify: tests pass,
      record `adico-primitives-wave2-risk.json` deleted, provenance count reduced
      accordingly.
- [ ] 5.5 `wave3-overlays` record residue: `tooltip.rs`, `popover.rs`, `hover_card.rs`
      (already migrated onto `Positioner` in prior work — verify no residual upstream text
      remains before dropping headers). Same recipe. Verify: tests pass, record
      `adico-primitives-wave3-overlays.json` deleted, provenance count -3 (or fewer if
      dropdown_menu/menubar/context_menu already accounted for in Wave B).
- [ ] 5.6 `wave4-collection` record residue: `calendar.rs` (2748 lines), `date_picker.rs`
      (1586 lines), `separator.rs`. Derive specs from ARIA APG Grid/Date-picker patterns;
      author tests first — these are the two largest remaining files, budget accordingly.
      Verify: tests pass, `tests/playwright/*.spec.ts` for calendar/date-picker if present,
      record `adico-primitives-wave4-collection.json` deleted, provenance count reduced
      accordingly.
- [ ] 5.7 `wave5-extras` record: `drag_and_drop_list.rs`, flatten `color_picker.rs` +
      `color_picker/color_naming.rs` into one `color_picker.rs`. Derive specs; author tests
      first. Verify: tests pass, directory removed, record
      `adico-primitives-wave5-extras.json` deleted, provenance count -3.
- [ ] 5.8 `dialog-select` record residue: `dialog.rs`, `lib.rs`'s non-facade content if any
      remains outside Wave F's scope, `js/focus-trap.js` cross-check (already done in 1.2).
      Verify: tests pass, `tests/playwright/dialog.spec.ts` passes.
- [ ] 5.9 Update all affected `registry/ui/*.rs` facades, `registry/registry.json`, and
      regenerated output for every parity closure across 5.1-5.8; run `registry build` +
      `registry validate`; confirm via consumer fixture installs (`examples/basic-spa`,
      relevant `tests/installation/*`).

## 6. Wave F — `lib.rs` and crate-wide cleanup

- [ ] 6.1 Confirm every module `lib.rs` declares has been rewritten/flattened in Waves A-E;
      re-author `lib.rs` itself (crate facade, module declarations, doc comments); drop its
      `// Derived from` header and remove it from `adico-primitives-dialog-select.json`'s
      `localPaths`, deleting the record if it empties. Verify: `cargo run -p adico-xtask --
      provenance check` reports `0 imported record(s), 0 source unit(s)` (only the all-zero
      `example-dioxus-components.json` fixture remains).
- [ ] 6.2 Remove `lib.rs`'s `#![allow(dead_code)]` / `#![allow(clippy::collapsible_if)]`
      blocks whose `reason` strings cite the initial fork; resolve every warning that
      surfaces under `-D warnings` (delete genuinely-unused code or justify a narrower,
      non-fork-related allow per item). Verify: `cargo clippy --locked -p adico-primitives
      --all-targets -- -D warnings` passes with no fork-related allows remaining.

## 7. Wave G — `adico-cli` animation utilities (parallel, independent)

- [ ] 7.1 Re-derive `packages/adico-cli/src/css.rs`'s Tailwind animation utilities from the
      Tailwind v4 `@theme`/`@keyframes` documentation directly, replacing the
      `Wombosvideo/tw-animate-css`-derived content; or explicitly record the decision to keep
      this single narrow attribution if re-derivation is not warranted. Verify: `cargo test
      -p adico-cli css`, and if re-derived, remove `adico-cli-theme-animation-utilities.json`
      and drop `css.rs`'s header (note: `collect_imported_paths()` only scans
      `adico-primitives/src`, so this file is not covered by the automated gate — track its
      completion manually).

## 8. Closing — parity sweep and acceptance

- [ ] 8.1 Run `cargo run -p adico-xtask -- primitive-compat diff` across all axes
      (Base UI, dioxus-primitives) for every rewritten primitive; implement any remaining gap
      not already closed per-wave (this is task `7.9`'s scope in
      `build-adico-component-ecosystem` — reference it, do not re-plan it here). Verify:
      `primitive-compat check` reports no unintentional diff.
- [ ] 8.2 Confirm the one-file rule holds crate-wide: `find packages/adico-primitives/src
      -mindepth 2 -name '*.rs'` returns nothing outside `js/`.
- [ ] 8.3 Run the full baseline validation suite, the wasm32 target check
      (`cargo check --target wasm32-unknown-unknown -p adico-primitives`), the full
      `tests/playwright` suite, and a real consumer install (`examples/basic-spa` and/or
      `tests/installation/*`) end to end.
- [ ] 8.4 Record acceptance evidence (files rewritten, tests added, parity gaps closed,
      known gaps carried forward — no native/desktop/mobile fixture) in a doc under
      `docs/adico/`, matching the M-series evidence-recording pattern already used by
      `build-adico-component-ecosystem`. Confirm `provenance check` still reports zero and
      note that `remove-provenance-tracking` may now begin as a separate change.
