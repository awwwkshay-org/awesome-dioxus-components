Rule references (R0-R8) are defined in `design.md`'s Decisions D1-D3 and this
file's own Wave 0 rule-set task. Every component task records its category
inline. "✓" after a component name means it is expected to need no change —
verify and record that explicitly rather than skipping it.

Each wave's close-out is exactly one CLI rebuild plus three `adico add
<items…> --replace` calls (`apps/playground`, `examples/basic-spa`,
`examples/basic-ssr`) — never a cycle per component.

## 0. Harness, tooling, and rule-set foundation (no registry source edits)

- [x] 0.1 Write the rule-set doc (R0-R8: global prohibitions incl. never emit
      a responsive `rounded-*`; Form C clamp vs. Form B breakpoint-restore and
      when each applies; fixed-overlay gutter clamps; positioner-anchored
      definite-width clamps; hard-pinned-dimension escapes; horizontal-flex
      wrap/scroll; `svh` over `vh`; grid collapse; composition-over-base-class
      needing `!`) as a committed reference (e.g.
      `docs/adico/mobile-first-rules.md`) and verify it's linked from
      `design.md`. Done: `docs/adico/mobile-first-rules.md` created, linked
      from `design.md`'s Decisions section intro.
- [x] 0.2 Add a `registry checksums --write` subcommand to `adico-xtask` that
      computes each `registry.json` item's file SHA-256 and writes it in
      place, and verify it round-trips: run it once with no source changes and
      confirm `git diff registry/registry.json` is empty. Done:
      `packages/adico-xtask/src/registry_checksums.rs`, wired into
      `main.rs`. Verified: (1) no-op run on clean tree produces empty diff,
      (2) editing `kbd.rs` and running produces a single-checksum diff for
      exactly that item with the rest of the 1500+ line file byte-identical,
      (3) reverting and re-running restores the original checksum exactly.
      `cargo build -p adico-xtask --locked` passes.
- [x] 0.3 Add `apps/playground/src/pages/responsive.rs` with realistic
      fixtures (5+ real tab labels, a File/Edit/View/Window/Help menubar, a
      full data-table toolbar + pagination footer, a card with a
      `CardAction`, a multi-crumb breadcrumb, etc. — one fixture per component
      this change touches) and verify `cargo check -p adico-playground`
      passes. Done: split into `pages/responsive_flow.rs` (in-flow cases:
      tabs ×5, menubar File/Edit/View/Window/Help, breadcrumb ×4, card with
      CardAction, button-group ×5, data-table with 7 rows to force real
      pagination) and `pages/responsive_overlay.rs` (query-param-selected
      overlay cases: dialog, alert-dialog, sheet, drawer, toast, popover,
      hover-card, command — extensible per wave). `cargo check -p
      adico-playground` passes with only pre-existing unrelated warnings.
      More overlay/flow cases will be added as later waves need them, per
      each file's own module doc.
- [x] 0.4 Add `/responsive/flow` and `/responsive/overlay?case=<item>` routes
      in `apps/playground/src/routes.rs`, outside `#[layout(Layout)]`, and
      verify both render in a browser with `dx serve` (no sidebar/shell
      chrome present). Done: routes added after `#[end_layout]` at the end
      of the `Route` enum (not in `nav_items()` — harness-only). Verified
      live via `dx serve` + Chrome: `/responsive/flow` renders all 6
      in-flow fixtures full-bleed with no shell chrome, including a working
      7-row/2-page DataTable; `/responsive/overlay?case=dialog` renders the
      Dialog pre-opened (`open: true`, no click needed) at 512px
      (`max-w-lg`), confirming both the route wiring and the pre-fix
      baseline this change's Wave 1 will move.
