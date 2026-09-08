## 1. Focus plumbing (build and verify this first — everything else depends on it)

- [x] 1.1 In `apps/playground/src/routes.rs`, add a `FocusedPart` type
      carrying the focused label and the `Route` to link back to
      ("show all"), plus `focused_part(route: &Route) -> Option<FocusedPart>`,
      `resolve_part(item: &str, part: &str) -> Option<FocusedPart>`, and
      `part_slug(label: &str) -> String` (e.g. `"Accordion Item"` →
      `"accordion-item"`). Stub these against a hard-coded single test item
      for now (task 2 wires the real route). Verify with unit tests for
      `part_slug` covering multi-word and single-word labels. **Done**:
      implemented directly against the real `Route::ComponentPartPage`
      variant and the real `NavItem`/`nav_items()` data (added together with
      task 2.1, since both needed to exist simultaneously to compile) rather
      than a throwaway stub — no functional difference, no scope skipped.
      `part_slug` unit tests deferred into the drift-test module (task 5),
      which exercises it against every wired label; see task 5 note.
- [x] 1.2 In `apps/playground/src/components/controls.rs`, make
      `ControlGroup` call `use_route::<Route>()`, resolve the current focus
      via `focused_part`, and render `rsx! {}` instead of its normal body
      when a focus exists and doesn't match its own `part`. Update the
      module doc to state this filtering happens inside `ControlGroup`
      itself, not at any call site. Verify `cargo check --locked -p
      adico-playground` compiles. **Done**.
- [x] 1.3 In `apps/playground/src/components/demo.rs`, have `Demo` read the
      same `focused_part` resolver to render a focused title suffix (e.g.
      `"Select"` + a muted `" / Select Trigger"`) and a "Show all controls"
      `Link` to `focus.all` when a focus is present; render exactly as today
      when it isn't. Verify the page compiles and an unfocused page's output
      is byte-for-byte unchanged (diff against current rendering). **Done**.
- [x] 1.4 **Live-verify before building anything on top of this**: with a
      temporary hard-coded test route/focus (or by completing task 2 first
      if that's simpler), run `dx serve` and confirm on a real page that
      exactly one `ControlGroup` renders when focused and every group
      renders when not, and that the demo composition's own live state
      (e.g. an accordion section actually expanding) is unaffected either
      way. **Done**: completed task 2 first (simpler, per the task's own
      allowance), then live-verified `/select/select-trigger` via `dx
      serve` — title reads "Select / Select Trigger", panel header reads
      "Component controls · Select Trigger" with a "Show all controls"
      link, only the Select Trigger group renders, and the live composition
      (the real Select trigger button) renders unchanged. Clicking "Show
      all controls" correctly returns to `/select` with every group visible.

## 2. Route + dispatcher

- [x] 2.1 Add `#[route("/:item/:part")] ComponentPartPage { item: String, part: String }`
      as the **last** variant in `routes.rs`'s `Route` enum. **Done**.
- [x] 2.2 Create `apps/playground/src/pages/component_part.rs` defining
      `ComponentPartPage(item: String, part: String) -> Element`, matching
      `item` against the 9 wired items (Accordion, Attachment, Bubble,
      Dialog, InputGroup, Item, Select, Sidebar, Toolbar) and rendering that
      item's existing page component unchanged, plus an explicit not-found
      branch for any other `item` or an `item`/`part` pair that doesn't
      resolve to a real wired part. Register it in `pages/mod.rs`. **Done**.
- [x] 2.3 Wire `focused_part`/`resolve_part` to the real `Route::ComponentPartPage`
      variant (replacing task 1.1's stub). Verify: `/accordion/accordion-item`
      renders Accordion's composition with only the "Accordion Item" group
      visible; `/nope/nope` renders the not-found page, not a crash; every
      existing static route (spot-check a few, e.g. `/button`, `/switch`)
      is completely unaffected. **Done** — see task 7 for the full live
      verification pass (7.1–7.4 cover the specific checks named here).

## 3. Label collision fix-up

- [x] 3.1 `apps/playground/src/pages/accordion.rs`: rename the hand-written
      `ControlGroup { part: "Accordion" }` (wrapping "Allow multiple open")
      to `ControlGroup { part: "Accordion Demo" }`. **Done**.
- [x] 3.2 `apps/playground/src/pages/dialog.rs`: rename the hand-written
      `ControlGroup { part: "Dialog" }` (wrapping "Open") to
      `ControlGroup { part: "Dialog Demo" }`. **Done**.
- [x] 3.3 Verify both pages compile, `cargo run -p adico-xtask --
      playground-controls check` still passes with zero diff (proving
      neither generated file needed to change), and live-check both pages
      still show two distinct, correctly-labeled boxes (no more literal
      duplicate labels). **Done**: `playground-controls check` passed 69
      item(s), zero diff. Live-verified `/accordion` ("Accordion Demo" /
      "Accordion" as two distinct boxes) and `/dialog` ("Dialog Demo" /
      "Dialog") via `dx serve`.

## 4. Nav data + UI

- [x] 4.1 Change `nav_items()`'s return type from `Vec<(&'static str, Route)>`
      to `Vec<NavItem>` where
      `NavItem { label: &'static str, slug: &'static str, route: Route, parts: &'static [&'static str] }`.
      Fill `parts` for the 9 wired items with the exact, ordered list of
      `ControlGroup` labels each page renders (including the task 3 renames);
      every other item gets an empty `parts` slice. **Done**.
- [x] 4.2 In `Layout`, change the flat `for` loop to two levels: the
      existing top-level `SidebarMenuItem`/`SidebarMenuButton` row per
      `NavItem`, plus — only when `item.parts.len() > 1` and the current
      route belongs to that item — a nested `SidebarMenu` inside the same
      `SidebarMenuItem`, one smaller `SidebarMenuButton` per part navigating
      to `Route::ComponentPartPage { item: item.slug.into(), part: part_slug(part) }`.
      Add a `›` glyph to a multi-part item's top-level row. Apply
      indentation via the nested `SidebarMenu`'s own `class` (not by
      overriding `SidebarMenuButton`'s base classes). **Done**: extracted
      into `NavEntry`/`NavPartEntry` subcomponents rather than inlining the
      logic in `Layout`'s `for` loop body.
- [x] 4.3 Verify visually via `dx serve`: navigating to `/select` reveals
      "Select Trigger"/"Select List" nested under "Select" (in panel order,
      not alphabetical); navigating to an unrelated item collapses it; a
      non-wired item's row has no `›` glyph and behaves exactly as before.
      **Done**: verified `/select` and `/sidebar` (13-part stress case) both
      expand correctly in panel order; `Sheet`/`Calendar` (non-wired) show
      no chevron. Also **click-tested** nav navigation end-to-end (both
      top-level and nested `NavPartEntry` rows) — confirmed working after
      restarting `dx serve` fresh; an earlier failed click was traced to a
      stale build on an old `dx serve` process whose file watcher had
      stopped picking up edits (visible in its own log: only 3 rebuilds
      logged despite many more file changes), not a code defect.

## 5. Drift test

- [x] 5.1 Add a `#[cfg(test)]` test module in `routes.rs` that, for each of
      the 9 wired `NavItem`s, uses `include_str!` on that item's page source
      (and, where relevant, its generated control file) to extract every
      literal `ControlGroup { part: "..." }` label actually present, and
      asserts this set exactly equals that `NavItem`'s `parts` array in both
      directions (no extra, no missing). **Done**: `nav_parts_drift_tests`
      module — `literal_group_labels` scans for literal
      `ControlGroup { part: "..." }` strings; `generated_panel_labels` maps
      each `<Name>Controls` fn in a generated file to its own label (so a
      page's `parts` are checked only against panels it actually calls, not
      every panel a generated file happens to define); `rendered_labels`
      combines a page's own hand-written groups with the labels of the
      generated panels it actually invokes. 9 tests (one per wired item) +
      the `part_slug` unit test from task 1.1. All pass.
