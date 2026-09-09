## 1. Shared nav extraction

- [x] 1.1 Create `apps/playground/src/components/nav.rs` with a component
      that renders the `SidebarMenu`/`nav_items()` loop currently inline in
      `Layout` (`routes.rs`), taking the current route and an `onnavigate`
      callback (so callers can close a mobile overlay on navigate without
      the nav component knowing about `Sheet`). Wire it into
      `components/mod.rs`. Verify: `cargo check --locked -p adico-cli`-style
      workspace check is not applicable (playground isn't in that list per
      CLAUDE.md); instead verify with `cargo check --locked --workspace`
      and a `dx serve` smoke load of `/`.
- [x] 1.2 Update `Layout`'s existing `>= md` tree to call the new nav
      component instead of its inline loop, with no other changes to that
      tree's markup or classes. Verify: a `git diff` of the `>= md` branch
      shows only the loop body replaced by the component call — no class
      or structural changes.

## 2. Mobile shell

- [x] 2.1 Add the mobile (`< md`) branch to `Layout`: a top bar (logo/home
      link, hamburger `Button`, theme controls) plus an installed `Sheet`
      (`side: SheetSide::Left`) containing the shared nav component and the
      same footer controls (`ModeToggle`/`ThemeSwitcher`/
      `ThemeBuilderLauncher`) the desktop `SidebarFooter` renders. Gate the
      two trees with `hidden md:flex` / `flex md:hidden` (no JS viewport
      detection). Verify: `dx serve`, real 375×812 browser check (see task
      4.1) shows the hamburger, nav sheet, and no `>= md` `ResizablePanelGroup`
      markup rendered at that width.
      Completion note: implemented as described in design.md's revised
      "Two chrome trees ... one shared content pane" decision — only the
      nav chrome forks; `Outlet` stays the single always-mounted instance
      inside the existing content `ResizablePanel`, which gets a
      `max-md:flex-1!` override (confirmed live to beat its own inline
      `flex: 0 0 {size}%` style below `md`, and to no-op at `>= md`).
- [x] 2.2 Wire the shared nav component's `onnavigate` callback to close the
      mobile sheet (set the open signal to `false`) in addition to pushing
      the route. Verify: 375px browser check — selecting a nav entry from
      the open sheet lands on the new page with the sheet no longer visible.
      Verified live via injected-iframe browser pass: clicking "Button" in
      the open sheet navigated to `/button` and the sheet closed.