- [x] 0.5 Restructure `tests/playwright/playwright.config.ts` into `desktop`
      (1280×800, ignores `responsive*.spec.ts`), `mobile` (375×812, matches
      `responsive*.spec.ts`), and `desktop-invariance` (1280×800, matches
      `responsive-desktop.spec.ts`) projects, and verify the existing 19 specs
      still collect and run only under `desktop`. Done: added
      `test:responsive`/`test:responsive-desktop` npm scripts. Verified via
      `npx playwright test --list`: all 66 pre-existing tests across all 19
      files collect exclusively under `[desktop]`, zero under `mobile`/
      `desktop-invariance` (those specs don't exist yet — next tasks).
- [x] 0.6 Write `tests/playwright/responsive.spec.ts`: for each
      `/responsive/...` fixture, walk visible descendants and assert
      `getBoundingClientRect()` stays within `[0, innerWidth]` horizontally
      after transitions settle, plus the targeted per-component assertions
      for dialog, alert_dialog, command, calendar, date_picker, time_picker,
      tabs, data_table, card, and toast (see `design.md` D7). Verify it FAILS
      against today's unmodified components on the known-broken list (dialog,
      alert_dialog, calendar, toast, tabs) — a check that has never been seen
      to fail is not trusted for the other ~54 components in Wave 6. Done:
      ran live against `dx serve` on port 8080 — **8 tests fail exactly as
      predicted**: toast's left edge measures -16px (confirming the exact bug
      cited in the proposal), dialog/alert-dialog have a 0px gutter, tabs list
      is not scrollable, data-table and the generic in-flow walk overflow,
      card renders 2 columns instead of 1 at 375px. The other 9 tests
      (sheet, drawer, popover, hover-card, command, time-picker overlay
      cases) already pass, matching the "already-correct" verify-only list.
- [x] 0.7 Write `tests/playwright/responsive-desktop.spec.ts`: capture and
      assert today's actual computed geometry (dialog `max-width`, calendar
      `width`/`height`, popover `width`, hover-card `width`, alert-dialog
      `max-width` for both sizes, carousel vertical `height`, sidebar open
      `width`, toast `max-width`, card's `grid-template-columns` track count,
      tabs list `overflow-x`) BEFORE any registry edit in this change, and
      verify it PASSES against today's unmodified components — this is the
      baseline the rest of the change is not allowed to move. Done: all 10
      values captured live (dialog/alert-dialog max-width 512px, popover
      288px, hover-card 256px, toast max-width 420px, calendar 288×320,
      carousel vertical height 384px, card grid 2 tracks, tabs overflow-x
      "visible"), 10/10 passing. **Incidental discovery, reported to and
      confirmed by the user**: `sidebar.rs`'s `w-[--sidebar-width]` compiles
      under Tailwind v4.1.5 to the invalid declaration
      `width: --sidebar-width` (missing `var(...)`), silently dropped by the
      browser — Sidebar's open width has never been the intended 256px on
      this toolchain, unrelated to viewport. User chose "fix now, fold into
      this change" — added as an explicit fix to task 5.5 below. The
      baseline test asserts the current broken shrink-to-fit width (`< 200`)
      with a comment that Wave 5's commit flips it to `toBe(256)`.
- [x] 0.8 Run `openspec validate make-registry-components-mobile-first
      --strict` and confirm it passes before starting Wave 1. Done: "Change
      'make-registry-components-mobile-first' is valid".

## 1. Fixed-position overlays (R1) — 5 components

- [x] 1.1 `dialog.rs`: add `max-w-[calc(100%-2rem)]` to `DialogContent`'s base
      class, move `max-w-lg` behind `sm:`; verify via
      `responsive-desktop.spec.ts` (unchanged ≥640px) and `responsive.spec.ts`
      (gutter present at 375px). Done: `sm:max-w-lg` restores 512px exactly.
- [x] 1.2 `alert_dialog.rs`: same clamp on the base class; move both size-enum
      values (`Default => max-w-lg`, `Sm => max-w-sm`) behind `sm:`; verify the
      same way for both sizes. Done: `sm:max-w-lg`/`sm:max-w-sm`.
