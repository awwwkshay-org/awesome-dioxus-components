## 1. Preview/controls split (`demo.rs`)

- [x] 1.1 In `apps/playground/src/components/demo.rs`, replace the
      `div { class: "mt-3 grid min-h-0 flex-1 gap-3", style:
      "grid-template-rows: minmax(0, 3fr) minmax(0, 1fr);", ... }` wrapper
      with `ResizablePanelGroup { direction: ResizableDirection::Vertical,
      class: "mt-3 min-h-0 flex-1", ... }`, wrapping the existing preview
      canvas `div` (unchanged internals) in `ResizablePanel { index: 0usize,
      default_size: 70.0, min_size: 40.0, max_size: 85.0, ... }`, a
      `ResizableHandle { handle_index: 0usize, with_handle: true }` between
      them, and the existing `ui::Card` controls panel (unchanged internals)
      in `ResizablePanel { index: 1usize, default_size: 30.0, min_size:
      15.0, max_size: 60.0, ... }`. Import `ResizableDirection`/
      `ResizablePanelGroup`/`ResizablePanel`/`ResizableHandle` from
      `components::ui`. Verify `cargo check -p adico-playground` compiles.
      **Done**: `ui::` prefix already resolves these (no new `use` needed,
      `components/ui/mod.rs` already `pub use resizable::*`). Added
      `class: "flex min-h-0 flex-col"` to each `ResizablePanel` and
      `flex-1` to the inner canvas div/`Card` so they stretch to fill their
      panel's allotted height (the old grid row did this implicitly).
      `cargo check -p adico-playground` compiles clean.
- [x] 1.2 Live-verify via `dx serve` on a page with real controls (e.g.
      `/button`): the handle drags smoothly, dragging past either bound
      clamps at 40%/85% (preview) and 15%/60% (controls) rather than
      collapsing to zero or overflowing, the preview canvas's own pan-drag
      and the "Center" reset button still work unchanged, and no console
      panic occurs. **Done**: dragged mid-range (70/30 → 46.5/53.5, still
      summing to 100), then verified both extremes clamp exactly at the
      configured bounds (40%/60% one direction, 85%/15% the other) rather
      than collapsing/overflowing. Screenshots confirm the controls panel
      visibly grows/shrinks and previously-scroll-clipped fields (e.g. a
      "Text" input) become visible as it grows. No console errors. (Note:
      browser-automation clicks on the 1px-tall handle line were imprecise
      for actual drag gestures — verified via direct `PointerEvent`
      dispatch on the handle/overlay instead, which exercises the exact
      same `onpointerdown`/`onpointermove`/`onpointerup` code path a real
      drag does.)
- [x] 1.3 Live-verify via `dx serve` on a page whose `controls` is `None`
      (grep `apps/playground/src/pages/*.rs` for one, e.g. a page passing
      no `controls:` prop to `Demo`) that the empty-state paragraph still
      renders correctly inside the resized controls panel. **Done**:
      verified on `/breadcrumb` — "This component has no live props in the
      playground yet." renders correctly inside the controls `ResizablePanel`.
- [x] 1.4 Live-verify via `dx serve` on `/resizable` itself (the page whose
      own demo content already uses `ResizablePanelGroup`/`ResizablePanel`/
      `ResizableHandle`) that there is no double-nesting confusion or
      z-index/overlay collision between the page's own resizable demo and
      `Demo`'s new outer resizable split. **Done**: the page's own inner
      horizontal group ("One"/"Two") renders correctly inside `Demo`'s
      outer vertical split, both independently visible and functional, no
      visual or overlay collision.

## 2. Nav/content split (`routes.rs`'s `Layout`)

- [x] 2.1 In `apps/playground/src/routes.rs`'s `Layout`, remove
      `SidebarProvider`, `Sidebar`, `SidebarTrigger`, and `SidebarRail` from
      the composition. Wrap the whole `Layout` body in
      `ResizablePanelGroup { direction: ResizableDirection::Horizontal,
      class: "h-full w-full", ... }`. **Done**.
- [x] 2.2 Compose the nav column as `ResizablePanel { index: 0usize,
      default_size: 18.0, min_size: 12.0, max_size: 30.0, class: "flex
      h-full flex-col", ... }` containing, unchanged: the `SidebarHeader`
      with the logo/home `Link`, `SidebarContent` > `SidebarGroup` >
      `SidebarGroupContent` > `SidebarMenu` (the `nav_items()` loop,
      `navigator.push`/`current_route == route` active-state logic
      untouched), and `SidebarFooter` (`ModeToggle`/`ThemeSwitcher`/
      `ThemeBuilderLauncher`, untouched). **Done**.