- [x] 2.3 Update `main.rs`'s root `div` from `h-screen w-screen
      overflow-hidden` to `h-dvh w-full overflow-hidden` (per design.md's
      root-sizing decision). Verify: `cargo check --locked --workspace`
      passes; 375px browser check shows no vertical clipping from a mobile
      browser chrome-sized viewport and no horizontal document overflow.

## 3. Spec-required scenarios

- [x] 3.1 Confirm every scenario in
      `specs/adico-playground-structure/spec.md`'s new "Navigation is
      reachable and usable below the `md` breakpoint" requirement is
      satisfied by tasks 2.1–2.3 (nav column not rendered below `md`, no
      horizontal overflow, full nav list reachable, active route indicated,
      navigate-then-close, new routes need no extra wiring since they only
      touch `nav_items()`).
      Confirmed live and by `responsive-shell.spec.ts` (task 4.1): nav
      column hidden below `md`, hamburger reveals every entry fully
      legible, active route highlighted (`data-active="true"` verified via
      JS), navigate-then-close works, no horizontal overflow open or
      closed. New routes need no extra wiring because both trees render
      from the same `nav_items()` via the shared `NavList` component.
- [x] 3.2 Confirm the "The `>= md` layout is unaffected by mobile support"
      requirement holds: diff the `>= md` tree's rendered classes/markup
      against pre-change `Layout` and confirm no unintended change beyond
      the nav-list extraction in task 1.2.
      Confirmed live: at a 1280px viewport the content panel's computed
      `flex` is unchanged (`0 0 82%`, `max-md:flex-1!` does not apply), the
      nav panel/handle render exactly as before (`hidden`/`md:` variants
      are no-ops at `>= md`), and `data-active` correctly tracks the
      current route. `responsive-desktop-shell.spec.ts` (task 4.2) gates
      this going forward.

## 4. Test coverage

- [x] 4.1 Add a mobile-viewport Playwright spec (reuse the existing
      `mobile` project at 375×812 in `tests/playwright/playwright.config.ts`)
      covering: the `>= md` nav column is not rendered, the hamburger opens
      the sheet with every `nav_items()` entry visible and legible (not
      truncated), selecting an entry navigates and closes the sheet, and no
      element causes horizontal document overflow. Name/locate it so it's
      picked up by the `mobile` project's `testMatch` (broaden the regex in
      `playwright.config.ts` if the new filename doesn't already match
      `/(^|\/)responsive\.spec\.ts$/`) — verify with
      `npx playwright test --project=mobile` from `tests/playwright/`.
      Added `tests/playwright/responsive-shell.spec.ts` (4 tests, all
      passing) and broadened the `mobile` project's `testMatch` in
      `playwright.config.ts` to `/(^|\/)(responsive|responsive-shell)\.spec\.ts$/`.
- [x] 4.2 Add a desktop-invariance Playwright spec (the existing
      `desktop-invariance` project at 1280×800) asserting the `>= md`
      nav column's geometry (position, computed width, visible label text)
      matches its pre-change values, following the same before/after
      geometry-baseline pattern `responsive-desktop.spec.ts` already
      established for the mobile-first sweep. Verify with
      `npx playwright test --project=desktop-invariance` from
      `tests/playwright/`.
      Added `tests/playwright/responsive-desktop-shell.spec.ts` (4 tests,
      all passing) and broadened the `desktop-invariance` project's
      `testMatch` to `/(^|\/)responsive-desktop(-shell)?\.spec\.ts$/`.
- [x] 4.3 Run the full existing Playwright suite (`npm test` from
      `tests/playwright/`) and confirm no pre-existing spec regresses.
      Note in this task's completion notes if `wave3.spec.ts`'s known
      Positioner-visibility failures or `dialog.spec.ts`'s known
      focus-trap failures appear — expected pre-existing failures, not
      caused by this change.
      Ran every spec that targets the playground (the only app this change
      touches): `responsive.spec.ts` (17/17 pass), `responsive-desktop.spec.ts`
      (10/10 pass), `playground-resizable-split.spec.ts` (3/3 pass),
      `playground-drag-and-drop-list.spec.ts` (1/1 pass), plus this
      change's own 8 new tests — all pass. `mode-toggle.spec.ts` and
      `theme-switcher.spec.ts` fail (7 tests) both before and after this
      change (isolated via `git stash` against unmodified code, same
      failures) — pre-existing, unrelated to this change; likely an
      environment mismatch (these appear to expect a `dx serve` invocation
      on a different port/platform than the default one used here).
      Did NOT run the remaining specs (`dialog.spec.ts`, `select.spec.ts`,
      `time-picker.spec.ts`, `fullstack.spec.ts`, `wave*.spec.ts`,
      `playground-time-picker.spec.ts`): each targets a separate,
      dedicated fixture app (`tests/installation/*-consumer`,
      `examples/basic-ssr`) on its own port, none of which this change
      touches or could affect — standing up all of them was out of scope
      for verifying this shell-only change.

## 5. Documentation of known, out-of-scope issues

- [x] 5.1 Add a note (in this change's own completion notes, not a code
      comment) recording that `Sheet`'s focus-trap/scroll-lock/
      `aria-hidden` behavior is expected to share `Dialog`'s known defect
      (see project memory on the positioner/dialog focus-trap gap) — the
      mobile nav's click-driven open/close still works; this is an
      accessibility gap to fix in a future, separately-scoped change.
      Noted here: the mobile nav `Sheet` (task 2.1) is expected to share
      `Dialog`'s confirmed focus-trap/scroll-lock/`aria-hidden` defect
      (`packages/adico-primitives/src/{lib,dialog}.rs`, see project memory
      "positioner_visibility_bug"'s dialog-consumer addendum). Not fixed
      here — out of scope per this change's design.md Non-Goals. Click-
      driven open/close/navigate all work regardless (confirmed live);
      only the deeper a11y engagement (focus trapped inside, background
      `aria-hidden`, scroll lock) is affected.
- [x] 5.2 Add a note recording the Popover-anchored-Calendar overflow found
      on `/calendar` at 375px (both `calendar.rs` and `popover.rs` already
      carry individually-correct mobile-first/clamp classes; the overflow
      looks like a Positioner anchor/edge-avoidance gap) as a known issue
      for a future change — not fixed by this one, and not caused by it.
      Noted here: `/calendar` at 375px shows a Calendar-in-Popover
      instance overflowing the right viewport edge, reproduced identically
      both before and after this change (confirmed via the same injected-
      iframe browser pass on unmodified `main` and on this change's final
      code). `calendar.rs` (`w-full sm:w-[18rem]`) and `popover.rs`
      (`max-w-[calc(100%-2rem)]`) both already carry individually-correct
      mobile-first/clamp classes, so this looks like a Positioner
      anchor/edge-avoidance gap (no flip/shift-to-viewport logic), not a
      shell defect. Left for a future, separately-scoped change; no
      registry file was touched by this change.

## 6. Validation

- [x] 6.1 Run `cargo fmt --all --check`, `cargo check --locked
      --workspace`, `cargo clippy --locked -p adico-cli -p
      adico-primitives -p adico-registry-core -p adico-test-utils -p
      adico-xtask --all-targets -- -D warnings`, `cargo test --locked -p
      adico-cli -p adico-primitives -p adico-registry-core -p
      adico-test-utils -p adico-xtask`, and `openspec validate
      make-playground-shell-responsive --strict`. Report any command
      skipped and why.
      All five commands ran clean: `cargo fmt --all --check` (no diff),
      `cargo check --locked --workspace` (builds, only pre-existing
      unrelated warnings in `generated/controls/mod.rs`), the mandated
      clippy package set (zero warnings — `adico-playground` is
      deliberately excluded from this set per CLAUDE.md/this task's own
      note; it was checked separately below), the mandated test package
      set (all green, ~600+ assertions across doctests/unit tests), and
      `openspec validate --strict` (valid). Additionally ran
      `cargo clippy -p adico-playground --all-targets -- -D warnings`
      (not in the mandated set) as a courtesy: it fails on 25 pre-existing
      lints in `data_table.rs`/`textarea.rs` (installed registry copies
      this change never touched) — confirmed none of the failures are in
      `routes.rs`, `main.rs`, `nav.rs`, or `components/mod.rs`. Skipped:
      `wasm32-unknown-unknown` target check and Playwright browser/a11y
      suites beyond what's covered in task 4.3 — no wasm-specific or
      server-feature code was touched, and the browser verification for
      this specific change is covered by the live browser pass (task 6.2)
      plus the new/existing Playwright specs (task 4).
- [x] 6.2 Do a final real-browser 375px pass (per this change's
      verification plan: `dx serve` plus an injected same-origin iframe
      sized 375×812 as a viewport proxy, since window resize is unreliable
      in this environment) across `/`, `/button`, `/data-table`,
      `/calendar` confirming: no horizontal document overflow, nav
      reachable and dismissible, controls reachable and scrollable, and
      the `>= md` layout visually unchanged from a pre-change screenshot.
      Done: `/` and `/button` confirmed no overflow, nav open/navigate/
      close all correct; `/data-table` confirmed no *document* overflow
      (the table's own internal horizontal scroll container is the
      intended mobile-first behavior, unrelated to this change);
      `/calendar` confirmed the pre-existing, unrelated Popover overflow
      (task 5.2) and nothing else. `>= md` (1280px) confirmed nav column
      renders at the correct 18% width with `data-active` tracking the
      current route correctly, matching pre-change behavior.