- [x] 1.3 `toast.rs`: fix the confirmed left-edge overflow bug — replace base
      `w-full` with `w-[calc(100%-2rem)]` and add `sm:w-full` (the existing
      `sm:max-w-[420px]` still caps desktop unchanged); verify
      `rect.left >= -1` at 375px in `responsive.spec.ts` and unchanged
      `max-width` at desktop. Also fold in R5's `max-h-screen` → `max-h-[100svh]`
      fix in the same edit, since this is Wave 1's only touch of this file
      and no other wave revisits it. Done: both fixes applied.
- [x] 1.4 `sheet.rs`✓: verify its existing `w-3/4 sm:max-w-sm` pattern needs no
      change; record the verdict. Verified — no change needed.
- [x] 1.5 `drawer.rs`✓: verify layout needs no change beyond the R5 `vh`→`svh`
      fix (`max-h-[80vh]` → `max-h-[80svh]`); record the verdict. Done: its
      `w-3/4 sm:max-w-sm` layout already correct; applied the `max-h-[80vh]`
      → `max-h-[80svh]` fix (inert at desktop, matches mobile-chrome-aware
      behavior). Reinstalled into all 3 apps.
- [x] 1.6 Wave close-out: `rustfmt --edition 2024` the 3 edited files, `xtask
      registry checksums --write`, `xtask registry build && registry
      validate`, `cargo build -p adico-cli --locked`, `adico add dialog
      alert_dialog toast sheet drawer --replace` in all 3 consumer apps,
      full `npm test` (desktop + mobile + desktop-invariance) green. Done:
      checksums updated for exactly the 3 edited files (verified 3-line
      `registry.json` diff), `registry build`/`validate`/`styling-usage
      check` all pass, CLI rebuilt, `adico add dialog alert-dialog toast
      --replace` (sheet/drawer omitted — unmodified) into all 3 apps,
      confirmed via `git status --short` + direct grep that exactly the 3
      expected files changed in each app with the right content. Mobile
      spec: **13 passed / 4 failed (up from 9/8)** — the 4 remaining
      failures are exactly the Wave 4/5 targets (tabs, data-table, card) not
      in this wave's scope. Desktop-invariance: **10/10 unchanged**,
      confirming dialog/alert-dialog/toast's desktop geometry is byte-identical
      to the pre-Wave-1 baseline. `cargo fmt --all --check` / `cargo check
      --locked --workspace` / `cargo clippy ... -D warnings` all clean.

## 2. Positioner-anchored surfaces (R2) — 8 components

- [x] 2.1 `popover.rs`: apply Form C — `w-72 max-w-[calc(100%-2rem)]`; verify
      unchanged 288px width at desktop, clamped at 375px. Done, verified
      10/10 desktop-invariance (288px unchanged).
- [x] 2.2 `hover_card.rs`: same, `w-64 max-w-[calc(100%-2rem)]`. Done,
      verified 256px unchanged at desktop.
- [x] 2.3 `tooltip.rs`: audit for a fixed width/`min-w`; apply Form C if
      present, record `n/a` if not. Verified `n/a` — no width/min-w class at
      all; content is shrink-to-fit short text, no overflow risk.
- [x] 2.4 `dropdown_menu.rs`: audit content `min-w-[…]`, pair with a
      `max-w-[calc(100%-2rem)]` clamp per R2. Done — both `DropdownMenuContent`
      (`min-w-40`) and `DropdownMenuSubContent` (`min-w-32`) fixed.
- [x] 2.5 `context_menu.rs`: same as 2.4. Done — `min-w-[8rem]` had no max-w
      at all (worse than dropdown_menu); paired with the clamp.