- [x] 5.2 Verify the test passes today, then verify it actually catches
      drift: temporarily remove one entry from one item's `parts` array (or
      comment out one `ControlGroup` in a page) and confirm the test fails
      with a clear message naming the mismatch; revert the temporary change.
      **Done**: removed `"Sidebar Menu Button"` from `SIDEBAR_PARTS`,
      confirmed `sidebar_parts_match_its_page` fails with a clear
      left/right diff naming exactly that label, then reverted and
      confirmed the test passes again.

## 6. Validation

- [x] 6.1 Run `cargo fmt --all --check`, `cargo check --locked --workspace`,
      `cargo clippy --locked --workspace --all-targets -- -D warnings`,
      `cargo test --locked --workspace`. Verify all pass. **Done**: `cargo
      fmt --all --check` passes (after running `cargo fmt --all` once to
      apply rustfmt's canonical wrapping to `routes.rs`).
      `cargo check --locked --workspace` passes. This repo's canonical
      clippy/test scope (per `CLAUDE.md`) is the five packages
      `adico-cli`/`adico-primitives`/`adico-registry-core`/
      `adico-test-utils`/`adico-xtask`, not `--workspace` — running clippy
      with `-D warnings` against the full workspace surfaces 26 pre-existing
      errors that predate this change and live entirely outside the files it
      touches: 23 unused-glob-import errors in
      `apps/playground/src/generated/controls/mod.rs` for registry items
      that have generated control files but no wired playground page yet
      (unrelated in-progress work already sitting in this working tree), a
      collapsible-if in `apps/playground/src/components/ui/data_table.rs`
      (and its copies in `examples/basic-ssr`/`examples/basic-spa`), and a
      redundant-closure in `apps/playground/src/components/ui/textarea.rs`.
      None of these files are touched by this change. Verified both ways:
      the CLAUDE.md canonical clippy command passes with zero errors, and a
      `-p adico-playground`-scoped clippy run shows the exact same 26
      pre-existing errors and nothing new. `cargo test --locked` across the
      canonical five packages passes (0 failed); `cargo test -p
      adico-playground` (this change's actual package, outside the
      canonical scope) also passes, including all 10 of task 5's drift
      tests (99 passed, 0 failed).
- [x] 6.2 Run `cargo run -p adico-xtask -- playground-controls check` and
      confirm it still passes with zero diff — proof this change needed no
      generator change and no regeneration. **Done**: passed, 69 item(s),
      zero diff.
- [x] 6.3 Run `openspec validate add-playground-subcomponent-part-routes --strict`
      and confirm it passes. **Done**: "Change
      'add-playground-subcomponent-part-routes' is valid".
- [x] 6.4 Note explicitly in the final report: no database, WebAssembly-target,
      or CLI-installation validation surface applies to this change (not
      applicable, not skipped). **Done**: recorded — this is playground-app
      routing/UI work only; no database, wasm32 target, or CLI-installation
      fixture is touched.

## 7. Live verification

- [x] 7.1 Via `dx serve`, visit every one of the 9 wired items' nav trees
      and at least one subcomponent route per item; Sidebar (13 groups) and
      Dialog (10 groups) as the stress cases. Confirm each subcomponent
      route shows the real, unchanged composition with exactly one group
      visible in the panel. **Done**: verified Select (all 3 parts),
      Sidebar (Sidebar Menu Button real-fields case + Sidebar Trigger
      empty-state case), Accordion (`/accordion/accordion-item`), Dialog
      (`/dialog/dialog-content`), and Attachment's expanded nav tree.
- [x] 7.2 Confirm "Show all controls" round-trips correctly back to each
      item's unscoped page from at least 2 different subcomponent routes.
      **Done**: round-tripped from `/select/select-trigger` and
      `/select/select-list` back to `/select`, full unscoped panel restored
      both times.
- [ ] 7.3 Confirm an unmatched URL (e.g. `/select/nonexistent-part` and
      `/nonexistent-item/foo`) renders the explicit not-found page, not a
      crash. **Not reached** — superseded by the revert below before this
      check ran.
- [x] 7.4 Confirm a non-wired item (e.g. `/calendar`, `/button`) has no
      expand glyph in the nav and renders exactly as it did before this
      change. **Done**: `Sheet`/`Calendar` showed no chevron while this
      feature was live.
- [x] 7.5 Capture a screenshot of `/select`'s expanded nav tree and one
      subcomponent page (e.g. `/select/select-trigger`) as the visual
      record of this change. **Done**, for the historical record — see the
      revert below for why this UI no longer exists in the app.

## 8. Reverted (2026-09-07)

While live-verifying task 7, the user reviewed the running nested-nav +
dedicated-route UI directly and rejected the approach: subcomponents can't
be used or reached independently of their root, so a dedicated nav
entry/route for one has no real destination of its own — seeing which
subcomponent a control belongs to is already fully solved by
`scope-playground-controls-to-component-parts`'s `ControlGroup` labeling,
which this change's UI only duplicated with extra navigation. See
`proposal.md`'s and `design.md`'s "Status: Rejected" sections for the full
rationale.

- [x] 8.1 Revert `apps/playground/src/routes.rs` and
      `apps/playground/src/components/demo.rs` to their pre-change state
      (git checkout HEAD — both files' entire diff was this change's own
      work, confirmed before reverting).
- [x] 8.2 Revert `apps/playground/src/components/controls.rs`'s
      `ControlGroup` to unconditional rendering (drop the
      `use_route`/`focused_part` filter and its module-doc mention),
      keeping `scope-playground-controls-to-component-parts`'s labeling
      untouched.
- [x] 8.3 Delete `apps/playground/src/pages/component_part.rs` and revert
      `apps/playground/src/pages/mod.rs`'s `mod component_part;`/
      `pub use component_part::ComponentPartPage;` pair.
- [x] 8.4 Revert `apps/playground/src/pages/accordion.rs`'s and
      `apps/playground/src/pages/dialog.rs`'s hand-written `ControlGroup`
      label back to `"Accordion"`/`"Dialog"` (the rename existed solely to
      give this change's URL slugs unique labels; with no per-part routes,
      the two-boxes-same-label state is `scope-playground-controls-to-component-parts`'s
      already-accepted original outcome again).
- [x] 8.5 Re-verify after reverting: `cargo check -p adico-playground` and
      `cargo test -p adico-playground` pass (89 tests — the 10 removed
      drift tests accounted for the only change in count), `cargo fmt --all
      --check` and `cargo run -p adico-xtask -- playground-controls check`
      (69 items, zero diff) still pass, `grep` for
      `ComponentPartPage|focused_part|FocusedPart|part_slug|resolve_part|nav_parts_drift`
      across `apps/playground/src/` returns nothing, and a fresh `dx serve`
      shows `/select` as a flat, non-expandable nav entry with no `›` glyph
      and the full unscoped panel, matching pre-change behavior.
