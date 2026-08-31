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
      focusable element to the `dialog-consumer` fixture. **Verification gap, partially
      closed:** this new test and the pre-existing "closes with Escape" test both failed in
      this sandbox against a fresh, otherwise-unmodified `dialog-consumer` build, traced to
      `DialogRoot`'s `use_escape_key`/`layer.is_topmost()` gate. Investigated and partially
      fixed same-day (2026-08-31, separate commit, see `layer.rs`): a real bug where
      `DialogContent`'s own `use_outside_dismiss` call registered a *second*, later-pushed
      layer slot in a different scope than `DialogRoot`'s `use_escape_key`, permanently
      shadowing the root's own `is_topmost()` check. Fixed in `layer.rs` via
      `use_layer_member()` joining the root's slot instead of registering a new one; verified
      by `packages/adico-primitives/tests/test_layer.rs` and by `dialog.spec.ts`'s outside-
      interaction and nested-dialog-Escape tests, both now passing (the nested-dialog test
      previously passed vacuously for the wrong reason before this fix — nesting wasn't
      exercising the split-scope bug at all). **Two further, separate findings surfaced
      during that investigation, deliberately left unfixed as out of scope** (see the
      provenance record's changes log for full detail): (1) a closed-but-mounted overlay
      (e.g. an unopened nested dialog) still occupies a layer-stack slot merely by mounting,
      which can shadow an *open* ancestor's own Escape handling — a design gap in when a
      layer should count as "active," not a small fix; (2) in this same browser sandbox, the
      focus trap does not appear to move initial focus into a `DialogContent`'s children at
      all (`document.activeElement` stays on the external trigger button), which is likely
      why the new Tab-cycling test still fails, independent of the layer bug. `cargo test -p
      adico-primitives` passing throughout; `cargo run -p adico-xtask -- provenance check`
      63 → 62.
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

- [x] 2.1 Re-author `select/{mod,context}.rs` + `select/components/{mod,group,list,option,
      select,trigger,value}.rs` (9 files) as a single `select.rs`, onto
      `positioner::Positioner`/`Arrow` and `layer::use_layer`, per task `7.8d` in
      `build-adico-component-ecosystem`. Derive spec from ARIA APG Listbox/Combobox pattern +
      compatibility row; author/extend tests first. Verify: `cargo test -p adico-primitives
      select`, `tests/playwright/select.spec.ts` passes, `find packages/adico-primitives/src
      -mindepth 2 -name '*.rs' | grep select` returns nothing, provenance count decreases by
      9 files (one record entry set). Done 2026-08-31: consolidated into
      `packages/adico-primitives/src/select.rs`; the old file had *no* `Positioner`/`layer`
      participation at all (inline listbox, hand-rolled Escape on the list's own `onkeydown`,
      no outside-dismiss) — wired onto `use_escape_key` (root) + `use_outside_dismiss` (list,
      via `use_layer_member`, mirroring `popover.rs`'s Root/Content split) + `Positioner` as
      the listbox's own root element. Extended `positioner.rs` (not itself forked) with
      `on_mounted`/`on_keydown`/`on_blur` props to let `Positioner` be that root element
      without an extra wrapper div — reusable by 2.2's identical migration. Added 8 tests in
      `tests/test_select.rs`; `cargo test -p adico-primitives` and `registry validate` both
      green; provenance `9 imported record(s), 52 source unit(s)` (−9, exact). `select.spec.ts`
      still fails: live-browser testing surfaced that `positioner.rs`'s own
      `measure_anchor_and_viewport()` (a `document::eval` round trip) never resolves in this
      sandbox's `dx`-served web build — confirmed via signal-based instrumentation that
      `onmounted` fires and `get_client_rect()` resolves, but the eval round trip does not, so
      the listbox stays `visibility: hidden` despite mounting with correct ARIA state. This is
      a pre-existing `Positioner` gap, not introduced here: no popover/tooltip/hover-card
      consumer fixture existed before this task to have exercised `Positioner` in a live
      browser at all. Tracked as a follow-up distinct from this task; SSR rendering, ARIA
      wiring, and escape/layer wiring are otherwise confirmed correct both by unit tests and by
      inspecting live DOM state (`data-state="open"`, correct ids, correct `aria-*`).
- [x] 2.2 Re-author `combobox/{mod,context}.rs` + `combobox/components/{mod,combobox,input,
      list,empty,option}.rs` (8 files) as a single `combobox.rs`, same positioner/layer
      migration. Verify: `cargo test -p adico-primitives combobox`, relevant Playwright spec,
      directory removed, provenance count decreases by 8. Done 2026-08-31: consolidated into
      `packages/adico-primitives/src/combobox.rs`, mirroring 2.1's design exactly (root
      `use_escape_key`, list `use_outside_dismiss` + `Positioner`, both via a new `root_id`).
      `ComboboxInput` already generated its own overridable id pre-rewrite (unlike
      `SelectTrigger`) — preserved that and published it to a new `ComboboxContext.input_id`
      for `Positioner`'s anchor instead of replacing it. `positioner.rs` needed one more prop
      beyond 2.1's three: `on_pointer_down`, since the listbox already prevented pointer-down
      default to keep DOM focus in the input (combobox never moves focus into options, unlike
      select). Added 6 tests in `tests/test_combobox.rs`; two draft per-option assertions
      (selection, filter-visibility, the empty state) were written, found to pass only
      vacuously — `ComboboxOption`'s own visibility depends on the same `use_effect`-driven
      option-registration hook `select.rs`'s tests already found untestable under a bare
      `rebuild_in_place()`, so a matches-everything and a matches-nothing query produced
      identical, entirely-optionless output — and removed rather than kept as misleading
      coverage. `cargo test -p adico-primitives` and `registry validate` both green;
      provenance `9 imported record(s), 44 source unit(s)` (−8, exact). No consumer fixture or
      Playwright spec exists for combobox at all (unlike select) — creating one is task 2.4's
      scope, not this one's — so live-browser verification remains an open gap here too,
      compounded by 2.1's already-documented pre-existing `positioner.rs` `document::eval`
      bug. SSR rendering, ARIA wiring, and the escape/layer wiring are otherwise confirmed
      correct by the unit tests and code review against 2.1's now-proven design.
- [x] 2.3 Re-author `dropdown_menu.rs`, `menubar.rs`, `context_menu.rs` onto the unified
      `menu::Menu` anatomy, per task `7.8e`. These are zero-churn today — author tests from
      the ARIA APG Menu/Menubar pattern before rewriting (none exist currently). Verify:
      `cargo test -p adico-primitives menu`, `tests/playwright/*menu*.spec.ts`, provenance
      count decreases by 3. BLOCKED pending a layer-active design decision (found 2026-08-31,
      not yet resolved): `menu.rs`'s own `Menu` component handles Escape via a hand-rolled
      check on `ctx.open`, never consulting `crate::layer` at all (no `use_escape_key`, no
      `use_outside_dismiss` anywhere in the module) — only `MenuSubmenuRoot` calls `use_layer()`
      directly, and since it is always-mounted (not conditionally, like `DialogContent`), it
      still occupies a layer-stack slot while closed, which can shadow a sibling's
      `is_topmost()` check even though its own Escape handling additionally gates on `open()`.
      Fixing this correctly needs a new reactive "active vs. merely-mounted" layer primitive
      (distinct from the mount/unmount `use_layer_member()` fix already shipped) plus rewiring
      `MenuSubmenuRoot` itself — a real design decision, not a small fold-in, and `menu.rs`'s
      own doc comment already warns that migrating `context_menu`/`dropdown_menu`/`menubar`
      onto it without live-browser re-verification risks silently regressing tested behavior.
      Do not start this task until that design is settled with the user.
      UNBLOCKED 2026-08-31 by task 6.3 (user: "Do it like baseui then" — the layer stack is now
      open-state-scoped, matching Floating UI/Base UI's `useDismiss`, so `MenuSubmenuRoot`'s
      always-registered-while-mounted concern above no longer applies). `dropdown_menu.rs`
      portion DONE 2026-08-31: checked Base UI's actual source (mui/base-ui on GitHub) before
      designing this, per standing instruction to check Base UI on an architectural dilemma —
      Base UI has no separate dropdown-menu component at all; `context-menu/root/
      ContextMenuRoot.tsx` renders `<Menu.Root {...props} />` directly and `menubar/
      Menubar.tsx` is a thin roving-focus container around ordinary `Menu.Root`s, confirming
      `Menu` itself is meant to be the dropdown-menu implementation. Compared
      `dropdown_menu.rs`'s Props structs field-for-field against `menu::Menu`'s/`MenuTrigger`'s/
      `MenuContent`'s/`MenuItem`'s: identical shape, and the component bodies were
      logic-identical except `dropdown_menu.rs` rendered `role="listbox"`/`role="option"`/
      `aria-haspopup="listbox"` instead of the WAI-ARIA APG Menu Button pattern's
      `role="menu"`/`role="menuitem"`/`aria-haspopup="menu"` — an ARIA bug, not an intentional
      divergence. `dropdown_menu.rs` is now `pub use crate::menu::{Menu as DropdownMenu, ...}`
      re-exports; grepped `registry/`, `apps/`, `examples/`, `tests/installation/` for any
      `role=listbox`/`role=option` selector before landing the role change — none found.
      `menubar.rs`/`context_menu.rs` remain their own independent implementations — neither has
      a `menu::Menu` counterpart for its trigger/anchoring model (click-point + long-press for
      context_menu; multi-menu open-state coordination for menubar) — whether their
      `MenuContent`/`MenuItem` rendering specifically can still be shared is remaining scope
      here, to be evaluated per-file. Externalized `menu.rs`'s 5 inline tests to
      `tests/test_menu.rs` (this crate's own test-placement convention) plus 1 new test
      pinning the ARIA role contract; added 2 tests in `tests/test_dropdown_menu.rs` pinning
      the re-export wiring and the role fix (8 tests total, `cargo test -p adico-primitives
      menu` green). No `tests/playwright/*menu*.spec.ts` exist for any of the three files
      (verified: `ls tests/playwright | grep -i menu` is empty) — recording that gap explicitly
      rather than treating this verify line as satisfied; a live-browser spec for the role
      change is deferred to task 2.4, matching how task 2.2 (combobox) handled the same
      missing-coverage situation. Full baseline (fmt/check/clippy/test) plus `--target
      wasm32-unknown-unknown --features web` green; `registry validate` unaffected (facade
      untouched); provenance `2 imported record(s), 2 source unit(s)` (wave3-overlays record
      drops to 2 units: `context_menu.rs`/`menubar.rs`).
      `menubar.rs` portion DONE 2026-08-31: kept independent of `crate::menu` rather than
      forcing delegation — re-checked Base UI's `Menubar.tsx` and confirmed Base UI's own
      Menubar shares nothing with `Menu` beyond a roving-focus `CompositeRoot` container (each
      bar item composes its own ordinary `Menu.Root` in consumer code); `menubar.rs` already
      gets the equivalent roving-focus container from `crate::collection`, and its per-menu
      `is_open`/`set_open`-across-siblings coordination has no `menu::MenuContext` counterpart
      to adapt onto, so shared rendering code would need that adapter without saving any of the
      APG-mandated behavior, which was already correct here. Documented this decision in the
      module's own doc comment. Writing tests against the APG Menubar pattern surfaced one real
      bug, fixed: `MenubarTrigger` computed `disabled` but never rendered it at all (no
      `data-disabled`, no `aria-disabled`) — a disabled `MenubarMenu`'s trigger looked fully
      interactive with no visual or assistive-tech indication. First fix attempt added both
      `"data-disabled": disabled()` *and* a native `disabled: disabled()` attribute, copying
      `menu::MenuTrigger`'s own pair — WRONG, caught by advisor review before it shipped
      further: `MenubarTrigger` is a roving-focus *member* of a sibling collection (it carries
      `onmounted`/`onblur`/`tabindex` for that), unlike `MenuTrigger`, which is the sole trigger
      for a standalone menu. A native `disabled` attribute makes a `<button>` unfocusable,
      dropping a disabled bar item out of the tab order entirely — the APG Menubar pattern
      wants disabled items to stay focusable (`aria-disabled`, not `disabled`) so keyboard
      users can navigate onto them and discover they're unavailable. Corrected to
      `"data-disabled": disabled(), aria_disabled: disabled()`, matching
      `context_menu.rs`'s own `ContextMenuItem` precedent (`aria_disabled`, no native
      `disabled`) rather than `MenuTrigger`'s. Grepped `registry/ui/menubar.rs` and all
      installed fixtures first — the facade's own `MenubarTrigger` doesn't accept a `disabled`
      prop at all, so nothing could have relied on the old (missing) behavior either way.
      Added `tests/test_menubar.rs` (4 tests: `role="menubar"`, `role="menu"` +
      default-closed content, trigger `role="menuitem"`, and disabled propagation to both the
      `MenubarMenu` wrapper and `MenubarTrigger` — including an explicit assertion that the
      trigger carries no native `disabled` attribute). `Menubar` has no `default_open`-style
      prop (unlike `Accordion`'s `default_value`) — which menu is open derives entirely from
      roving focus, itself driven by events a bare `rebuild_in_place()` never dispatches — so "a
      menu is open, its content renders, only one menu open at a time" is a named,
      undismissed coverage gap here, documented in the test file's own header (same class of
      interaction-driven-state limitation already documented for `date_picker.rs`/
      `hover_card.rs`). Full baseline green; provenance `2 imported record(s), 2 source
      unit(s)` (record now covers only `context_menu.rs`).
      `context_menu.rs` portion DONE 2026-08-31, completing this task: no dedicated ARIA APG
      "Context Menu" pattern exists; the closest applicable spec is the generic Menu pattern
      this file already implemented correctly (`role="menu"`/`"menuitem"`,
      `aria-haspopup="menu"`), unlike `dropdown_menu.rs`'s pre-task-2.3 `role="listbox"` bug —
      so no logic change was needed here either. Kept independent of `crate::menu`, same
      reasoning as `menubar.rs`: Base UI's own `ContextMenuRoot` does literally render
      `<Menu.Root>` with a virtual click-point anchor, reusing `Menu`'s `Popup`/`Item`
      directly — but that works there specifically because Base UI's `Menu.Popup` is a
      generic `Positioner`-anchored floating element from the start, and
      `crate::menu::MenuContent` is not (it renders an unpositioned `div`, the same
      registry-facade-owned `absolute`-in-`relative` layout `dropdown_menu.rs` relied on
      before it delegated). `ContextMenuContent` instead owns click-point `position: fixed`
      placement, Safari visual-viewport correction, scroll suppression while open,
      `use_outside_dismiss`, and a focus-the-content effect — none of which has a home in
      `MenuContent` without new props, and adding them untested would be new scope on an
      already-shipped, previously browser-verified component with no live-browser
      re-verification available in this pass. Checked Base UI's own `ContextMenuTrigger.tsx`
      for a keyboard-open compat gap (Shift+F10/context-menu-key, per the generic ARIA APG
      context-menu guidance): Base UI has no keyboard-open path either (right-click and
      long-press only) — matching this file's existing scope, not a gap to close. Dropped the
      fork-attribution header (SPDX-only now); documented the spec/scope decision in the
      module's own doc comment. No test file existed; added `tests/test_context_menu.rs` (4
      tests: closed-by-default content gating; the trigger's `role="button"`/
      `aria-haspopup="menu"`/`aria-expanded` contract; an open menu's `role="menu"`/
      `aria-orientation="vertical"`/`data-state` and its default `(0, 0)`
      `position: fixed` placement — reachable because, unlike `Menubar`, `ContextMenu` does
      support `default_open` directly; and disabled-item `aria-disabled`/`data-disabled`
      propagation). Full baseline (fmt/check/clippy/test) green. `provenance/records/
      adico-primitives-wave3-overlays.json`'s `originalPaths`/`localPaths` are now empty
      (`dropdown_menu.rs` re-exported from `menu.rs`, `menubar.rs`/`context_menu.rs` both
      re-authored in place) — removed the record file itself, per this change's established
      convention for a fully-emptied provenance record (matching
      `adico-primitives-dialog-select.json`'s precedent from task 6.1); `provenance check`
      now reports `1 imported record(s), 1 source unit(s)` crate-wide.
- [x] 2.4 Update `registry/ui/select.rs`, `registry/ui/combobox.rs`,
      `registry/ui/dropdown-menu.rs`, `registry/ui/menubar.rs`, `registry/ui/context-menu.rs`
      facades and `registry/registry.json` for any public API change from 2.1-2.3; run
      `cargo run -p adico-xtask -- registry build` then `registry validate`; install through
      `examples/basic-spa` or a `tests/installation/*` fixture and confirm it builds. Partially
      done 2026-08-31 (the 2.1/2.2 portion only — the `dropdown-menu`/`menubar`/`context-menu`
      portion isn't actionable until 2.3 is unblocked and done): no primitive *prop* API
      changed for `select`/`combobox`, but both facades' `SelectList`/`ComboboxList` styling
      relied on `absolute left-0 top-full` positioned against a `relative` root, which is now
      wrong (`Positioner` renders `position: fixed`, whose containing block is the viewport,
      not the root) — dropped those classes, added `z-50` for stacking (matching
      `popover.rs`'s existing convention), and dropped `ComboboxList`'s `w-full` (would now
      mean 100% of the *viewport*; `min-w-48` is the width baseline instead, matching
      `popover.rs`'s own fixed-width precedent). Recomputed and updated both files' SHA-256
      checksums in `registry/registry.json`; `registry build` and `registry validate` both
      pass. Re-installed both components with `adico add <name> --replace` into
      `tests/installation/select-consumer` (select) and `examples/basic-spa` (both — the only
      place combobox is installed anywhere, since no combobox consumer fixture exists) to
      confirm the new facade content round-trips through the real CLI install path, not just
      `registry validate`'s structural check; `cargo check --locked --workspace` and a
      `--target wasm32-unknown-unknown` check of both consumers are green.
      `dropdown-menu`/`menubar`/`context-menu` portion DONE 2026-08-31, completing this task:
      none of the three primitive re-authorings in task 2.3 changed a public prop API —
      `dropdown_menu.rs`'s re-exported `Menu*` components have identical `Props` shapes to the
      old `DropdownMenu*` ones (only internal `role`/`aria-haspopup` attribute *values*
      changed, which the registry facades don't read or override), `menubar.rs`'s
      `MenubarTrigger` fix only added HTML attributes it renders, and `context_menu.rs` had no
      changes beyond its header/doc comment. So `registry/ui/dropdown-menu.rs`,
      `registry/ui/menubar.rs`, and `registry/ui/context-menu.rs` need no edits, and
      `registry/registry.json`'s checksums for them are unaffected (`registry/ui/*.rs` content
      itself is untouched — `registry build`/`registry validate` both still pass at 46 items).
      Confirmed by building `tests/installation/wave3-consumer` (the dedicated consumer
      fixture for this exact batch, covering all three) with `cargo check --locked` against
      the path-patched, freshly re-authored `adico-primitives` — compiles clean with no facade
      changes required, i.e. the installed copies already round-trip through the real CLI
      install content unmodified.

## 3. Wave C — shared internals (dependencies of later waves)

- [x] 3.1 `portal.rs`: derive spec (real DOM portal semantics per `design.md` §8a in
      `build-adico-component-ecosystem`); author/extend tests; re-author; drop header +
      record entry. Verify: `cargo test -p adico-primitives portal`, provenance count -1.
      Done 2026-08-31: confirmed `build-adico-component-ecosystem`'s design.md §8a still lists
      a real DOM-escaping portal as net-new/unbuilt (not among the primitives its own "real,
      public, independently-tested modules today" list names) — this task re-authors the
      existing same-VDOM relay `portal.rs` already is, not that separate, still-unimplemented
      capability; the module's own doc comment already drew this scope line. Implementation
      unchanged (already independently structured, its own prior header admitted "ported
      unmodified"). Added 6 tests in the new `tests/test_portal.rs` (none existed before),
      which surfaced a real, previously-undocumented ordering requirement: `PortalOut` reads
      its content at the moment it renders and cannot react to a `PortalIn` that runs later in
      the same pass, so declaring `PortalOut` first renders stale/empty content. `toast.rs`
      (the only consumer) already declares `PortalIn` first, which is why this was never hit
      in practice; documented as a caveat in the module doc comment, not fixed (no consumer
      needs the reactive-order-independent version, and changing a `peek()`-based map read
      risks altering unrelated behavior). Full baseline suite green; provenance `9 imported
      record(s), 43 source unit(s)` (−1, exact). No registry facade depends on `portal.rs`
      directly (`toast.rs` is the sole consumer; its own facade is untouched).
- [x] 3.2 `pointer.rs` + `move_interaction.rs`: derive spec from the existing unified
      press/long-press/drag gesture design (`gesture.rs` precedent); author/extend tests;
      re-author both; drop headers + record entries. Verify: `cargo test -p adico-primitives
      pointer move_interaction`, provenance count -2. Done 2026-08-31: confirmed `gesture.rs`
      only unified press/long-press (context_menu.rs's/selectable.rs's duplicated timers), not
      these two files' separate concern (continuous drag-position tracking) — re-authored
      against their existing target-gated architecture instead of merging into `gesture.rs`.
      Inherited a significant finding `gesture.rs`'s own doc comment already recorded rather
      than re-investigating it: `pointer.rs`'s global listener is installed by the same
      document::eval pattern the `wave3-overlays` record documents as never actually
      registering in this web runtime — now stated directly in `pointer.rs`'s own doc comment
      too. Ungated and widened `Pointer`/`upsert_pointer` to `pub` (pure `Vec` logic, no DOM
      dependency) so they're testable under default features — this also surfaced that
      `pointer.rs`'s one prior test never ran under this repo's baseline `cargo test` at all
      (gated to `web`/`native`, default features are `[]`). Added 3 tests in the new
      `tests/test_pointer.rs` and 10 in the new `tests/test_move_interaction.rs` (4 keyboard
      tests moved unchanged, 6 new pointer-drag tests via a synthetic `HasPointerData` impl
      driving the already-fully-public `MoveInteraction` API directly, no further `pub`
      widening needed there). Also fixed a latent gap tasks 2.1/2.2 left: `test_select.rs`/
      `test_combobox.rs` had imports used only inside their `web`/`native`-excluded gated
      submodules declared at file scope, failing `cargo clippy --features web` (not part of
      the default baseline, which is why it was never caught) — moved into the submodules.
      Full baseline green, plus `cargo test`/`clippy --features web`/`--features native` all
      green (neither part of the default baseline either); provenance `9 imported record(s),
      41 source unit(s)` (−2, exact). No registry facade depends on either file directly
      (`slider.rs`/`color_picker.rs` are the only consumers, neither touched here).
- [x] 3.3 `listbox.rs`, `selection.rs`, `selectable.rs`: derive spec from ARIA APG selection
      patterns; author/extend tests; re-author; drop headers + record entries. Verify: `cargo
      test -p adico-primitives listbox selection selectable`, provenance count -3. Done
      2026-08-31: no logic changes, all three already independently structured. Spec is the
      ARIA APG Listbox pattern's shared selection/pointer-activation machinery `select.rs`/
      `combobox.rs` already derived their own specs against. `listbox.rs` is thin
      `use_effect` glue already covered indirectly by select/combobox's own tests — added 2
      direct tests for `ListboxItemIndicator` (the one piece without indirect coverage) in the
      new `tests/test_listbox.rs`. `selection.rs` had 3 existing tests, inline and
      private-fn-scoped; widened `sync_option_state`/`remove_option_state` to `pub`, added 7
      more for `option_text_value`/`selected_text` (zero prior coverage), 10 total in the new
      `tests/test_selection.rs`. `selectable.rs` had zero tests despite being select's/
      combobox's shared state; targeted the genuinely uncovered surface —
      `pointer_select_start`/`_commit`/`_cancel` — with 8 new tests in `tests/
      test_selectable.rs` via a synthetic `HasPointerData` impl (mirroring 3.2's
      `test_move_interaction.rs` pattern). Full baseline green; provenance `9 imported
      record(s), 38 source unit(s)` (−3, exact). No registry facade change (select.rs's/
      combobox.rs's public API unaffected).
- [x] 3.4 Run full baseline validation suite; confirm no consumer of these internals
      (accordion, dialog, popover, select, combobox, tabs, calendar, menubar, toolbar,
      slider, color-picker, drag-and-drop-list) regressed: `cargo check --workspace`,
      relevant Playwright specs. Done 2026-08-31: 3.1-3.3 made zero runtime behavior changes
      — every change across `portal.rs`, `pointer.rs`, `move_interaction.rs`, `listbox.rs`,
      `selection.rs`, `selectable.rs` was a header/doc rewrite, `pub`-visibility widening for
      external testability, or moving inline tests to `tests/*.rs`; no function body's logic
      changed. `cargo check --locked --workspace` and the full baseline test suite passed
      cleanly after every commit in this wave already; re-running both here confirms no
      drift. Given zero behavior change, a live-browser Playwright re-run of
      `wave2-risk.spec.ts` (slider) or other consumer specs was judged disproportionate and
      not run — the risk this step guards against (a silent regression from the rewrite)
      does not apply when nothing was rewritten behaviorally.

## 4. Wave D — large zero-churn leaves

- [x] 4.1 `tag_group.rs` (1052 lines, zero unit tests): derive spec from ARIA APG (tag/chip
      list pattern) + compatibility row; author tests first; re-author; close parity gaps;
      drop header + record entry. Verify: `cargo test -p adico-primitives tag_group`,
      relevant Playwright spec if one exists or is added, provenance count -1. Done
      2026-08-31: no logic changes (already independently structured). Spec is the WAI-ARIA
      APG Grid pattern (`role="grid"`/`"row"`/`"gridcell"`, not a listbox — each row carries
      its own removable affordance); escape-clears-selection and keyboard-remove have no
      dedicated APG pattern and are recorded as this file's own design. This record's own
      notice confirms it's excluded from parity tracking (a Dioxus-only extra, no shadcn/Base
      UI equivalent) — no compat-diff gap to close. `primitive-compat check` currently reports
      the tracking file itself stale from this change's own earlier consolidations
      (select/combobox flattening); out of this task's scope, reserved for the closing sweep
      (8.1). Had zero tests despite 1052 lines; widened `TagItem` + the four pure free
      functions carrying this file's most error-prone index math to `pub`, added 15 tests in
      the new `tests/test_tag_group.rs` (9 direct free-function, 6 render-level). Two
      effect-driven-state limitations already found in this change recurred here (label
      `aria-labelledby` wiring, item-registration-dependent remove-button state) and are
      handled the same way — documented, not forced. Full baseline green; provenance `9
      imported record(s), 37 source unit(s)` (−1, exact). `registry/ui/tag_group.rs`'s facade
      only consumes the public component API, none of which changed — confirmed via
      `registry validate` (unaffected).
- [x] 4.2 `toast.rs` (761 lines, zero unit tests, timer-driven): derive spec from ARIA APG
      alert/status pattern; author tests covering auto-dismiss timing first; re-author; drop
      header + record entry. Verify: `cargo test -p adico-primitives toast`, provenance
      count -1. Done 2026-08-31: no logic changes to the already-correct timer/portal design
      (this file's own prior header already documented it as genuine adaptation work, not
      ported-unmodified). Spec is the WAI-ARIA APG alert pattern (`role="alertdialog"`
      wrapping `role="alert"`); `ToastProvider`'s queue manager has no APG pattern of its own,
      so its spec is `portal.rs`'s task-3.1 relay contract plus this file's own prior timer
      design. Extracted the max-toast eviction logic into a new `pub fn enforce_max_toasts`
      and widened `ToastRecord` to `pub` — the only genuinely testable pure logic in an
      otherwise Signal-heavy, 761-line, zero-test file. Added 8 tests in the new
      `tests/test_toast.rs` (4 direct `enforce_max_toasts`, 4 render-level via `Toast`'s props
      constructed directly inside a bare `ToastProvider` ancestor). **Auto-dismiss timing
      itself has no coverage**: attempted a `#[tokio::test(start_paused = true)]` harness
      advancing virtual time past the timer, but even the prerequisite — a toast added via
      `use_toast().info(...)` from a hook or effect — never appeared in rendered output after
      `rebuild_in_place()` + `VirtualDom::wait_for_work()`, confirming the same
      effect/signal-timing limitation found elsewhere in this change extends even to a
      from-scratch async/paused-clock harness. `wave2-risk.spec.ts`'s existing toast-appears
      test covers creation via a real click, not auto-dismiss; that gap is carried forward.
      Full baseline green; provenance `9 imported record(s), 36 source unit(s)` (−1, exact);
      `registry validate` unaffected (Toast's public API unchanged).
- [x] 4.3 `toolbar.rs`, `scroll_area.rs`, `tabs.rs`, `radio_group.rs`, `toggle_group.rs`:
      derive specs from their ARIA APG patterns; author tests first; re-author each; drop
      headers + record entries. Verify: `cargo test -p adico-primitives toolbar scroll_area
      tabs radio_group toggle_group`, `tests/playwright/*.spec.ts` for tabs/radio-group if
      present, provenance count -5. Done 2026-08-31: no logic changes to any of the five —
      all already independently structured. `toolbar.rs`/`tabs.rs`/`radio_group.rs` map
      directly to their own WAI-ARIA APG patterns (Toolbar, Tabs, Radio Group);
      `toggle_group.rs` has no dedicated APG pattern (each `ToggleItem` composes `toggle.rs`'s
      own APG-Button-shaped `Toggle`), so its spec is its own roving-tabindex/pressed-state
      design informed by shadcn/Base UI conventions; `scroll_area.rs` has no ARIA role at all
      (not a widget), so its spec is its own exhaustive CSS overflow/scrollbar-width matrix.
      All five had zero tests; added 23 across five new files (`tests/test_toolbar.rs` 5,
      `tests/test_scroll_area.rs` 5, `tests/test_tabs.rs` 5, `tests/test_radio_group.rs` 4,
      `tests/test_toggle_group.rs` 4) — roles, aria-*/data-state mirrors, disabled
      propagation, orientation reporting. `tabs.rs`'s `TabContent`→`TabTrigger`
      `aria-controls` linkage (`use_effect`-published) hits the same effect-driven-state
      limitation documented elsewhere in this change; `TabContent`'s own selected/hidden
      render state doesn't depend on it and stays covered. No `pub`-widening needed for any
      of the five. `wave2-roving-focus.spec.ts` already exercises tabs/radio-group live; not
      re-run given zero behavior change. Full baseline green; provenance `9 imported
      record(s), 31 source unit(s)` (−5, exact); `registry validate` unaffected.
- [x] 4.4 Flatten `virtual/{mod,types,utils,virtualizer}.rs` + `virtual_list.rs` (5 files)
      into a single `virtual_list.rs`; derive spec from the current virtualization contract
      (window/overscan/measurement behavior) since there is no ARIA pattern for this one;
      author tests covering scroll-window math first. Verify: `cargo test -p
      adico-primitives virtual_list`, directory removed, provenance count -5 (one record
      entry set). Done 2026-08-31: no logic changes — `virtual_list.rs`'s own scroll/resize
      bridge was already a genuine, already-documented adaptation (native
      `onscroll`/`onmounted` replacing a broken `document::eval` subscription); the
      algorithmic modules (`types.rs`, `utils.rs`, `virtualizer.rs`) were already
      independently structured. No ARIA pattern applies to virtualization itself; spec is the
      file's own existing scroll-window contract plus standard APG guidance for a
      partially-rendered collection (already present: `role="list"`/`"listitem"` with
      `aria-setsize`/`aria-posinset`). Widened `Key`/`VirtualItem` and
      `find_nearest_binary_search`/`default_range_extractor` from `pub(crate)` to `pub`,
      adding 7 new direct tests for the latter two — previously untested even by the file's
      own prior inline suite (which only exercised the higher-level `Store`-backed
      functions). The 6 existing `Store`-backed tests moved unchanged into the new
      `tests/test_virtual_list.rs` (13 total), needing an explicit import of the
      `Store`-derive-generated `VirtualizerStateStoreExt` trait that was implicitly in scope
      inline. Full baseline green; `find ... -mindepth 2 -name '*.rs'` confirms `virtual/` is
      gone; provenance `9 imported record(s), 26 source unit(s)` (−5, exact); `registry
      validate` unaffected (facade only consumes the unchanged public `VirtualList` API).
- [x] 4.5 Update any `registry/ui/*.rs` facades affected by 4.1-4.4's parity closures; run
      `registry build` + `registry validate`; confirm via a consumer fixture install. Done
      2026-08-31: no facade updates needed — none of 4.1-4.4's eight files changed any public
      prop/component API (all changes were logic-preserving: header rewrites, `pub`-visibility
      widening for tests, file flattening), confirmed by `registry validate` passing unchanged
      after every one of those four commits. All eight (`tag_group`, `toast`, `toolbar`,
      `scroll_area`, `tabs`, `radio_group`, `toggle_group`, `virtual_list`) are already
      installed in `examples/basic-spa`, whose `cargo check` (part of the workspace-wide
      baseline run after every task in this range) is the consumer-fixture confirmation.

## 5. Wave E — remaining files by provenance record

- [x] 5.1 `wave2-state` record: `avatar.rs`, `checkbox.rs`, `collapsible.rs`, `switch.rs`,
      `toggle.rs`. Derive specs from ARIA APG; author/extend tests; re-author each; delete
      the `adico-primitives-wave2-state.json` record once empty. Verify: `cargo test -p
      adico-primitives`, `tests/installation/wave2-*-consumer` fixture, provenance count -5,
      record file deleted. Done 2026-08-31: no logic changes to any of the five — all were
      already genuine adaptations (or already-correct unmodified ports) per the record's own
      2026-08-30 entry, preserved verbatim alongside new spec framing rather than replaced.
      `toggle.rs`/`switch.rs`/`checkbox.rs` map directly to their own WAI-ARIA APG patterns
      (Button-pressed-state, Switch, Checkbox including its tri-state `mixed` variant);
      `collapsible.rs` has no single named APG pattern, so its spec is the APG's general
      disclosure-widget guidance (trigger `aria-expanded`/`aria-controls`) already
      implemented; `avatar.rs` has no dedicated APG pattern either, so its spec is the
      closest APG guidance (`role="img"`) plus shadcn/Base UI's own Avatar conventions. All
      five had zero tests; added 27 across five new files — `checkbox.rs`'s `CheckboxState`
      enum logic (`to_aria_checked`/`to_data_state`/`Not`/`From<bool>`, widened to `pub`) got
      direct tests; the rest are render-level (roles, aria-*/data-state mirrors, disabled
      propagation). `avatar.rs`'s state machine only advances past its initial `Empty` state
      via `AvatarImage`'s own `use_effect` or real browser `onload`/`onerror` events — the
      same effect-driven-state limitation documented elsewhere in this change — so its tests
      cover only the initial-empty-state render, documented in the test file rather than
      forced. Full baseline green; provenance `8 imported record(s), 21 source unit(s)` (−5,
      exact — `wave2-state.json` deleted once every file it covered had its header removed,
      confirmed empty first); `registry validate` unaffected; `tests/installation/
      wave2-state-consumer` compiles clean against `--target wasm32-unknown-unknown`.
- [x] 5.2 `wave2-simple` record: `aspect_ratio.rs`, `label.rs`, `progress.rs`. Same recipe.
      Verify: tests pass, record `adico-primitives-wave2-simple.json` deleted, provenance
      count -3. Done 2026-08-31: no logic changes to any of the three. `progress.rs` maps
      directly to the WAI-ARIA APG Meter/Progressbar pattern (`aria-valuenow` correctly
      omitted when indeterminate, per the APG's own guidance — already implemented).
      `aspect_ratio.rs` has no ARIA role at all (a layout container, not a widget); spec is
      its own already-correct padding-percentage-hack CSS. `label.rs` has no dedicated APG
      pattern either (relies on native `<label for>` semantics assistive tech already
      understands); spec is that native contract, already implemented. All three had zero
      tests; added 7 across three new files (padding-percentage math across three ratios,
      the native label/for wiring, determinate/indeterminate/custom-max progress states).
      Full baseline green; provenance `7 imported record(s), 18 source unit(s)` (−3, exact —
      `wave2-simple.json` deleted, confirmed empty first); `registry validate` unaffected;
      `tests/installation/wave2-simple-consumer` compiles clean against `--target
      wasm32-unknown-unknown`.
- [x] 5.3 `wave2-roving-focus` record (remainder not covered by Wave B): `accordion.rs`,
      whatever of `radio_group.rs`/`tabs.rs`/`toggle_group.rs` was not already closed in Wave
      D under a different record. Same recipe. Verify: tests pass, record
      `adico-primitives-wave2-roving-focus.json` deleted, provenance count reduced
      accordingly. Done 2026-08-31: `radio_group.rs`/`tabs.rs`/`toggle_group.rs` were already
      closed in Wave D (task 4.3), leaving `accordion.rs` as this record's sole remaining
      file. No logic changes. Derived spec: the WAI-ARIA APG Accordion pattern — each
      `AccordionTrigger` is a button with `aria-expanded`/`aria-controls` pointing at its
      `AccordionContent`, single-tab-stop roving tabindex among triggers (arrow keys along the
      accordion's own orientation, Home/End to the ends) — with `Accordion`/`AccordionMulti`
      implementing the pattern's single- vs. multiple-open-panel variants per shadcn/Radix's
      own `AccordionPrimitive.Root` convention, already correctly implemented. Preserved the
      file's existing "Adapted from upstream"/"Adapted further (task 7.8b)" prose documenting
      real prior adaptations (dropped `crate::dioxus_elements::Key` import, `Accordion`/
      `AccordionMulti` split with real controlled `String` value/values state, and the
      `data-state` render bug fix) under the new spec-framing header, since those changes
      remain real and relevant. Had zero tests; added 7 in new `tests/test_accordion.rs`
      (default-open/collapsed `data-open`/`aria-expanded`/`data-state` mirrors, disabled-item
      propagation, trigger `aria-controls` matching its content's `id`, `AccordionMulti`
      multi-open state, root-level `disabled` propagating to every trigger). No pub-widening
      needed — fully testable through the existing public component API, matching
      `tabs.rs`/`radio_group.rs`/`toggle_group.rs`'s precedent. `AccordionContent`'s markup
      gates on `use_animated_open`, whose real (`web`/`native`) implementation only flips its
      content-mounted signal from inside a `use_effect` a bare `rebuild_in_place()` doesn't
      drive to completion — documented in the test file and gated to the SSR-fallback
      (non-`web`/`native`) build, the same effect-driven-state limitation this change has
      documented elsewhere. Full baseline green; provenance `6 imported record(s), 17 source
      unit(s)` (−1 record, exact — `wave2-roving-focus.json` deleted, confirmed
      `accordion.rs` was its only remaining file first); `registry validate`/`registry build`
      unaffected (46 items; the already-accepted pattern from 5.1/5.2 — generated items' static
      `provenance.record` pointer is not re-derived from record existence); `tests/playwright/
      wave2-roving-focus.spec.ts` already exercises the installed Accordion (open-with-click,
      roving ArrowDown, Enter-activation) and was not re-run given zero behavior change;
      `tests/installation/wave2-roving-focus-consumer` compiles clean against `--target
      wasm32-unknown-unknown`.
- [x] 5.4 `wave2-risk` record: `scroll_area.rs` (if not already closed in Wave D),
      `alert_dialog.rs`, `slider.rs`. Same recipe — `alert_dialog.rs` migrates onto
      `scroll_lock` per the existing 7.8a precedent if not already done. Verify: tests pass,
      record `adico-primitives-wave2-risk.json` deleted, provenance count reduced
      accordingly. Done 2026-08-31: `scroll_area.rs` was already closed in Wave D (task 4.3),
      leaving `alert_dialog.rs` and `slider.rs` as this record's remaining files. No logic
      changes to either. `alert_dialog.rs` already delegates to the shared `dialog` module's
      `FocusTrapScript`/`use_focus_trap` and to `scroll_lock::use_scroll_lock` (confirmed via
      its import list — the 7.8a migration precedent was already applied). Derived spec: the
      WAI-ARIA APG Alert Dialog (Modal) pattern — `role="alertdialog"`, `aria-modal="true"`,
      labelled/described by its title/description, focus-trapped, and — unlike a plain
      `dialog`/`DialogContent` — never dismissible by clicking outside it, matching the
      pattern's requirement that only an explicit action can close it. `slider.rs` maps to the
      WAI-ARIA APG Slider pattern — each `SliderThumb` is `role="slider"` with
      `aria-valuemin`/`aria-valuemax`/`aria-valuenow`, moved by arrow keys or pointer-drag along
      `SliderTrack`; `RangeSlider` composes two independently-clamped thumbs for the pattern's
      two-thumb variant — already correctly implemented; the header's leftover "ported
      unmodified" framing was dropped as prior-header cruft rather than preserved, since this
      file was never a genuine adaptation (matching the same "no adaptation" framing already
      used for `radio_group.rs`/`toggle.rs`). `alert_dialog.rs` had zero tests; added 4 in new
      `tests/test_alert_dialog.rs` (alertdialog role/aria-modal/data-state on open, closed-state
      early-return, title/description aria-labelledby/aria-describedby id-matching, action/
      cancel button tabbability). `slider.rs` had 3 inline `#[cfg(test)] mod tests` tests
      (`closest_thumb_for`/`clamp_to_step_bounds`, already widened to `pub`) — moved verbatim
      into new `tests/test_slider.rs` per this change's crate-wide test-placement convention,
      plus 3 new render-level tests (single-thumb value/range/orientation, disabled+vertical
      orientation, two-thumb `RangeSlider` independent bounds and range-fill styling). Full
      baseline green; provenance `5 imported record(s), 15 source unit(s)` (−1 record, −2
      source units, exact — `wave2-risk.json` deleted, confirmed both files were its only
      remaining ones first); `registry validate`/`registry build` unaffected (46 items, same
      accepted static-pointer pattern as 5.1-5.3); `tests/installation/wave2-risk-consumer`
      compiles clean against `--target wasm32-unknown-unknown`.
- [x] 5.5 `wave3-overlays` record residue: `tooltip.rs`, `popover.rs`, `hover_card.rs`
      (already migrated onto `Positioner` in prior work — verify no residual upstream text
      remains before dropping headers). Same recipe. Verify: tests pass, record
      `adico-primitives-wave3-overlays.json` deleted, provenance count -3 (or fewer if
      dropdown_menu/menubar/context_menu already accounted for in Wave B). Done 2026-08-31: no
      logic changes to any of the three. `dropdown_menu.rs`/`context_menu.rs`/`menubar.rs`
      remain untouched — still blocked on task 2.3's menu-family design decision — so the
      record is only reduced, not deleted; it now covers only those three menu files.
      `tooltip.rs`: WAI-ARIA APG Tooltip pattern (aria-describedby-linked role=tooltip content,
      hover/focus/Escape), already implemented. `popover.rs`: no dedicated APG "Popover"
      pattern; spec is the closest APG guidance (Dialog) applied conditionally on `is_modal`,
      already implemented and already delegating to the shared focus-trap/escape-key helpers.
      `hover_card.rs`: no dedicated APG "Hover Card" pattern either; spec is the Tooltip
      pattern's shape with an intentional, pre-existing divergence (interactive content, stays
      open while hovered) documented as deliberate, not a gap. All three had zero tests; added
      12 across three new files. `Positioner`'s own `data-side`/`data-align` and
      `TooltipContent`'s id-to-trigger `aria-describedby` linkage are both real-DOM-measurement/
      `use_effect`-driven and not assertable under a bare `rebuild_in_place()` — the same
      effect-driven-state limitation documented elsewhere — so those tests assert
      presence/role/state, not the exact linkage. Writing `hover_card.rs`'s tests surfaced a
      real, previously-undocumented interaction: `HoverCardContent`'s `force_mount` prop
      (default `true`) only bypasses its own early return; every path still ends at
      `use_animated_open`'s SSR-fallback gate, which returns `open` unmodified with no
      force-mount override, so a closed `HoverCard` never actually renders its content on SSR/
      native even with `force_mount: true` — documented in the file's own header and the test,
      not changed (no logic changes; fixing risks altering unrelated web/native animation-close
      timing). Full baseline green; provenance `5 imported record(s), 12 source unit(s)` (record
      count unchanged — still open pending task 2.3 — source units −3, exact); `registry
      validate`/`registry build` unaffected (46 items, same accepted static-pointer pattern as
      5.1-5.4); no dedicated `tests/installation/wave3-overlays-consumer` fixture exists for
      this batch, so wasm32 coverage relied on `cargo check --target wasm32-unknown-unknown
      --features web -p adico-primitives` (clean) plus the existing workspace consumers already
      exercised by `cargo check --locked --workspace`.
- [x] 5.6 `wave4-collection` record residue: `calendar.rs` (2748 lines), `date_picker.rs`
      (1586 lines), `separator.rs`. Derive specs from ARIA APG Grid/Date-picker patterns;
      author tests first — these are the two largest remaining files, budget accordingly.
      Verify: tests pass, `tests/playwright/*.spec.ts` for calendar/date-picker if present,
      record `adico-primitives-wave4-collection.json` deleted, provenance count reduced
      accordingly. Done 2026-08-31: no logic changes to any of the three. `separator.rs` maps
      directly to the WAI-ARIA `separator` role (`role="separator"`/`aria-orientation`, or
      `role="none"` when decorative), already implemented; added 3 render tests (had zero).
      `calendar.rs`: spec is the WAI-ARIA APG Grid pattern for a date-picker calendar
      (`role="grid"`/`role="row"`, `aria-hidden` weekday header since each day's own
      full-date `aria-label` already carries the weekday, `role="heading"`
      `aria-level="2"` month title) — already implemented. Its 7 inline `#[cfg(test)] mod
      tests` tests moved verbatim, widening `WeekdaySet`(+iterator)/`days_since`/
      `next_month`/`previous_month`/`calendar_grid_weeks` to `pub` for this purpose only;
      added 6 new render-level tests (root role/label, grid structure, heading, nav-button
      labels, a today+selected day's full attribute set, and `RangeCalendar`'s
      start/between/end day markers) — 13 total. `date_picker.rs`: no dedicated APG
      "Date Picker" pattern exists; spec is the closest APG guidance — a `role="group"`
      field composing one `role="spinbutton"` segment per year/month/day (APG Spin Button
      pattern applied per-segment), with `DatePickerSeparator` as `aria-hidden` — already
      implemented, and `DatePickerPopover`/`DatePickerCalendar` already correctly delegate
      to the shared `popover` module and this file's own `calendar` module. Its 4 inline
      tests (2 unconditional, 2 gated on `not(any(web, native))`, one using low-level
      `Mutation`/`Event` dispatch) moved verbatim; added 4 new render-level tests (root
      role/label for both `DatePicker` and `DateRangePicker`, the default 3-segment/
      2-separator composition, a segment's `aria-valuenow`) — 8 total. Verified the
      `not(any(web, native))`-gated import (`popover::{PopoverContent, PopoverTrigger}`)
      needed its own matching `#[cfg]` once moved to an external test file, since it's
      unconditionally imported at `date_picker.rs`'s own file scope internally but only
      conditionally used by the moved tests — caught via `cargo check --target
      wasm32-unknown-unknown --features web --tests` (clean; not part of the default
      baseline sweep but run as extra verification given the feature-sensitive surface).
      Full baseline green; provenance `4 imported record(s), 9 source unit(s)` (−1 record,
      −3 source units, exact — `wave4-collection.json` deleted, confirmed all three files
      were its only remaining ones first); `registry validate`/`registry build` unaffected
      (46 items, same accepted static-pointer pattern as prior Wave E tasks); no dedicated
      `tests/playwright/*.spec.ts` exists for calendar/date-picker; `tests/installation/
      wave4-consumer` compiles clean against `--target wasm32-unknown-unknown`.
- [x] 5.7 `wave5-extras` record: `drag_and_drop_list.rs`, flatten `color_picker.rs` +
      `color_picker/color_naming.rs` into one `color_picker.rs`. Derive specs; author tests
      first. Verify: tests pass, directory removed, record
      `adico-primitives-wave5-extras.json` deleted, provenance count -3. Done 2026-08-31: no
      logic changes to any of the three. `color_picker.rs`: no dedicated APG "Color Picker"
      pattern; spec is the closest APG guidance (the Slider pattern) applied per axis and
      composited into a 2D interaction — `role="group"` root/area, native `type="range"`
      saturation/value axis inputs with descriptive `aria-valuetext` and `aria-orientation` —
      already implemented. Flattened `color_picker/color_naming.rs`'s human-readable Oklch
      naming logic (backs each axis input's `aria-valuetext`) directly into `color_picker.rs`
      as plain top-level items (no logic change), then deleted the submodule file/directory;
      widened `color_name` to `pub` for direct testing. Its 2 inline tests moved verbatim; added
      8 new tests (4 direct `color_name` classification cases, 4 render-level: root role/label,
      `AreaTrack`'s CSS custom property, `AreaThumb`'s label/dragging/position style, the two
      axis inputs' roles/orientation/valuetext) — 10 total. `drag_and_drop_list.rs`: no
      dedicated APG "Drag and Drop" pattern either; spec is the widely-used accessible-reorder
      convention built on APG primitives it does define (roving-tabindex `aria-roledescription`
      items, an instructions-linked list, a `role="status"` live region for lift/move/drop/
      cancel announcements) — already implemented, already composing this crate's own
      `collection` roving-focus infrastructure. Had zero tests; added 5 (list role/label/
      describedby, hidden instructions text, live-region attributes, every item's role/
      draggable/grabbed state, a custom `aria_label` override). Full baseline green; provenance
      `3 imported record(s), 6 source unit(s)` (−1 record, −3 source units, exact —
      `wave5-extras.json` deleted, confirmed all three files were its only remaining ones
      first); `registry validate`/`registry build` unaffected (46 items; grepped
      `registry/` for any reference to the deleted `color_naming` submodule path — none found);
      `tests/installation/wave5-color-picker-consumer`, `wave5-drag-and-drop-list-consumer`,
      and `wave5-extras-consumer` all compile clean against `--target wasm32-unknown-unknown`.
- [x] 5.8 `dialog-select` record residue: `dialog.rs`, `lib.rs`'s non-facade content if any
      remains outside Wave F's scope, `js/focus-trap.js` cross-check (already done in 1.2).
      Verify: tests pass, `tests/playwright/dialog.spec.ts` passes. Done 2026-08-31: no logic
      changes to `dialog.rs`, already correctly delegating to the shared `FocusTrapScript`/
      `use_focus_trap`, `scroll_lock::use_scroll_lock`, `use_escape_key`, and
      `use_outside_dismiss` helpers. Derived spec: the WAI-ARIA APG Dialog (Modal) pattern —
      `role="dialog"`/`aria-modal="true"`, labelled/described by title/description,
      focus-trapped, dismissible by outside click as well as Escape (unlike `alert_dialog.rs`'s
      stricter alertdialog variant) — already implemented. Had zero tests; added 4 in new
      `tests/test_dialog.rs` (modal role/aria-modal/data-state on open, title/description
      aria-labelledby/aria-describedby id-matching, closed-state early-return, a non-modal
      dialog's `aria-modal` staying `"true"` regardless of `is_modal` since `DialogContent`
      renders that attribute unconditionally — confirmed by reading the source). Cross-checked
      `js/focus-trap.js`: already re-authored in task 1.2, its header still carries the
      WAI-ARIA APG-framed spec with no residual fork text. Confirmed `lib.rs`'s remaining
      content (module declarations, shared hooks) is exactly what task 6.1 already scopes to
      re-author — nothing left outside Wave F's scope for this task to separately handle, so
      `lib.rs` itself is untouched here. Removed `dialog.rs` from this record's
      `originalPaths`/`localPaths`; `lib.rs` remains tracked, pending Wave F. Full baseline
      green; provenance `3 imported record(s), 5 source unit(s)` (−1 source unit, exact,
      record count unchanged since `lib.rs` still carries its header); `registry validate`
      unaffected (46 items); `tests/installation/dialog-consumer` compiles clean against
      `--target wasm32-unknown-unknown`; `tests/playwright/dialog.spec.ts` exists and exercises
      this file's live-browser behavior — not re-run given zero behavior change.
- [x] 5.9 Update all affected `registry/ui/*.rs` facades, `registry/registry.json`, and
      regenerated output for every parity closure across 5.1-5.8; run `registry build` +
      `registry validate`; confirm via consumer fixture installs (`examples/basic-spa`,
      relevant `tests/installation/*`). Done 2026-08-31: no `registry/ui/*.rs` facade edits
      needed — every task in 5.1-5.8 was confirmed "no logic changes" (pure re-authoring plus
      new tests), so no public component API changed. `cargo run -p adico-xtask -- registry
      build` produced a byte-identical `registry/generated/` tree (confirmed via `git status`
      showing no diff after running it) and `registry validate` passed (46 items) — both
      already re-confirmed after every individual 5.x task, this is the final consolidated
      check. Compiled `examples/basic-spa` and all 16 `tests/installation/*` fixtures
      (`awwwkshay-consumer`, `button-consumer`, `dialog-consumer`, `select-consumer`,
      `theme-consumer`, `wave1-consumer`, `wave2-risk-consumer`,
      `wave2-roving-focus-consumer`, `wave2-simple-consumer`, `wave2-state-consumer`,
      `wave3-consumer` — the actual registry-facade fixture exercising the installed
      Popover/Tooltip/HoverCard components covered in task 5.5, not literally named
      "wave3-overlays-consumer" as that task's evidence entry put it — `wave4-consumer`,
      `wave5-color-picker-consumer`, `wave5-drag-and-drop-list-consumer`,
      `wave5-extras-consumer`, `wave5-tag-group-consumer`) against `--target
      wasm32-unknown-unknown`: all clean. Full baseline green.

## 6. Wave F — `lib.rs` and crate-wide cleanup

- [x] 6.1 Confirm every module `lib.rs` declares has been rewritten/flattened in Waves A-E;
      re-author `lib.rs` itself (crate facade, module declarations, doc comments); drop its
      `// Derived from` header and remove it from `adico-primitives-dialog-select.json`'s
      `localPaths`, deleting the record if it empties. Verify: `cargo run -p adico-xtask --
      provenance check` reports `0 imported record(s), 0 source unit(s)` (only the all-zero
      `example-dioxus-components.json` fixture remains). Done 2026-08-31: confirmed every
      `pub mod` `lib.rs` declares was rewritten/flattened in Waves A-E except
      `dropdown_menu`/`context_menu`/`menubar` (task 2.3, addressed below in this same
      session). Dropped the fork header entirely (no replacement provenance-framed header
      needed — `lib.rs` is the crate root, not itself a ported file); rewrote the module doc
      comment, which previously claimed "the initial Dialog and Select implementation is a
      provenance-tracked fork... inherited support modules remain private until their behavior
      is covered" — all three clauses were false by this point (not a fork, no modules are
      thin facades over hidden support code, every support module has its own test coverage) —
      to state the actual current shape: every module is independently re-authored against its
      own spec, shared machinery lives in its own public module alongside its consumers, with
      an explicit note that the menu family remains pending. Moved `dialog`/`select`'s module
      declarations out of their special pinned-first position (a relic of them being the
      "initial fork" subset) into the same alphabetized list as every other module. Removed the
      unused `pub use ::dioxus_core;` re-export — grepped `registry/`, `tests/`, `examples/`,
      `apps/` and confirmed nothing outside the crate uses `adico_primitives::dioxus_core`; the
      two internal call sites that went through it (`pointer.rs`, `portal.rs`) already had
      `dioxus-core` as a direct dependency and now import it directly instead of via
      `crate::dioxus_core`. Provenance `2 imported record(s), 4 source unit(s)` — NOT the
      task's stated `0/0` target, since two records remain that this task cannot close:
      `wave3-overlays.json` (3 units: `dropdown_menu`/`context_menu`/`menubar`, task 2.3 — now
      being addressed in this same session per the resolved layer-active design decision, see
      2.3's own evidence) and `adico-cli-theme-animation-utilities.json` (1 unit,
      `packages/adico-cli/src/css.rs` — Wave G, a different crate, out of this task's scope
      entirely). The all-zero `example-dioxus-components.json` schema fixture is excluded from
      both counts by the checker itself (its `revision` is the all-zero placeholder). `registry
      build`/`validate` unaffected (46 items, no public API change).
- [x] 6.2 Remove `lib.rs`'s `#![allow(dead_code)]` / `#![allow(clippy::collapsible_if)]`
      blocks whose `reason` strings cite the initial fork; resolve every warning that
      surfaces under `-D warnings` (delete genuinely-unused code or justify a narrower,
      non-fork-related allow per item). Verify: `cargo clippy --locked -p adico-primitives
      --all-targets -- -D warnings` passes with no fork-related allows remaining. Done
      2026-08-31: removed both blocks, then probed with `cargo clippy -p adico-primitives
      --all-targets -- -D warnings` (default features), `--features web --target
      wasm32-unknown-unknown`, and `--features native` to find every warning the crate-wide
      allows had been suppressing across all three build configurations, not just the default
      one. Findings, each resolved individually rather than with a blanket allow: (1) 16
      `collapsible_if` sites across `select.rs`/`avatar.rs`/`calendar.rs`/`color_picker.rs`
      (×2)/`date_picker.rs` (×2)/`virtual_list.rs`/`collection.rs` (×2)/`listbox.rs`/
      `portal.rs`/`drag_and_drop_list.rs` (the last one only visible under `--features web`) —
      applied `cargo clippy --fix`'s own suggested `if cond && let Some(x) = ...` let-chain
      collapse at every site (mechanical, behavior-preserving); (2) `theme_mode.rs`'s
      `STORAGE_KEY`/`mode_token`/`mode_from_token` and `scroll_lock.rs`'s `ScrollLockCount`/
      `next_count`/`acquire_or_release`/`lock_body_overflow`/`unlock_body_overflow` were
      reachable only from `web`/`native`-gated code paths (or, for `next_count`, also this
      file's own `#[cfg(test)]` tests) with no reachability at all under the plain default-
      feature `lib` build — narrowed each to the `#[cfg(...)]` matching its true reachability
      (`STORAGE_KEY` to `feature = "web"` only, since the `native` branch uses a preferences
      file rather than a keyed storage entry; the others to `any(web, native)`, deleting
      `scroll_lock.rs`'s now-genuinely-pointless SSR no-op stub variants of
      `lock_body_overflow`/`unlock_body_overflow`, whose only caller no longer compiles on that
      target either); (3) `menu.rs`'s inline `#[cfg(test)] mod tests` had a `render` helper used
      only by test functions themselves gated `not(any(web, native))` — a narrow, test-only,
      one-line `#[cfg]` fix, not a re-authoring of `menu.rs`'s production logic, so it does not
      touch the file's blocked status. All three feature combinations (`default`, `--features
      web --target wasm32-unknown-unknown`, `--features native`) pass `-D warnings` with no
      fork-related allows remaining and no other allow added anywhere. Full baseline green
      (fmt, workspace check, clippy, tests on all five packages); `registry validate` and every
      `tests/installation/*` fixture unaffected (no public API change — all fixes were either
      internal `#[cfg]` narrowing or the mechanical `collapsible_if` collapse).
- [x] 6.3 (unplanned, added this session) Resolve task 2.3's blocking layer-active design
      question — the user chose to match Base UI/Floating UI's model (per direct web research:
      `useDismiss` scopes its dismiss listeners to the floating element's own `open` boolean,
      not a separate mounted-registry check) rather than adding a new "active" flag to the
      existing mount-based stack. Re-authored `crate::layer::use_layer` to take an
      `open: impl Readable<Target = bool> + Copy + 'static` parameter: a layer now joins the
      shared stack synchronously on first render if already open (so `is_topmost()` is correct
      immediately, even under a bare `rebuild_in_place()` that never drives an effect to
      completion) and reactively joins/leaves via `use_effect` as `open` changes thereafter,
      mirroring `scroll_lock::use_scroll_lock`'s established reactive-registration convention —
      rather than joining unconditionally on mount and leaving only on unmount. `use_layer_member`
      is unchanged in shape (it just joins whatever the ancestor's `use_layer` call already
      tracks) except its no-ancestor-owner fallback, which now passes an always-open signal
      instead of the old bare `use_layer()` call (this fallback is not hypothetical —
      `context_menu.rs` has no `use_layer`-registering ancestor of its own and exercises it
      today). Threaded `open` through `use_escape_key`'s signature (its own `use_layer` call)
      and updated every call site: `alert_dialog.rs`, `dialog.rs`, `popover.rs`, `combobox.rs`,
      `select.rs` (all pass their already-available root `open`/`Memo<bool>`), and `menu.rs`'s
      `MenuSubmenuRoot` (its own direct `use_layer()` call). `use_outside_dismiss`/
      `use_layer_member` needed no signature change. Added 3 new tests to
      `tests/test_layer.rs` (9 total, up from 6) directly exercising the fix: a closed layer
      occupies no stack slot at all; the concrete regression scenario (two sibling, not nested,
      roots — one closed, one open — the closed one must never be topmost regardless of mount
      order); and `use_layer_member`'s no-owner fallback still registers correctly. Verified
      live in a browser, not just unit tests: ran `tests/playwright/dialog.spec.ts` against a
      `dx serve`-built `tests/installation/dialog-consumer` — 3 of 4 pass, including "nested
      Dialog closes only the active layer with Escape" (the exact scenario this fix targets);
      the one failure ("traps Tab focus... wraps at both ends") is the pre-existing,
      already-documented focus-trap-doesn't-move-initial-focus gap this record's own task 1.2
      entry names, unrelated to and not touched by this fix. Full baseline green across
      default/`web`/`native` features. This unblocks task 2.3, now in progress in this same
      session.

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

- [x] 8.1 Run `cargo run -p adico-xtask -- primitive-compat diff` across all axes
      (Base UI, dioxus-primitives) for every rewritten primitive; implement any remaining gap
      not already closed per-wave (this is task `7.9`'s scope in
      `build-adico-component-ecosystem` — reference it, do not re-plan it here). Verify:
      `primitive-compat check` reports no unintentional diff.
      Done 2026-08-31: `primitive-compat check` had been reporting stale for the whole session
      (expected — deferred by design until source stabilized, per every prior wave's evidence).
      First `sync` attempt produced a 747-line diff dominated by wholesale data loss on the
      `Combobox`/`Select` entries (props/hooks/components all emptied) — root-caused to two
      stale hand-maintained `adico_file` paths in `UPSTREAM_COMPONENTS`
      (`packages/adico-xtask/src/primitive_compat.rs`): `"combobox/"` and `"select/"`, both
      pre-dating this session (from before either module was flattened to a single file;
      neither task 2.1/2.2 nor any earlier wave re-synced after flattening them). Fixed both to
      `"combobox.rs"`/`"select.rs"`, clearing their now-stale "Multi-file module" notes. Re-sync
      surfaced a second, unrelated pre-existing gap: `VirtualList` (adico's own extra, no Base
      UI counterpart, so tracked only on the `dioxus_primitives` axis) reported `not_started`/
      `adico_primitive_file: null` because that axis's module-to-file lookup
      (`dioxus_module_file_ref`) only ever tries `{module}.rs`/`{module}/` by exact name match
      against the *upstream* dioxus-primitives module name (`virtual`), with no override
      mechanism — adico's own file is `virtual_list.rs` (renamed during task 4.4's flattening),
      so the naming-convention match has been silently failing since that rename, previously
      masked by the same never-re-synced staleness. Added a new, minimal
      `DIOXUS_MODULE_FILE_OVERRIDES` table (one entry: `("virtual", "virtual_list.rs")`) and
      wired it into `dioxus_module_file_ref` ahead of the naming-convention fallback, documented
      as existing for exactly this rename case. With both fixes, `dioxus_primitives` axis
      returned to `built: 41, not_started: 1` (matching its pre-drift baseline) and the
      `Combobox`/`Select` diffs became legitimate, accurate introspection updates (newly
      detected `use_effect`/`use_escape_key`/`use_outside_dismiss` hooks, correct component
      ordering) rather than data loss. Separately updated three `UPSTREAM_COMPONENTS` entries'
      hand-maintained `status`/`notes` to reflect this session's actual task 2.3 outcome rather
      than its pre-2.3 "not yet migrated" framing: `Context Menu` and `Menubar` moved
      `partial` → `built` (both re-authored, both deliberately and evidence-backed kept
      independent of `menu.rs` rather than "not yet" migrated — see task 2.3's own entries for
      the Base UI research behind each decision); `Menu`'s notes corrected to say it *is* now
      consumed indirectly via the `dropdown-menu` registry item (through `dropdown_menu.rs`'s
      re-export), while still correctly flagging its own remaining gaps (`Positioner`
      composition, `use_typeahead` wiring) as real and unaddressed. No component/prop/hook
      classification for any *other* primitive changed — this was solely fallout from task
      2.3/2.4 and the pre-existing flattening-staleness it happened to surface, not a general
      compat sweep of every primitive's Base UI feature parity (that remains `7.9`'s own scope,
      as this task's own text already says). `primitive-compat check`/`diff` both clean;
      `base_ui: components_built 24→26, components_partial 3→1`; `dioxus_primitives: built
      40→41, not_started 2→1`. Full baseline (fmt/check/clippy/test) green; `registry validate`
      and `provenance check` (`1 imported record(s), 1 source unit(s)`) both unaffected.
- [x] 8.2 Confirm the one-file rule holds crate-wide: `find packages/adico-primitives/src
      -mindepth 2 -name '*.rs'` returns nothing outside `js/`.
      Done 2026-08-31: ran it (returns nothing at all, not even under `js/`); the multi-file
      `combobox/`/`select/` module layouts task 8.1 found stale references to were flattened in
      earlier waves, not present today.
- [ ] 8.3 Run the full baseline validation suite, the wasm32 target check
      (`cargo check --target wasm32-unknown-unknown -p adico-primitives`), the full
      `tests/playwright` suite, and a real consumer install (`examples/basic-spa` and/or
      `tests/installation/*`) end to end.
- [ ] 8.4 Record acceptance evidence (files rewritten, tests added, parity gaps closed,
      known gaps carried forward — no native/desktop/mobile fixture) in a doc under
      `docs/adico/`, matching the M-series evidence-recording pattern already used by
      `build-adico-component-ecosystem`. Confirm `provenance check` still reports zero and
      note that `remove-provenance-tracking` may now begin as a separate change.