- [x] 2.3 Add `ResizableHandle { handle_index: 0usize, with_handle: true }`
      between the two panels. **Done**.
- [x] 2.4 Compose the main column as `ResizablePanel { index: 1usize,
      default_size: 82.0, min_size: 70.0, max_size: 88.0, class: "flex
      h-full min-h-0 flex-col", ... }` containing today's `SidebarInset`'s
      two children as plain `div`s (unchanged classes/content): the header
      row MINUS the `components::ui::SidebarTrigger { "☰" }` button (its
      sole purpose no longer exists in this design), and the
      `Outlet::<Route> {}` content area. **Refined during implementation**:
      initially kept the header row as an empty bordered strip per the
      plan's literal wording, then live-verified via screenshot that an
      empty `div` with only padding/border reads as a purposeless visual
      gap above every page's own title, not an intentional design element
      — removed the row entirely (a minor, uncontroversial cleanup with no
      spec impact; the spec only requires the `SidebarTrigger` button be
      gone, not that its containing row remain). Only the
      `Outlet::<Route> {}` content div remains in the main column.
- [x] 2.5 Verify `cargo check -p adico-playground` compiles with no
      remaining references to `SidebarProvider`/`Sidebar`/`SidebarTrigger`/
      `SidebarRail` in `routes.rs`, and no panic-risk `use_sidebar()` call
      path remains reachable. **Done**: compiles clean (only the 23
      pre-existing unrelated unused-import warnings); grep for
      `SidebarProvider|SidebarTrigger|SidebarRail|components::ui::Sidebar `
      in `routes.rs` returns nothing.
- [x] 2.6 Live-verify via `dx serve`: the nav column renders identically in
      content (logo, every nav item in the same alphabetical order, correct
      active-state highlighting on the current page, footer's mode
      toggle/theme switcher/theme builder launcher all functional), dragging
      the handle resizes nav vs. content within the 12%/30% and 70%/88%
      bounds, clicking a nav item still navigates to the correct route, and
      no console panic occurs anywhere during normal use. **Done**:
      verified nav content unchanged (logo, alphabetical nav items,
      footer), dragged the handle (18/82 → 26.8/73.2, sums to 100),
      clicked "Bubble" in the nav and confirmed it navigated correctly with
      active-state highlighting moving to the new page.
- [x] 2.7 Live-verify the two splits' independent persistence behavior
      documented in the added spec requirement: resize the nav/content
      split, navigate to a different page via a nav click (no reload), and
      confirm the nav/content split's size is retained while the new page's
      own preview/controls split renders at its default sizes; then reload
      the browser and confirm both splits reset to their defaults. **Done**:
      exactly as specified — after resizing the nav column and clicking to
      `/bubble`, the nav column kept its resized width while Bubble's own
      preview/controls split rendered at its coded defaults; a full browser
      reload then reset both splits back to their original default sizes.

## 2a. Post-completion fix: nav label/logo truncation