- [x] 2.6 `select.rs`: audit trigger/content widths; apply R2's clamp to
      content if it carries a fixed/min width. Done — content's `min-w-32`
      clamped. Trigger's `w-full min-w-32` left alone (bounded by parent via
      `w-full`, not a `fixed`/anchored overflow risk).
- [x] 2.7 `combobox.rs`: same as 2.6. Done — content's `min-w-48` clamped.
- [x] 2.8 `native_select.rs`: audit; likely `n/a` (native `<select>` sizing is
      browser-controlled) — record the verdict either way. Verified `n/a` —
      real `<select>`, no CSS-controlled popup exists to clamp.
- [x] 2.9 `navigation_menu.rs` / `menubar.rs` content (`min-w-[12rem]`): pair
      with a `max-w-[calc(100%-2rem)]` clamp per R2 (root-level horizontal
      overflow for these two is handled separately in Wave 4). Done for both
      files' content classes; root-level R4 fix still pending for Wave 4.
- [x] 2.10 Wave close-out: same cycle as 1.6 for all Wave 2 items, plus a live
      Chrome check at 375px per D3's risk note — confirm an anchored popover
      near a viewport edge repositions and does not get stuck
      `visibility: hidden`. Done: 8 checksums updated (exactly the 8 edited
      files), registry build/validate/styling-usage check pass, CLI rebuilt,
      reinstalled into all 3 apps (confirmed via git status). **Discovered a
      real Playwright/dx-serve pitfall during verification**: running the
      harness immediately after a reinstall against a long-lived `dx serve`
      hot-reload session produced 8-9 spurious failures on already-fixed
      Wave-1 components (dialog/alert-dialog/toast) — the live-patched tab
      showed the correct state but a *fresh* page load (Playwright's own)
      served a stale pre-rebuild bundle. Killing and restarting `dx serve`
      fresh resolved it immediately (mobile 23/27 passing — only the
      expected Wave 4/5 failures remain; desktop-invariance 10/10). **This
      is now the required practice for every wave's verification**, not
      just Chrome spot-checks: always restart `dx serve` before running the
      harness post-reinstall, not just before manual checks. Live Chrome
      check: popover's trigger sits at the extreme left edge of the
      viewport; the popover renders fully visible, correctly positioned, no
      stuck-hidden state.

## 3. Composers over Wave 1/2 base classes (R8) — 8 components

- [x] 3.1 `command.rs`: relax `max-h-[300px]` to `max-h-[min(300px,60svh)]`;
      confirm `CommandDialog` inherits Wave 1's `dialog.rs` fix with no
      further change needed. Done; confirmed via code (composes
      `DialogContent` directly, no width override).
- [x] 3.2 `date_picker.rs`: verify its `w-auto` override of Popover's width
      (already `!`-free since it fully replaces rather than fights the base
      class) composes correctly with Wave 2's popover fix; apply R3 to
      whatever width the calendar inside it now presents. Verified live in
      Chrome + via `getComputedStyle`: `width: 320px` (auto, from
      CalendarView's content) and `max-width: calc(100% - 32px)` both apply
      simultaneously with no conflict (different CSS properties) — no source
      edit needed. `date_picker.rs` itself unmodified.
- [x] 3.3 `date_time_picker.rs`✓: verify existing `sm:flex-row` pattern needs
      no change; record the verdict. Verified — no change needed.
- [x] 3.4 `time_picker.rs`: apply R3 to the `size-56` analog dial (fluid
      escape, e.g. `size-56 max-w-[calc(100%-2rem)]` inside its own
      container) and verify it still renders as a circle (`aspect-square` or
      equivalent) at every size. Done: `size-56` → `aspect-square w-full
      max-w-56` (identical 224px circle wherever parent ≥224px, true today;
      shrinks as a circle, not an oval, if ever narrower). Verified no
      hardcoded pixel assumptions elsewhere in the file (size is measured
      dynamically). Live-checked in Chrome: renders as a perfect circle with
      evenly-spaced hour labels.
- [x] 3.5 `color_picker.rs`: audit the saturation square/hue bar/hex-field
      layout for fixed widths per R3; apply the fluid escape. Done:
      `ColorPickerContent`'s `w-64` (composes over `PopoverContent`, an R8
      site) gets the same Form C clamp as Wave 2's `popover.rs`/`hover_card.rs`
      — `w-64 max-w-[calc(100%-2rem)]`. Updated the mirroring unit test
      literal for consistency. Saturation square/hue bar/hex fields
      themselves already fluid (`w-full`/`flex-1`), no other edit needed.
- [x] 3.6 `theme_builder.rs`: audit beyond its existing `sm:grid-cols-2`;
      apply R6 if any additional grid needs collapsing. Audited the two
      other `grid-cols-3` instances (palette swatch picker, active-role
      preview) — verdict `n/a`: these are compact chip/swatch grids (short
      `text-[10px]` labels, ~100px per cell), not content-dense two-column
      layouts; 3 columns fits comfortably at any realistic width and
      collapsing to 1 column would look worse, not better. R6 doesn't apply
      to this UI pattern.
- [x] 3.7 `theme_switcher.rs`: audit; record verdict. Its `SelectList {
      class: "w-56", ... }` composes over `select.rs`'s `SelectList`, whose
      base already gained Wave 2's `max-w-[calc(100%-2rem)]` clamp — the
      `w-56` override (a different property, `width`, vs. the base's
      `min-width`/`max-width`) is automatically protected. `n/a`, no edit.
- [x] 3.8 `mode_toggle.rs`: audit; record verdict. Composes
      `DropdownMenuContent` directly with no width override — inherits Wave
      2's fix automatically. `n/a`, no edit.
- [x] 3.9 Wave close-out: same cycle as 1.6 for all Wave 3 items. Done: 3
      checksums updated (command/time_picker/color_picker — the only 3 files
      actually edited), registry build/validate/styling-usage check pass,
      CLI rebuilt, `adico add command time-picker color-picker --replace`
      into all 3 apps (confirmed via git status). Restarted `dx serve`
      fully clean (killed two stale/conflicting processes found on port
      8080 from earlier waves' restarts, freed the port, started one single
      instance) before verification per the Wave 2 lesson. Mobile: 23/27
      passing, same 4 expected Wave 4/5 failures, unchanged. Desktop
      -invariance: 10/10. Targeted command/time-picker overlay cases: 2/2
      passing. `cargo fmt --all --check`/`check --workspace`/`clippy -D
      warnings` all clean.

## 4. Horizontal-flex containers (R4) — 12 components

- [ ] 4.1 `tabs.rs`: add `overflow-x-auto` to `TabsList`, drop `flex-1` from
      `TabsTrigger` below `sm`; verify via the `/responsive/flow` 5-tab
      fixture that the list scrolls rather than clips, and unchanged desktop
      layout for a 2-tab case.
- [ ] 4.2 `menubar.rs`: add `overflow-x-auto` to the root; verify with the
      File/Edit/View/Window/Help fixture.
- [ ] 4.3 `navigation_menu.rs`: same treatment on `NavigationMenuList`.
- [ ] 4.4 `toolbar.rs`: decide and apply whether the (currently unstyled) root
      gains a default wrap/scroll class, given it's a bare primitive
      re-export; record the decision either way.
- [ ] 4.5 `button_group.rs`: switch to the existing vertical variant (already
      at line ~27) at a breakpoint rather than writing new CSS; verify a
      4-5-button group below `sm`.
- [ ] 4.6 `toggle_group.rs`: audit; apply R4 if it shares `button_group`'s
      no-wrap risk.
- [ ] 4.7 `pagination.rs`: audit; apply `flex-wrap` if needed for many page
      links.
- [ ] 4.8 `breadcrumb.rs`✓: verify existing `flex-wrap break-words sm:gap-2.5`
      needs no change; record the verdict.
- [ ] 4.9 `input_group.rs`: audit for horizontal-flex overflow risk.
- [ ] 4.10 `input_otp.rs`: audit; a fixed-width digit row is the likely risk —
      apply R3/R4 as appropriate.
- [ ] 4.11 `tag_group.rs`: audit for wrap behavior with many tags.
- [ ] 4.12 `data_table.rs`: add `flex-wrap` to the toolbar row (line ~255) and
      pagination footer (line ~376); verify with the full-toolbar fixture at
      375px.
- [ ] 4.13 Wave close-out: same cycle as 1.6 for all Wave 4 items.

## 5. Fixed dimensions and grids (R3/R5/R6) — 12 components

- [ ] 5.1 `calendar.rs`: apply the flex-child rule from D2 —
      `w-full sm:w-[18rem]` (not `max-w`, since it's a flex child in
      `date_time_picker.rs`) on `CalendarView`'s `h-[20rem] w-[18rem]`; drop
      `grow-0` from the `w-[60%]`/`w-[40%]` month/year selects so they can
      absorb slack; verify both call sites (`date_picker.rs`,
      `date_time_picker.rs`) render correctly at 375px and unchanged at
      desktop.
- [ ] 5.2 `carousel.rs`: apply R3 to the vertical orientation's `h-[24rem]`.
- [ ] 5.3 `card.rs`: apply R6 — base `grid-cols-1`, move
      `has-[[data-slot=card-action]]:grid-cols-[1fr_auto]` behind `sm:`;
      verify Tailwind v4 actually emits the `sm:has-[…]` compound by grepping
      the built `assets/tailwind.css` after `dx serve`; if it doesn't emit,
      fall back to a `@container` query instead (recorded as a design
      deviation if taken).
- [ ] 5.4 `table.rs`✓: verify existing `overflow-x-auto` wrapper needs no
      change; record the verdict.
- [ ] 5.5 `sidebar.rs`: add `max-w-[85vw]` to the open panel's width class per
      `design.md` D4; verify it never exceeds the viewport at 375px and is
      inert (no visible change) at desktop. **Also fix the incidentally
      discovered pre-existing defect** (confirmed with the user, out of this
      change's original scope but folded in here since it's the same file):
      `w-[--sidebar-width]` / `w-[--sidebar-width-icon]` (lines ~187-189)
      compile under Tailwind v4.1.5 to the invalid `width: --sidebar-width`
      declaration (missing `var(...)`), silently dropped by the browser, so
      Sidebar has never rendered its intended 16rem open width. Fix: change
      the bracket form to Tailwind v4's parens var-reference syntax —
      `w-(--sidebar-width)` / `w-(--sidebar-width-icon)`. Verify: rebuild
      `apps/playground/assets/tailwind.css` and confirm the compiled rule
      reads `width: var(--sidebar-width)`; flip
      `responsive-desktop.spec.ts`'s sidebar assertion from
      `toBeLessThan(200)` to `toBe(256)` and confirm it now passes for the
      correct reason (not just because nothing regressed).
- [ ] 5.6 `resizable.rs`: audit; record whether the 4px handle hit area or
      panel min-sizes need any narrow-viewport treatment (likely audit-only,
      confirm).
- [ ] 5.7 `virtual_list.rs`: apply R5 if it has a fixed `vh`/`px` height
      assumption.
- [ ] 5.8 `message_scroller.rs`: same as 5.7.
- [ ] 5.9 `scroll_area.rs`: audit for fixed dimensions.
- [ ] 5.10 `drag_and_drop_list.rs`: audit for fixed dimensions.
- [ ] 5.11 `bubble.rs`: audit its `max-w-[80%]` — likely already fine, record
      the verdict.
- [ ] 5.12 `attachment.rs`: audit for fixed dimensions.
- [ ] 5.13 Wave close-out: same cycle as 1.6 for all Wave 5 items.

## 6. Audit-only remainder — 24 components

For each, apply rules R0-R6 if a real fixed-dimension/overflow issue is found;
otherwise record `already-correct`/`n/a` explicitly. Promote any component
that needs a real fix into the wave matching its category and re-review it
there rather than folding a nontrivial change into this wave's close-out.

- [ ] 6.1 `accordion.rs` — audit and record verdict.
- [ ] 6.2 `alert.rs` — audit and record verdict (known candidate for a grid
      fix per `design.md`'s Wave 6 risk note; check before assuming `n/a`).
- [ ] 6.3 `aspect_ratio.rs` — audit and record verdict.
- [ ] 6.4 `avatar.rs` — audit and record verdict.
- [ ] 6.5 `badge.rs` — audit and record verdict.
- [ ] 6.6 `button.rs` — audit and record verdict.
- [ ] 6.7 `checkbox.rs` — audit and record verdict.
- [ ] 6.8 `collapsible.rs` — audit and record verdict.
- [ ] 6.9 `copy_button.rs` — audit and record verdict.
- [ ] 6.10 `empty.rs` — audit its existing `md:p-12`; record verdict.
- [ ] 6.11 `input.rs` — audit and record verdict.
- [ ] 6.12 `item.rs` — audit and record verdict.
- [ ] 6.13 `kbd.rs` — audit and record verdict.
- [ ] 6.14 `label.rs` — audit and record verdict.
- [ ] 6.15 `marker.rs` — audit and record verdict.
- [ ] 6.16 `message.rs` — audit and record verdict.
- [ ] 6.17 `progress.rs` — audit and record verdict.
- [ ] 6.18 `radio_group.rs` — audit and record verdict.
- [ ] 6.19 `skeleton.rs` — audit and record verdict.
- [ ] 6.20 `slider.rs` — audit and record verdict.
- [ ] 6.21 `spinner.rs` — audit and record verdict.
- [ ] 6.22 `switch.rs` — audit and record verdict.
- [ ] 6.23 `textarea.rs` — audit and record verdict.
- [ ] 6.24 `toggle.rs` — audit and record verdict.
- [ ] 6.25 Wave close-out: same cycle as 1.6 for any Wave 6 items that
      received a real fix (if none did, skip the reinstall — nothing changed).

## 7. Full-sweep validation and archive

- [ ] 7.1 Run the complete validation sequence from `design.md`/the approved
      plan across the whole workspace: `cargo fmt --all --check`, `cargo check
      --locked --workspace`, `cargo clippy --locked -p adico-cli -p
      adico-primitives -p adico-registry-core -p adico-test-utils -p
      adico-xtask --all-targets -- -D warnings`, `cargo test --locked` for the
      same packages, `cargo check --target wasm32-unknown-unknown -p
      adico-primitives`.
- [ ] 7.2 Run `cd tests/playwright && npm test` (all three projects) and
      confirm zero regressions in the pre-existing 19 desktop specs.
- [ ] 7.3 Update `statics/styling_usage/*.json`'s `inspirationNote` for every
      item touched in Waves 1-6, and verify `xtask styling-usage check` still
      passes.
- [ ] 7.4 Update `docs/validation.md`'s surface matrix with a responsive-layout
      row, and update `docs/adico/m4-parity-audit.md` /
      `docs/adico/m4-acceptance.md`'s "unmeasurable" responsive verdicts to
      reflect the new harness's actual results.
- [ ] 7.5 Run `openspec validate make-registry-components-mobile-first
      --strict` and confirm it passes.
- [ ] 7.6 Live spot-check in Chrome at 375px (via `dx serve` on
      `apps/playground`'s `/responsive/...` routes) for the full known-broken
      list from the proposal, and confirm visually that desktop
      (`apps/playground` at 1280px) is unchanged.
- [ ] 7.7 Archive the change once all tasks above are checked.