- [x] 2a.1 **Regression found by the user after this change shipped**: at a
      high browser zoom level (or the nav column dragged near its 12%
      minimum), multi-word nav labels ("Alert Dialog", "Aspect Ratio",
      "Attachment", "Breadcrumb") and the "adico playground" logo wrapped
      across multiple lines and got visually clipped, instead of eliding
      cleanly. Root cause: the old fixed-`16rem` `Sidebar` (dropped by this
      change) always had enough width in CSS pixels regardless of zoom or
      window size (`rem` doesn't scale with viewport width), so this latent
      gap never surfaced; the new percentage-based `ResizablePanel` width
      can resolve to fewer CSS pixels than the unwrapped text needs, both
      at higher zoom (fewer CSS px per viewport) and near the resize
      minimum. `SidebarMenuButton`'s own root `button` has `overflow-hidden`
      but not `whitespace-nowrap`, so a bare text child still wraps rather
      than eliding — and the logo `Link` had `shrink-0`, actively
      preventing that row from shrinking at all.
- [x] 2a.2 Fixed in `apps/playground/src/routes.rs` only (no registry
      change — this is the playground's own composition choice for the
      generic `children: Element` `SidebarMenuButton` accepts, not a
      registry API defect): wrapped each nav label in
      `span { class: "min-w-0 flex-1 truncate", "{label}" }`; changed the
      logo `Link`'s class from `"flex shrink-0 items-center ..."` to
      `"flex min-w-0 items-center ..."`, moved `shrink-0` onto the logo
      `img` (which should stay a fixed 32px square), and wrapped "adico
      playground" in `span { class: "min-w-0 truncate", ... }`; added
      `min-w-0` to the nav `ResizablePanel`'s own class. Verified `cargo
      check -p adico-playground` compiles, `cargo fmt --all --check` and
      `cargo run -p adico-xtask -- styling-usage check` (69 items) both
      pass.
- [x] 2a.3 Live-verified via `dx serve`: dragged the nav column to its 12%
      minimum and confirmed every multi-word label ("Accordion", "Alert
      Dialog", "Aspect Ratio", "Attachment", "Breadcrumb", "Button") and
      the logo now truncate with an ellipsis on a single line instead of
      wrapping/clipping; no console errors.

## 3. Validation

- [x] 3.1 Run `cargo fmt --all --check`, `cargo check --locked --workspace`,
      this repo's canonical clippy/test scope (`cargo clippy --locked -p
      adico-cli -p adico-primitives -p adico-registry-core -p
      adico-test-utils -p adico-xtask --all-targets -- -D warnings` and
      `cargo test --locked` across the same five packages — NOT
      `--workspace`, since `--workspace` clippy currently surfaces
      pre-existing errors unrelated to this change, outside the files it
      touches). Verify all pass. **Done**: all pass.
- [x] 3.2 Run `cargo check -p adico-playground`, `cargo test -p
      adico-playground`, and `cargo clippy -p adico-playground --all-targets
      -- -D warnings` explicitly (outside the canonical scope, but this
      change's actual package). Verify no new warnings/errors introduced by
      this change's files (`demo.rs`, `routes.rs`) versus the pre-existing
      baseline. **Done**: check passes; 89 tests pass; clippy shows the
      same 26 pre-existing errors from unrelated files (23 unused-glob
      imports for unwired generated components, a collapsible-if, a
      redundant-closure) — none reference `demo.rs`/`routes.rs`.
- [x] 3.3 Run `cargo run -p adico-xtask -- playground-controls check` and
      confirm it still passes with zero diff — proof this change touches no
      generated file. **Done**: passed, 69 item(s), zero diff.
- [x] 3.4 Run `openspec validate add-playground-resizable-panels --strict`
      and confirm it passes. **Done**: "Change
      'add-playground-resizable-panels' is valid".
- [x] 3.5 Note explicitly in the final report: no database, WebAssembly-target,
      or CLI-installation validation surface applies to this change (not
      applicable, not skipped). **Done**: recorded — this is playground-app
      layout/UI work only; no database, wasm32 target, or CLI-installation
      fixture is touched.

## 4. Post-completion fix: `ResizableHandle`'s grip, and margin around it

- [x] 4.1 **Regression found by the user after this change shipped**: the
      grip decoration inside `ResizableHandle` (`with_handle: true`) looked
      visually wrong specifically on the horizontal line (`Demo`'s
      preview/controls split) while looking fine on the vertical line
      (`Layout`'s nav/content split). Root cause, found via direct
      `getComputedStyle`/`getBoundingClientRect` measurement, not guessing:
      **two separate, independent registry bugs** in
      `registry/ui/resizable.rs`, both pre-existing (not introduced by this
      change — this change is what exercised the `Vertical` direction with
      `with_handle: true` for the first time in this codebase):
      1. The grip's box was a single static `"h-4 w-3"` class, never
         rotated for `direction` — reads as visually wrong on whichever
         orientation it wasn't shaped for.
      2. The grip is a flex child of the handle (itself `display: flex`
         with no explicit `flex-direction`, i.e. row). For `Horizontal`
         direction, the handle's own main-axis (width) is constrained to
         `w-px` (1px) — the grip's default `flex-shrink: 1` +
         `min-width: auto` let the flex algorithm crush it down to ~2px
         wide instead of its coded 12px, confirmed via
         `getComputedStyle`. `Vertical`'s 1px constraint (`h-px`) lands on
         the handle's cross axis instead, which `align-items: center` only
         centers on and never shrinks — so this bug was invisible in the
         one orientation this codebase had tested until this change added
         the other one.
      3. (Found after fixing 1–2, still visually wrong) The grip's
         `border bg-border` used two tokens that happened to collide: plain
         `border`'s outline color resolves via `currentColor` to this
         theme's near-white foreground text color, while `bg-border` fills
         with the theme's dark slate `--color-border` token — the SAME
         dark token the 1px line underneath is filled with. Wherever the
         grip's edge coincided with the line (its bottom edge on a
         horizontal line), the dark fill blended into the equally-dark
         line right behind it, and only the near-white outline stayed
         visible on the non-touching edges — an open-bottomed "bracket"
         look instead of a solid chip.
- [x] 4.2 Fixed all three in `registry/ui/resizable.rs` (a genuine registry
      defect reproducible by any consumer using `Vertical` direction with
      `with_handle: true` — not a playground-only issue, so fixed at the
      source, not worked around in `apps/playground`): grip box dimensions
      now swap by `direction` (`h-4 w-3` / `h-3 w-4`); added `shrink-0` to
      the grip so it never gets crushed regardless of which axis the
      handle constrains; changed the grip's fill from `bg-border` to
      `bg-foreground` (the same near-white the border outline was already
      resolving to via `currentColor`) and dropped the now-redundant
      `border` class, so the whole chip renders as one consistently
      visible solid piece regardless of which edge touches the line.
      `bg-background` was tried as an intermediate step for the fill and
      made it worse (confirmed live) — it's also a dark token and blended
      into the surrounding dark canvas instead of the line. Propagated to
      `apps/playground/src/components/ui/resizable.rs`, bumped the
      registry checksum three times (once per iteration) via `cargo run -p
      adico-xtask -- registry build`, and re-validated after each.
      `cargo run -p adico-xtask -- styling-usage check` (69 items) passed
      after every iteration.
- [x] 4.3 **Second regression, same session**: the resize handle had no
      visual breathing room — it sat flush between the preview canvas's
      border and the "Component controls" header's border, making it hard
      to see as a distinct, grabbable element. Fixed by adding `gap-3` to
      `Demo`'s `ResizablePanelGroup` class (`apps/playground/src/components/demo.rs`)
      — `ResizablePanelGroup`'s own base class (`"flex h-full w-full"` +
      direction) has no competing `gap-*` utility, so this appends cleanly
      with no `cn()`-merge conflict; a flex `gap` is excluded from the
      space `ResizablePanel`'s percentage `flex-basis`es divide up, so it
      doesn't perturb the existing 70/30 sizing math.
- [x] 4.4 Live-verified all three grip fixes plus the margin addition via a
      temporary `dx serve` instance (the user's own separately-running
      instance on port 3000 hadn't picked up the file changes when
      checked, likely a stalled watcher — not investigated further since a
      fresh instance gave a reliable, directly-observable result):
      measured both grips' `getBoundingClientRect()` before/after (2px→12px
      width fix confirmed numerically, not just visually), zoomed
      screenshots confirmed both grips now render as solid, correctly-sized,
      clearly visible chips with no bracket artifact, and confirmed visible
      breathing room around the handle on both sides. Re-ran `cargo fmt
      --all --check`, `cargo check -p adico-playground`, `cargo test -p
      adico-playground` (89 passed), `cargo run -p adico-xtask --
      playground-controls check` (zero diff), `cargo run -p adico-xtask --
      primitive-usage check`, and `openspec validate
      add-playground-resizable-panels --strict` — all pass. Killed the
      temporary verification instance afterward; the user's own port 3000
      instance was left untouched.

## 5. Post-completion fix: grip reads as a blank rectangle, not a handle

- [x] 5.1 **Feedback from the user**: even after task 4's fixes (correct
      size, no crush, solid visible fill), the grip still just looked like
      a plain rounded rectangle, not recognizable as a drag handle.
      Directed to check upstream shadcn/ui's own `resizable.tsx` for
      reference — fetched via `gh api
      repos/shadcn-ui/ui/contents/apps/v4/registry/new-york-v4/ui/resizable.tsx`
      rather than reproducing it from memory. Two concrete differences
      found: (1) upstream never swaps the grip's own `h-4`/`w-3` classes by
      orientation — it keeps the box fixed and instead applies
      `rotate-90` to the whole box via
      `[&[aria-orientation=horizontal]>div]:rotate-90`; (2) upstream
      renders a `<GripVerticalIcon className="size-2.5" />` inside the
      box — the dots-pattern glyph is what actually reads as "a handle,"
      not the bare box shape.
- [x] 5.2 Rewrote `registry/ui/resizable.rs`'s grip to match: `grip_class`
      is now a fixed `"...h-4 w-3..."` plus a conditional `"rotate-90"`
      when `direction == ResizableDirection::Vertical` (this component's
      own aria-orientation mapping puts that case at `aria-orientation:
      "horizontal"`, exactly matching upstream's selector target) instead
      of the earlier per-direction `h-4 w-3` / `h-3 w-4` swap. Added
      `use adico_primitives::icons::GripVertical;` and render
      `GripVertical { class: "size-2.5 shrink-0 text-background" }` inside
      the grip — `text-background` overrides the icon's default
      `stroke="currentColor"` (confirmed against the icon's generated
      source, `packages/dioxus-icons` via `~/.cargo/registry`), since
      without it the near-white stroke would be invisible on the
      near-white `bg-foreground` chip. `shrink-0` on the icon mirrors the
      same flex-crush reasoning already documented for the chip itself.
- [x] 5.3 Updated `registry/registry.json`'s `resizable` entry: added
      `{ "crate": "adico-primitives", "version": "=0.1.0" }` to
      `cargoDependencies` (no `"web"` feature — confirmed by cross-checking
      every other icon-only registry item, e.g. `spinner`/`breadcrumb`,
      which also omit it; only items using actual interactive/web-specific
      primitive behavior, e.g. `checkbox`/`mode-toggle`, declare it), and
      adjusted the description from "no dedicated primitive dependency" to
      "no dedicated primitive behavior" (accurate now that it depends on
      `adico-primitives` for the icon, just not for its own drag/resize
      behavior). Updated `statics/primitive_usage/resizable.json`'s
      `primitiveModules` from `[]` to `["icons"]`, matching the exact
      "icons import is purely decorative" phrasing convention already used
      by `attachment.json`/`carousel.json`/`data-table.json` (two of which
      already referenced "resizable.json's identical rationale" as if this
      update was expected). Verified with `cargo run -p adico-xtask --
      primitive-usage diff` — no drift between the hand-written record and
      the generator's own detected module list.
- [x] 5.4 Propagated to `apps/playground/src/components/ui/resizable.rs`,
      bumped the registry checksum, ran `cargo run -p adico-xtask --
      registry build`/`registry validate`. Full validation re-run: `cargo
      fmt --all --check`, `cargo check --locked --workspace`, canonical
      clippy/test scope, `cargo check`/`test -p adico-playground` (89
      passed), `playground-controls check`, `primitive-usage check`,
      `styling-usage check`, `registry validate`, `openspec validate
      add-playground-resizable-panels --strict` — all pass.
- [x] 5.5 Live-verified via a fresh temporary `dx serve` instance: both
      grips now show the six-dot `GripVertical` glyph, correctly rotated
      90° on the horizontal handle to match upstream's own visual
      convention exactly; confirmed via direct `PointerEvent` dispatch
      that dragging still works correctly after the rotation change (70/30
      → 58.4/41.6, sums to 100); no console errors. Killed the temporary
      instance afterward; the user's own port 3000 instance was left
      untouched throughout.

## 6. Post-completion refinement: thinner grip

- [x] 6.1 **User preference, after task 5's fix**: the grip, now correctly
      shaped/colored/iconed to match upstream, was still visually bulkier
      than the user wanted. Reduced `registry/ui/resizable.rs`'s
      `grip_class` from `h-4 w-3` (16×12px) to `h-3 w-2` (12×8px), and the
      `GripVertical` icon from `size-2.5` to `size-1.5`, keeping the same
      fixed-box-plus-`rotate-90` mechanism from task 5 unchanged — a
      deliberate, explicit divergence from upstream shadcn's own default
      size (already matched exactly in task 5), not a bug fix. Propagated
      to `apps/playground/src/components/ui/resizable.rs`, bumped the
      registry checksum, `registry build`/`validate`/`styling-usage check`
      all pass.
- [x] 6.2 Live-verified via a fresh temporary `dx serve` instance: both
      grips render noticeably slimmer while the dots icon stays clearly
      legible at the smaller size. Re-ran `cargo test -p adico-playground`
      (89 passed), `playground-controls check`, `primitive-usage check`,
      `openspec validate add-playground-resizable-panels --strict` — all
      pass. Killed the temporary instance afterward.
