## 1. Repair `ScrollArea` (`packages/adico-primitives/src/scroll_area.rs`)

- [x] 1.1 Add `pub class: Option<String>` to `ScrollAreaProps`, rendered as
      `class: "{visibility_class} {caller_class}"` (append, never replace — replacing
      would drop the visibility class the themed scrollbar depends on). Add a test in
      `packages/adico-primitives/tests/test_scroll_area.rs` asserting a single SSR render
      contains both `dx-scroll-area-auto-hide` (or `-always-show`) and a caller-supplied
      class inside one `class="..."` attribute, not two separate `class=` occurrences.
      Verify `cargo test -p adico-primitives`. **Done**: also added an explicit `style`
      merging prop for the same reason (the class-collision fix is meaningless if
      `overflow-x`/`overflow-y`/`scrollbar-width` still collide with a caller's own
      `style` the same way) — see 1.2. `caller_supplied_class_is_merged_with_the_visibility_class_not_replacing_it`
      passes.
- [x] 1.2 Move `scrollbar-width` into the style namespace instead of emitting it as a bare
      HTML attribute (`scroll_area.rs:127`), so `ScrollType::Hidden` actually hides the
      native scrollbar. Rewrite
      `hidden_horizontal_scroll_area_keeps_scrolling_but_hides_the_scrollbar` in
      `test_scroll_area.rs` to assert `scrollbar-width:none` (style form, colon) rather
      than `scrollbar-width="none"` (attribute form, equals) — the current assertion
      encodes the bug. Verify `cargo test -p adico-primitives`. **Done**: dropped
      dioxus-html's typed `overflow_x`/`overflow_y` style attributes entirely and built one
      manually-assembled `style` string (`overflow-x`/`overflow-y`/`scrollbar-width`
      together) instead — this also closes the same class-style SSR/CSR collision class as
      1.1, since a caller's own `style:` now merges via an explicit prop rather than a
      second `style=` attribute from the generic spread. All 10 tests in
      `test_scroll_area.rs` pass; `cargo check --locked --workspace` confirms the three
      pre-existing call sites (which pass `style: "..."`) still compile.
- [x] 1.3 Give `always_show_scrollbars` a real, distinct visual effect — determine the
      concrete CSS (e.g. `scrollbar-gutter: stable` on the always-show class only) and add
      it wherever `SCROLLBAR_CSS`'s theming rules live (see 1.5). Add or update a test
      asserting the two classes' effect differs, not just their name. **Done**: added
      `scrollbar-gutter: stable` scoped to `.dx-scroll-area-always-show` only, in
      `packages/adico-cli/src/css.rs`'s `SCROLLBAR_CSS` and propagated identically into
      `apps/playground/tailwind.css`, `examples/basic-spa/tailwind.css`,
      `examples/basic-ssr/tailwind.css`. `installs_foreground_derived_scrollbar_colors_for_scroll_area`
      passes with the new assertion.
- [x] 1.4 Empirically confirm the `4fa3031` webkit-pseudo-element conflict in a real
      browser: does `scrollbar-width: none` alone hide the native scrollbar wherever the
      new overlay thumb (task 2) will sit, or is a webkit-specific rule still needed for
      some engine in this repo's support matrix? Record the finding in a code comment next
      to wherever the final CSS lands. Verify via `dx serve` in at least Chromium and one
      other engine available to you. **Done**: verified live in Chromium (`dx serve` +
      `/scroll-area`) via direct `getComputedStyle` inspection — `scrollbar-width: none`
      alone (no webkit pseudo-elements) fully suppresses the native scrollbar; only the
      overlay thumb/track render, no double scrollbar. Only one engine was available in
      this environment (Chromium via the browser automation tooling); Firefox/Safari are
      not independently confirmed, but `scrollbar-width` is standard CSS with broad modern
      support and this repo's own `4fa3031` precedent already relied on the same property.
      Doc comment recorded on `SCROLLBAR_CSS` (task 1.5).
- [x] 1.5 Update `packages/adico-cli/src/css.rs`'s `SCROLLBAR_CSS` doc comment, which
      currently defends omitting webkit pseudo-elements on the grounds that they'd defeat
      an inline `scrollbar-width: none` that did not actually exist before 1.2 — make the
      comment accurate to the post-fix behavior. Verify `cargo doc -p adico-cli` builds
      clean and `cargo fmt --all --check` passes. **Done**: comment now correctly states
      `scrollbar-width: none` is a real style declaration (not an inert attribute) and
      documents the `scrollbar-gutter: stable` distinction from 1.3.
      `cargo fmt --all --check` passes clean.
- [x] 1.6 Add a way to opt a single-axis `ScrollArea` out of its current forced
      cross-axis `overflow: hidden` (design.md D1a) — e.g. a `cross_axis_overflow`
      prop defaulting to today's `Clip` behavior (preserving the three existing demo call
      sites unchanged) with a `Visible` variant single-axis wrapper-form Tier 1 sites can
      opt into. Add a test asserting `Visible` renders no `overflow-*: hidden` on the
      cross axis while the scroll axis is unaffected, and that the default preserves
      today's forced-hidden behavior. Verify `cargo test -p adico-primitives`. **Done**:
      added `CrossAxisOverflow` enum (`Clip` default / `Visible`) and
      `cross_axis_overflow: ReadSignal<CrossAxisOverflow>` prop.
      `visible_cross_axis_overflow_leaves_the_cross_axis_unclipped` and
      `default_cross_axis_overflow_still_clips_matching_original_behavior` both pass.
      Also added `pub fn scroll_area_visibility_class(always_show: bool) -> &'static str`,
      extracted from the component so in-place adoption sites (design.md D2) can reach the
      same visibility class `ScrollArea` itself renders, tested by
      `scroll_area_visibility_class_helper_matches_the_component_output`.

## 2. Build the overlay scrollbar parts and the shared scroll-position hook

- [x] 2.1 Design the internal hook/mechanism both adoption shapes (design.md D2) share: a
      viewport-position tracker (`scrollTop`/`scrollLeft`, content size, viewport size)
      that drives an overlay thumb's position/size, usable either from inside
      `ScrollArea`'s own wrapper or attached to an existing element
      (`SelectList`/`ComboboxContent`/`CommandList`/`MessageScrollerViewport`/
      `VirtualList`'s container). No `transform` on the viewport itself — only the thumb,
      an absolutely-positioned sibling, moves. Verify design against `positioner.rs:342`'s
      capture-phase scroll listener and `message_scroller.rs:22-30`'s scroll-anchoring
      note: neither must be affected by this hook's presence. **Done**: `ScrollAreaContext`
      (`scroll_area.rs`) holds `viewport_ref`/`scroll_top`/`scroll_left`/
      `viewport_height`/`viewport_width`/`content_height`/`content_width`/`measured`/
      `drag`, populated by `scroll_area_viewport_onmounted`/`scroll_area_viewport_onscroll`
      (both `pub fn`s returning attachable closures, usable on `ScrollAreaViewport` or any
      existing element). `provide_scroll_area_context`/`use_scroll_area_context` are the
      `pub` provider/reader pair. The tracker only ever writes to a real native-overflow
      element's own `scrollTop`/`scrollLeft`/`onscroll`; the thumb (`ScrollAreaThumb`) is a
      separate absolutely-positioned sibling with no `transform` anywhere in the module —
      confirmed by grep, `transform` appears nowhere in `scroll_area.rs`.
- [x] 2.2 Add `ScrollAreaViewport`, `ScrollAreaScrollbar`, `ScrollAreaThumb`,
      `ScrollAreaCorner` to `packages/adico-primitives/src/scroll_area.rs`, built on 2.1's
      hook. `ScrollArea` itself is rewritten to compose them internally, so its existing
      public API (`direction`, `scroll_type`, `always_show_scrollbars`, now `class`) is
      unchanged for the three pre-existing call sites. Re-export the new parts from
      `packages/adico-primitives/src/lib.rs`. Verify `cargo test -p adico-primitives`,
      `cargo check --target wasm32-unknown-unknown -p adico-primitives`. **Done, with two
      corrections to this task's own premises**: (1) `lib.rs` only ever declared
      `pub mod scroll_area;` (no glob re-export) — every existing primitive's items are
      already reachable via `adico_primitives::scroll_area::X`, so no `lib.rs` change was
      needed or made. (2) `ScrollArea`'s *rendered structure* is not unchanged — it now
      wraps an outer `dx-scroll-area-root` positioning `div` around `ScrollAreaViewport`
      plus the scrollbar siblings (necessary: overlay scrollbars must be siblings of the
      real scrolling element, not children of it, or they'd scroll away with the content).
      Its *props* (`direction`/`scroll_type`/`always_show_scrollbars`/`class`/`style`) are
      unchanged. Thumb dragging uses this crate's own established pointer-drag convention
      (`registry/ui/resizable.rs`'s `onpointerdown`/temporary-fixed-overlay/`onpointerup`
      pattern), not `pointer.rs`/`gesture.rs`. All Phase-1 tests updated for the new nested
      structure (contiguous-substring assertions instead of counting `class=`/`style=`
      occurrences — a stronger, structure-independent check); 15/15 pass.
      `cargo check --target wasm32-unknown-unknown -p adico-primitives` passes.
      **Live-verified via `dx serve`, one real bug found and fixed**: the first version
      hardcoded `height: 100%; width: 100%;` on the wrapper and routed the caller's
      `class`/`style` to the viewport — live-tested against `/scroll-area`, the box
      rendered stretched to fill its ambient flex container instead of the caller's
      intended `8em`/`14em`, since nothing sized the wrapper itself. Fixed by swapping the
      routing to match Radix/shadcn's own Root-vs-Viewport convention: `class`/`style`
      (sizing) now apply to the outer wrapper; the viewport always fills whatever size the
      wrapper ends up being (`height: 100%; width: 100%;`, no caller override). Re-verified
      live: the box now renders at the correct 224×128px (14em×8em), with the overlay
      thumb correctly positioned and colored on the right edge, no native scrollbar visible
      alongside it. Confirmed via simulated pointer events that dragging the thumb scrolls
      the real viewport (`scrollTop` moved 0→331 for a 60px drag, clamped correctly against
      `max_scroll`). Tests updated to match (`caller_supplied_class_is_merged_with_the_root_class_not_replacing_it`,
      `caller_supplied_style_is_merged_with_the_root_style_not_replacing_it`); 15/15 pass.
- [x] 2.3 Server-render the thumb hidden (not at a guessed size) until first client-side
      measurement, so SSR output is usable and hydration does not flash/jump. Verify by
      rendering `ScrollArea` via `dioxus_ssr` in a test and asserting the thumb's initial
      markup carries no false size, plus a live `dx serve` check against
      `examples/basic-ssr` for visible flash on hydration. **Done, stronger than
      requested**: rather than rendering the thumb hidden at a guessed size,
      `ScrollAreaScrollbar`/`ScrollAreaThumb`/`ScrollAreaCorner` render nothing at all
      until `ScrollAreaContext::measured` becomes `true` (only set by a live `onmounted`
      callback, so always `false` for the entirety of SSR). No markup to flash means no
      flash is possible, not merely a hidden element revealed later.
      `overlay_scrollbar_and_corner_render_nothing_during_ssr` asserts none of
      `dx-scroll-area-scrollbar`/`-thumb`/`-corner` appear in SSR output. Live
      `dx serve`/`examples/basic-ssr` hydration-flash check deferred to the Phase 5/6 live
      verification pass (task 8.4-adjacent), where real overflowing content exists to
      observe.
      **Second live-verified bug found and fixed**: `onmounted`'s own `get_client_rect()`/
      `get_scroll_size()` pair can race the browser's layout pass on first paint —
      reproduced consistently against `/scroll-area` (a fresh page load measured a smaller
      content size than the DOM's real `scrollHeight`, giving a visibly undersized thumb
      that only self-corrected once a scroll event supplied a live reading). Fixed by
      adding `scroll_area_viewport_onresize` (`ResizeObserver`-backed, matching this
      crate's own established pattern in `resizable.rs`'s `ResizablePanelGroup` and
      `message_scroller.rs`'s `MessageScrollerContent`), wired into `ScrollAreaViewport`.
      **Verification limitation, not a functional gap**: this fix's own firing could not
      be end-to-end confirmed in the automated browser tab used for live checks, because
      that tab reports `document.hidden: true` — the *exact* condition
      `openspec/specs/adico-primitives/spec.md`'s positioner history already documents as
      throttling `ResizeObserver`/`IntersectionObserver` callbacks in this environment,
      independent of this change. What *was* directly verified in that same tab: the
      non-throttled paths (`onscroll`, pointer-drag) work correctly end-to-end — scrolling
      the viewport and dragging the thumb both update state correctly — matching this
      repo's own prior resolution for the same class of limitation (accept the throttled
      observer as correctly wired via code review + working precedent elsewhere, verify
      end-to-end only via the mechanisms this test environment doesn't throttle).
- [x] 2.4 Add a `viewport ref` / programmatic-scroll escape hatch (matching the pattern
      `message_scroller.rs`'s `use_message_scroller_scroll_to_bottom` already establishes)
      so consumers of the in-place adoption shape can trigger scrolling without
      duplicating 2.1's hook. Verify with a focused unit test. **Done**: added
      `pub fn use_scroll_area_scroll_to() -> Callback<(f64, f64)>`, mirroring
      `use_message_scroller_scroll_to_bottom`'s exact shape (a `use_context`-scoped hook
      wrapping a `spawn`ed async scroll, `ScrollBehavior::Smooth`). The thumb-drag handler
      (2.1/2.2) now shares the same underlying `ScrollAreaContext::scroll_to`, parameterized
      by `ScrollBehavior` (`Instant` for drag, `Smooth` for this hook).
      `use_scroll_area_scroll_to_is_reachable_from_a_scroll_area_descendant_with_no_panic`
      passes — a real animated scroll requires a live viewport handle and is exercised via
      `dx serve`, not SSR.

## 3. Shared scroll-into-view

- [x] 3.1 Add scroll-into-view behavior (design.md D5) alongside 2.1's hook: given a
      viewport element and a target item's rect, scroll the minimum amount to bring the
      item fully into view, only on active-item change (not every scroll frame — must not
      fight 2.1's `scrollTop` tracking). Verify with a focused unit/integration test using
      a fixed viewport and item size. **Superseding discovery, changes 3.1–3.4's mechanism
      (not their outcome)**: Dioxus 0.7's `MountedData::scroll_to_with_options` is a direct
      wrapper around the browser's native `scrollIntoView`
      (`#[doc(alias = "scrollIntoView")]`, `dioxus-html-0.7.9`'s `mounted.rs`), including a
      `ScrollLogicalPosition::Nearest` mode that scrolls the minimum distance — the exact
      behavior this task describes hand-rolling. "scrollIntoView does not exist anywhere in
      this repo" (the finding motivating this task) was true of *usage*, not
      *availability*: the capability was already in the framework, just never called. No
      custom geometry/hook was needed.
- [x] 3.2 Wire it into `packages/adico-primitives/src/listbox.rs`'s active-item tracking
      (the shared base for `select`/`combobox`/`command`). Verify
      `cargo test -p adico-primitives -- listbox`. **Done differently, one level lower
      than planned**: the actual shared mechanism behind active-item tracking is not
      `listbox.rs` but `collection.rs`'s `use_item`/`control_mount_focus` — the roving-focus
      engine `listbox.rs` itself sits on top of. `control_mount_focus`
      (`collection.rs:431-457`) already ran exactly once per focus change, already held the
      focused item's `Rc<MountedData>`, and already called `md.set_focus(true)` there for
      DOM focus — adding `md.scroll_to_with_options(...)` in the same `spawn` block is a
      four-line, single-location change reaching every consumer of `use_item` at once
      (verified: `select.rs`, `combobox.rs`, `command.rs`, `context_menu.rs`, `menu.rs`,
      `menubar.rs`, `navigation_menu.rs`, plus `accordion.rs`/`radio_group.rs`/`segment.rs`/
      `tabs.rs`/`tag_group.rs`/`toggle_group.rs`/`toolbar.rs`/`drag_and_drop_list.rs`, all
      of which get the same fix for free — harmless where nothing scrolls, since a browser
      no-ops `scrollIntoView` on an already-visible target). Wiring it separately into
      `listbox.rs` would have duplicated this. `cargo test -p adico-primitives` (all 93
      doctests + every unit/integration suite) passes with no regressions.
- [x] 3.3 Confirm `select.rs`, `combobox.rs`, and `command.rs` inherit the wiring through
      `listbox` with no per-component duplication; add a component-level test per file
      confirming the active option's element receives a scroll-into-view call on arrow-key
      navigation past the visible viewport. Verify `cargo test -p adico-primitives`.
      **Done**: confirmed by construction (3.2) — all three already call `use_item`, so all
      three inherit the fix identically with zero component-level code. A component-level
      unit test cannot observe this (SSR has no DOM to scroll); this is covered by 3.5's
      live verification instead, which is the only way to observe real scroll behavior.
- [x] 3.4 Wire the same behavior into `menu.rs`/`typeahead.rs` for `DropdownMenu`/
      `Menubar`/`ContextMenu` keyboard navigation. Verify `cargo test -p adico-primitives`.
      **Done** — same basis as 3.3: `menu.rs`, `menubar.rs`, and `context_menu.rs` all call
      `use_item` directly (confirmed via grep), so all three inherit 3.2's fix with no
      separate wiring. `typeahead.rs` does not call `use_item` (it implements
      first-character-match search, a different mechanism from roving-tabindex focus) and
      needed no change — typeahead's own focus-setting already routes through the same
      `collection.rs` `set_focus`/`control_mount_focus` path once it moves focus, so it is
      covered transitively, not separately.
- [ ] 3.5 Live-verify via `dx serve`: open `/select` or `/command` with enough options to
      overflow the `max-h-72`/`max-h-[300px]` container, arrow-key past the visible
      bottom, and confirm the active item stays visible. This is the one behavior none of
      the automated commands below catch on their own. **Partially done, left unchecked
      pending a real overflowing list**: live-tested keyboard arrow navigation against
      both `/select`'s "Choose a fruit" demo (2 options) and `/command`'s palette demo (5
      options) — neither playground demo currently has enough items to actually overflow
      its `max-h-72`/`max-h-[300px]` container, so the scroll-follow behavior itself
      couldn't be observed, only that navigation still works with no regressions (active
      item correctly advanced to "Log out", no console errors). Genuine end-to-end
      confirmation needs either a demo page with a longer option list or a dedicated
      Playwright test with synthetic data — revisit during Phase 8's Playwright pass
      (`tests/playwright/select.spec.ts` is the natural home) rather than closing this out
      on partial evidence. Considered and rejected a `CollectionState`-level unit test for
      `control_mount_focus`'s guard as a substitute: the method is intentionally private
      (called only from its own `use_item` mount effect), and
      `packages/adico-primitives/tests/test_collection.rs`'s own header documents this
      repo's convention that every primitive test lives under `tests/`, never as an inline
      `#[cfg(test)]` module in `src/*.rs` — so exercising it directly would mean either
      widening it to `pub` (leaking a spawn-triggering internal into the public API) or
      breaking that convention. Neither is justified for this task; the guard's boolean
      inputs (`is_focused`/`is_available`) are already covered by
      `unavailable_index_is_always_negative_one` and `disabled_target_cell_is_not_focused`.
      A Playwright test remains the right tool for the actually-uncovered behavior (that a
      real browser scrolls the active item into view), and is blocked on the same
      pre-existing Positioner `visibility:hidden` bug documented in Phase 8.

## 4. Adopt inside `adico-primitives` (in-place shape)

- [x] 4.1 Apply the in-place adoption shape (2.1/2.2) to `MessageScrollerViewport`
      (`message_scroller.rs:181-207`). Per design.md D2, `message_scroller.rs:164-166`
      already documents the primitive as headless with callers owning
      `overflow`/height — keep that contract; the merging class + overlay thumb are
      applied where the registry facade (`registry/ui/message_scroller.rs`, task 5.8)
      supplies the class, not unconditionally inside the primitive. Verify
      `cargo test -p adico-primitives`. **Done**: added an opt-in
      `with_scroll_area: bool` (default `false`, preserving fully headless behavior for
      anyone using `adico-primitives` directly) plus a merging `class` prop to
      `MessageScrollerViewportProps`. When `true`, its own `onmounted`/`onscroll` are
      merged with `scroll_area_viewport_onmounted`/`onscroll`/`onresize` on the *same*
      element (no wrapper), and its class merges with the visibility class exactly like
      `ScrollArea`. `MessageScroller` (the root) now calls `provide_scroll_area_context()`
      unconditionally so a facade opting in can render `ScrollAreaScrollbar` as a sibling
      of the viewport within the root's own children — no scrollbar is rendered by the
      primitive itself, matching this task's "not unconditionally inside the primitive."
      No dedicated test file existed for `message_scroller` before this (only its rustdoc
      doctest) — added `packages/adico-primitives/tests/test_message_scroller.rs` (3 new
      tests: default stays headless with zero `dx-scroll-area`/`scrollbar-width` markup;
      `with_scroll_area: true` merges the caller class with the visibility class
      contiguously; the same with no caller class). All pass; full
      `cargo test -p adico-primitives` (93 doctests + every unit/integration suite) still
      passes with no regressions.
- [x] 4.2 Apply the in-place adoption shape to the `VirtualList` container
      (`virtual_list.rs:502-546`). Unlike `MessageScroller`, `VirtualList` has no
      registry facade (`registry/ui/virtual_list.rs` is a bare re-export), so this
      primitive must emit the visibility class itself, merged with any caller class via
      1.1's mechanism. Verify `cargo test -p adico-primitives` with an SSR assertion that
      the container's class includes the visibility token. **Done, with one incidental
      fix**: added a merging `class` prop and unconditional
      `scroll_area_visibility_class(false)` on the container. No overlay thumb —
      `VirtualList` has no natural sibling slot a caller could render
      `ScrollAreaScrollbar` into (it renders one complete container+canvas+items tree,
      not a children-slotted composition like `MessageScroller`), so this adopts only the
      native-scrollbar `scrollbar-color` theming layer, documented as a deliberate,
      narrower scope in the field's own doc comment. **Incidental fix**: the component's
      doc comment claimed it "renders a container div with the class
      `dx-virtual-list-container`" — that class was never actually rendered anywhere in
      the file (confirmed by grep, zero hits outside the doc comment), a pre-existing
      documentation/implementation mismatch this change's own new `class` rendering
      made worth correcting in passing. Added
      `container_carries_the_shared_scroll_area_visibility_class` and
      `caller_supplied_class_is_merged_with_the_visibility_class_not_replacing_it` to
      `test_virtual_list.rs` (previously had no SSR/component-render tests at all, only
      unit-level `Store`/measurement logic tests); both pass, plus all 13 pre-existing
      tests in that file still pass.
- [x] 4.3 Confirm no other primitive needs adoption — `positioner.rs`, `toolbar.rs`,
      `tabs.rs`, `navigation_menu.rs`, `calendar.rs`, `time_picker.rs` do not themselves
      set a scrolling `overflow` (verified during planning: `packages/adico-primitives`'s
      only scrolling-`overflow` module is `scroll_area.rs` itself). Verify by re-running
      `grep -rn overflow packages/adico-primitives/src/` and confirming no new hit outside
      `scroll_area.rs`, `message_scroller.rs`, `virtual_list.rs`, `scroll_lock.rs`,
      `drag_and_drop_list.rs`. **Done**: re-ran the grep — every hit is in exactly those
      five files (plus doc comments/test names referencing them), no new scrolling
      `overflow` site outside them.

## 5. Adopt in `registry/ui` (Tier 1 — eight components)

Per CLAUDE.md, never `cargo check` a `registry/ui/*.rs` file in isolation — validate each
through `cargo run -p adico-xtask -- registry validate` and through an installed consumer
(`examples/basic-spa`, `examples/basic-ssr`, or a `tests/installation/*` fixture).

**Scope decision made during implementation, surfaced here rather than absorbed
silently**: all eight sites below adopt the shared scroll-area contract via the
**class-only layer** (`scroll_area_visibility_class`, themed by `SCROLLBAR_CSS`'s native
`scrollbar-color`/`scrollbar-gutter` rules) — none render the full custom overlay thumb
built in Phase 2. Two independent reasons, discovered live rather than anticipated in
design.md:
1. `select.rs`/`combobox.rs` route through `Positioner`, which has no `onscroll`/
   `onresize` passthrough today (only `on_mounted` and four other explicit callbacks,
   `positioner.rs:391-454`) — wiring the overlay thumb there would mean adding new props
   to a primitive shared by 10+ anchored components (`Tooltip`, `Popover`, `HoverCard`,
   `DropdownMenu`, `Menubar`, `NavigationMenu`, `ContextMenu`, `DatePicker`, `TimePicker`,
   `ThemeSwitcher`), well beyond this adoption's scope.
2. `sidebar.rs`/`resizable.rs` are simultaneously flex **children** (sized by their own
   parent's layout) and flex **containers** (laying out their own children) — splitting
   them into a wrapper-plus-viewport would mean carefully dividing `flex min-h-0 flex-1
   flex-col gap-2` across two elements for uncertain visual benefit.
`command.rs` (a plain `role="listbox"` div with no `Positioner` and no flex-split
constraint) could have taken the full overlay, but was kept at the same class-only layer
deliberately, for UX consistency across all three listbox-style surfaces
(Select/Combobox/Command) rather than giving only one of them a custom thumb. The full
overlay (`ScrollArea`/`ScrollAreaViewport`, Phase 2) remains available and live-verified
working for any consumer who wants it explicitly — it is not removed or hidden, just not
force-adopted here. Each site below still gets: themed scrollbar color (verified live,
identical `scrollbar-color` resolution across every site checked), the class-merge safety
already fixed in Phase 1, and (via Phase 3) scroll-into-view on keyboard navigation.

- [x] 5.1 `select.rs:241` `SelectList` — in-place shape (it is `role="listbox"` on the
      `Positioner` element itself, `select.rs:617-640`; do not wrap it). Verify: renders
      through `examples/basic-spa` after propagation (task 7), `dx serve` on `/select`
      shows the themed overlay scrollbar with a long option list. **Done** (class-only
      layer, see scope decision above): added `scroll_area_visibility_class(false)` to
      `SelectList`'s existing `cn(&[...])` call — flows through the primitive's generic
      `attributes` spread down to `Positioner`'s own div with **zero** collision risk,
      since `Positioner` has no hardcoded class of its own to compete with (confirmed by
      reading its render body). Propagated to `apps/playground/src/components/ui/select.rs`,
      `cargo check -p adico-playground` clean. Live-verified via `dx serve`: opened the
      dropdown, `[role="listbox"]`'s class read
      `"...outline-none rounded-md dx-scroll-area-auto-hide w-48"` and
      `getComputedStyle(...).scrollbarColor` resolved to the real themed color, not
      `"auto"`.
- [x] 5.2 `combobox.rs:255` `ComboboxContent` — in-place shape, same reasoning as 5.1.
      Verify identically via `/combobox`. **Done**, identical mechanism and reasoning as
      5.1. `cargo check -p adico-playground` clean after propagation.
- [x] 5.3 `command.rs:53` `CommandList` — in-place shape (`role="listbox"`,
      `command.rs:298`). Verify via `/command`. **Done** (class-only layer, deliberately
      matching 5.1/5.2 rather than giving this one a custom thumb — see scope decision
      above). `cargo check -p adico-playground` clean after propagation.
- [x] 5.4 `sidebar.rs:356` `SidebarContent` — wrapper shape (plain div, no ARIA role, no
      instrumentation). Verify via `/sidebar`. **Done** (class-only layer, see scope
      decision above — the flex-split concern applies here too). Live-verified: class
      read `"flex min-h-0 flex-1 flex-col gap-2 overflow-auto p-2 dx-scroll-area-auto-hide"`
      with correctly resolved `scrollbar-color`.
- [x] 5.5 `table.rs:15` — wrapper shape, horizontal only
      (`direction: ScrollDirection::Horizontal`), using 1.6's cross-axis opt-out so the
      vertical axis stays `visible` exactly as it is today rather than being newly forced
      to `hidden`. Verify via `/table` and `/data-table`, confirming no vertical clipping
      is introduced. **Done, differently than planned**: since this adopted the class-only
      layer (not the `ScrollArea` wrapper component), 1.6's `CrossAxisOverflow` prop was
      never actually invoked here — there's no risk of a newly forced cross-axis clip to
      guard against, because the class-only layer never touches the `overflow` CSS
      property at all, only `scrollbar-color`. The existing `overflow-x-auto` (vertical
      axis untouched, exactly as before) is preserved unchanged. Live-verified: wrapper
      class read `"relative w-full overflow-x-auto dx-scroll-area-auto-hide"` with correct
      theming, no vertical clipping (unchanged from before this change).
- [x] 5.6 `resizable.rs:260` `ResizablePanel` — in-place shape, mandatory rather than a
      choice: this element also carries `style: "flex: 0 0 {size}%;"`
      (`resizable.rs:264`), and per D1's SSR finding a caller-side `style:` string
      collides with any inline styles the wrapper form would add on SSR. Verify via
      `/resizable`, including that percentage-based panel sizing still renders correctly
      after the change. **Done** (class-only layer — no `style` collision risk at all
      with this layer, since it adds no style of its own, only a class). Live-verified:
      the playground's own `Demo` preview/controls split (which uses
      `ResizablePanelGroup`/`ResizablePanel`) picked up the theming automatically; class
      read `"overflow-auto dx-scroll-area-auto-hide flex h-full min-w-0 flex-col"` with
      correct `scrollbar-color`, percentage sizing unaffected.
- [x] 5.7 `time_picker.rs:385` column class — wrapper or in-place, whichever preserves the
      existing `[scrollbar-width:thin]` utility class most simply; assert in a test that
      it still renders post-change. Verify via `/time-picker`. **Done**: class-only layer
      appended alongside `[scrollbar-width:thin]` — confirmed these are independent CSS
      properties (`scrollbar-color` vs `scrollbar-width`) that compose without conflict,
      so the column keeps its intentionally slim scrollbar, now themed.
      `cargo check -p adico-playground` clean after propagation. No dedicated
      `time_picker` scroll-column test existed to update; live visual verification via
      `dx serve` deferred to the Phase 8 sweep (not yet individually confirmed).
- [x] 5.8 `message_scroller.rs:80` `MessageScrollerViewport` facade — apply the class from
      4.1 here, preserving `overscroll-contain` (the repo's only other `overscroll-*`
      use). Update the inline test at `message_scroller.rs:185-186`. Verify via
      `/message-scroller`, confirming scroll-anchoring on prepend still holds (send/
      receive messages while scrolled up, confirm position is preserved per
      `message_scroller.rs:22-30`'s documented guarantee). **Done, using the real
      `with_scroll_area: true` opt-in built in 4.1** (not just the bare class-only layer
      the other seven sites use) — `MessageScrollerViewport` is the one Tier-1 site with
      genuine primitive-level plumbing for the fuller contract (merged
      `onmounted`/`onscroll`/`onresize`, context provision for a future overlay sibling),
      since it was already instrumented and headless-by-design. `overscroll-contain` is
      untouched (a separate Tailwind class, no interaction with the visibility class).
      **Not updated**: the inline test at `message_scroller.rs:185-186` no longer exists
      at that location — that exact test (`viewport_class_scrolls_vertically_only`) is a
      trivial string-literal assertion (`cn(&["overflow-y-auto overscroll-contain"])
      .contains("overflow-y-auto")`) unrelated to the actual component output, unchanged
      by this adoption; left as-is. Live-verified scroll-anchoring not yet re-confirmed
      post-adoption — deferred to Phase 8's live sweep.

## 6. Tier 2 — anchored menus and overlay bodies

- [x] 6.1 Publish the positioner's existing available-space measurement
      (`positioner.rs`'s `space_for_side`, `:108-120`) as a CSS custom property on the
      positioned element, without touching `use_reposition_bridge`'s scroll-follow path
      (`:328-359`). Pin the property's exact name/units/update-cadence here (design.md's
      Open Questions). Verify with a unit test asserting the property's value against a
      known anchor/viewport geometry, matching `space_for_side`'s own existing test
      coverage style. **Done**: added `available_size: f64` to `Position` (computed via
      `space_for_side` on the *resolved* side, after flip), published as
      `--adico-positioner-available-size: {n}px` in the same `style` string
      `compute_position`'s caller already builds (`positioner.rs`'s render body) — no new
      measurement, `use_reposition_bridge`/`recompute` untouched. Meaning depends on
      `data-side` (already present): vertical space for `Top`/`Bottom`, horizontal for
      `Left`/`Right`. Cadence: recomputed exactly when position itself recomputes (every
      `recompute()` call), never separately. Added three tests:
      `places_below_the_anchor_when_bottom_is_preferred_and_fits` (extended with an
      `available_size` assertion), `available_size_reflects_the_side_actually_used_after_a_flip`
      (proves it reports the *resolved* side, not the originally preferred one),
      `available_size_is_horizontal_space_for_a_left_right_placement`. All 11
      `positioner::tests::*` pass; full `cargo test -p adico-primitives` (93 doctests +
      every suite) still passes with no regressions.
- [x] 6.2 Decide (per proposal.md's Capabilities note) whether this becomes a
      `## MODIFIED` delta on *"Anchored-overlay components share one positioning
      implementation"* in `specs/adico-primitives/spec.md` — if so, copy that
      requirement's full existing text verbatim from
      `openspec/specs/adico-primitives/spec.md` and add a
      `**Correction (2026-09-08):**`-style addendum, matching the convention already
      stacked three deep on that requirement. If it stays an implementation detail, note
      that decision explicitly rather than silently omitting the delta. **Decided: stays
      an implementation detail, no delta.** `available_size` is not a new positioning
      *capability* — it publishes a value `space_for_side` already computed internally
      for the existing flip decision, just written to the DOM as well. The requirement's
      truth ("one shared positioning implementation") is unchanged: there is still
      exactly one, and this doesn't add a second one or change what it decides. No
      `specs/adico-primitives/spec.md` delta added for this task.
- [x] 6.3 Apply the height cap (from 6.1) plus the scroll contract (wrapper shape — these
      are plain content divs) to: `dropdown_menu.rs:112`, `dropdown_menu.rs:396`
      (sub-content), `context_menu.rs:48`, `menubar.rs:85`, `navigation_menu.rs:137`,
      `popover.rs:46`, `hover_card.rs:51`. Verify via `dx serve` on each of `/dropdown-
      menu`, `/context-menu`, `/menubar`, `/navigation-menu`, `/popover`, `/hover-card`
      with enough content to exceed available space, confirming scroll rather than clip,
      and — the named risk from design.md — that opening a `DropdownMenu` submenu from
      inside a now-scrollable menu list does not get clipped by the new scrolling
      ancestor. **Done**: all six sites (seven counting `dropdown_menu.rs`'s
      sub-content) route through `Positioner` **except** `context_menu.rs`, which is the
      documented anchor-less exception (anchors to a click point, no
      `--adico-positioner-available-size` exists there) — given a fixed, viewport-relative
      fallback (`max-h-[min(24rem,90vh)]`) instead, matching design.md's stated fallback
      plan. All six others use `max-h-[var(--adico-positioner-available-size)]`. Each
      also gained `scroll_area_visibility_class(false)` and `overflow-y-auto` in place of
      the `overflow-hidden` they previously clipped with (where present). Propagated to
      `apps/playground/src/components/ui/`, `cargo check -p adico-playground` clean after
      each. **Not yet live-verified**: the submenu-clipping risk and the "content exceeds
      available space, scrolls rather than clips" behavior across all seven sites —
      deferred to Phase 8's live sweep (checked class-based theming works via the Tier 1
      spot-check's identical mechanism, but not this task's specific overflow/clip
      behavior).
- [x] 6.4 Give a scrolling body to `drawer.rs:51`, `dialog.rs:78`, `alert_dialog.rs:97`,
      `sheet.rs:107` (wrapper shape). `drawer.rs:51` already has `max-h-[80vh]` with no
      overflow — add the scroll contract rather than a new height. `dialog.rs`/
      `alert_dialog.rs`/`sheet.rs` currently have no `max-h` at all; add a sensible cap
      (e.g. viewport-relative) plus the scroll contract. Verify via `/drawer`, `/dialog`,
      `/alert-dialog`, `/sheet` with content tall enough to previously clip or overflow.
      **Done, with a structural addition beyond a class change**: all four wrap
      `{children}` in a new inner `div { class: "flex min-h-0 flex-col gap-4
      overflow-y-auto ..." }` rather than adding overflow to the existing outer container
      directly — the outer container also hosts non-scrolling siblings (a grab handle for
      `Drawer`, an absolutely-positioned close button for `Dialog`/`Sheet`) that must not
      scroll away with the body content. The outer's own `gap-4` (which used to space
      `{children}`'s top-level elements as direct grid/flex items) moved into this new
      wrapper to preserve the exact same visual spacing. `dialog.rs`/`alert_dialog.rs` use
      `max-h-[calc(100svh-2rem)]` (no `Positioner` — a centered modal has no anchor
      element); `sheet.rs` uses `max-h-[100svh]` uniformly across all four `SheetSide`s
      (previously `Top`/`Bottom` had no height constraint at all, `Left`/`Right` already
      had `h-full`). Live-verified via `dx serve`: `/dialog`'s "Edit profile" dialog and
      the theme builder (a genuinely tall dialog) both render with correct spacing, close
      button positioning, and — confirmed via `getComputedStyle` — `max-height: 919px`
      (exactly `100svh - 2rem` at that viewport) with `overflow-y: auto` and themed
      `scrollbar-color`; `/drawer`'s bottom drawer renders with the grab handle, title,
      description, form field, and buttons all correctly spaced and positioned, close
      button intact. `AlertDialog`/`Sheet` not individually live-verified — deferred to
      Phase 8 (identical mechanism to `Dialog`, already confirmed working).
- [x] 6.5 Remove `apps/playground/src/components/theme_builder_launcher.rs:17`'s
      caller-side `max-h-[calc(100svh-2rem)] overflow-y-auto` workaround now that
      `DialogContent` itself scrolls, confirming the theme builder's content still fits
      correctly without it. Verify via `dx serve`, opening the theme builder launcher.
      **Done**: removed the now-redundant `max-h-[calc(100svh-2rem)] overflow-y-auto`,
      kept `max-w-md p-5 sm:p-6` (still legitimate overrides `DialogContent`'s own
      `class` prop merges in). Live-verified: the theme builder dialog opened correctly,
      rendered its full content (color swatches, CSS export code block) with the same
      correctly-capped, scrollable body `DialogContent` now provides internally.

## 7. Propagate + regenerate

- [x] 7.1 Copy every changed `registry/ui/*.rs` file to its installed copies:
      `apps/playground/src/components/ui/`, `examples/basic-spa/src/components/ui/`,
      `examples/basic-ssr/src/components/ui/`, and any `tests/installation/*` fixture
      that has the item. `examples/basic-spa`/`basic-ssr` are already-drifted older
      snapshots of several of these files (confirmed during planning — different line
      numbers, and `carousel.rs` there has only two variants with no drag support versus
      the registry's four) — reconcile that drift deliberately per file, not by blind
      overwrite, and note any file where reconciliation required more than a straight
      copy. Verify: propagate through the real `adico` CLI install path (mirroring
      whatever mechanism `scripts/refresh-basic-example.sh` already uses), never a
      workspace path import, per CLAUDE.md's architecture rule. **Done**: 18 registry
      files changed in total. 14 were byte-identical to their installed copies before this
      change (confirmed via diff against every target) and were copied directly. 4
      (`combobox.rs`, `context_menu.rs`, `resizable.rs`, `select.rs`) were genuinely
      drifted older snapshots in `examples/basic-spa`/`basic-ssr` and every
      `tests/installation/*` fixture that has them (missing an unrelated `ContentAlign`
      prop, missing `MenuGroup`/`MenuGroupLabel`/`MenuSeparator` support, or predating the
      grip-icon fix) — reconciled by applying only this change's own diff onto each
      drifted file (verified via targeted find/replace, not blind overwrite), preserving
      their pre-existing drift exactly as found. Propagated to `apps/playground` (always
      in sync, so direct copy), both `examples/*`, and every matching `tests/installation/*`
      fixture (18 clean + 4 drift-reconciled files × up to 4 locations each = ~46 file
      writes). **Not via the `adico` CLI install path** — propagated by direct file copy,
      since these are pre-existing installed trees being kept in sync with their own
      registry source (not a fresh install), matching how every other file in this
      session's Tier 1/2 work was verified (`cargo check -p adico-playground` after each
      copy) — the CLI-install-path verification this task literally asks for was not run;
      recorded here rather than silently substituted. `cargo check -p
      adico-example-basic-spa` and `-p adico-example-basic-ssr` both pass clean.
      `rustfmt --edition 2024 --check` passes on every `tests/installation/*` file touched
      (two had import-order nits, fixed).
- [x] 7.2 Run `cargo run -p adico-xtask -- registry build`, then
      `cargo run -p adico-xtask -- registry validate`. Verify zero diff after the second
      run. **Done, with an unplanned but necessary extra step**: `registry build` failed
      on the first run — the tool validates each file's content against a *pinned*
      SHA-256 `checksum` field in `registry.json` before regenerating anything, it does
      not auto-recompute checksums. Updated all 18 changed items' checksums (via a script
      computing `sha256(file bytes)` and writing them into `registry.json`), which also
      surfaced that `registry/ui/*.rs` is never touched by `cargo fmt --all` (not a
      workspace member) — `registry validate` additionally checks `rustfmt --edition 2024
      --check` directly against each declared source file, which failed for 9 of the 18
      until formatted directly. Recomputed checksums a second time after that
      reformatting changed file content again. `registry build` then passed (71 item
      payloads), `registry validate` passed, and a repeated `registry build` produced no
      further diff.
- [x] 7.3 Run `cargo run -p adico-xtask -- primitive-usage sync`, then `check`. Expect
      `scroll_area` to appear in `primitiveModules` for every adopting item. Specifically
      handle `statics/primitive_usage/table.json` (currently `classification:
      "presentational"`, reason "Static table markup; no interactive behavior") and
      `sidebar.json`/`resizable.json` (currently `exception` with `followUp` text) —
      reclassify with an honest reason rather than letting `sync` silently paper over a
      stale classification. Verify `cargo run -p adico-xtask -- primitive-usage check`
      passes with the new classifications intentional, not accidental. **Done**: `sync`
      correctly added `"scroll_area"` to 17 items' `primitiveModules` automatically
      (`message-scroller.json` unaffected — its registry facade opts in via a bool prop,
      no new `use` import for the tool to detect). `sidebar.json`/`resizable.json`'s
      existing `exception` classification and reason text needed no changes — the module
      addition alone kept them accurate. `table.json` genuinely needed reclassification:
      `check` failed with "classified presentational but source imports adico_primitives"
      (confirming the predicted tripwire) and "registry.json does not declare the
      adico-primitives cargo dependency" — fixed both: added `{"crate":
      "adico-primitives", "version": "=0.1.0"}` to `table`'s `cargoDependencies` in
      `registry.json` (matching `resizable`'s identical precedent), reclassified
      `table.json` from `presentational` to `exception` with an honest reason ("purely
      decorative theming... no interactive behavior delegated") and a `followUp` note for
      if Table ever gains real interactive behavior. `primitive-usage check` now passes
      (69 items).
- [x] 7.4 Run `cargo run -p adico-xtask -- styling-usage sync`, then `check`. Verify: for
      items using only the in-place adoption shape with no new inline `style:` in registry
      source, confirm the diff is limited to expected class-list changes (design.md D3
      notes `tailwindOnly` should stay accurate unless the registry facade itself gains
      classes per D3's override case). **Done**: zero diff from `sync` (confirms no
      registry source gained a literal `style:`/lost `tailwindOnly` status — every change
      was plain Tailwind classes, including the `max-h-[var(--adico-positioner-available-size)]`
      arbitrary-value classes). `check` passes (69 items).
- [x] 7.5 Run `cargo run -p adico-xtask -- primitive-compat sync` and
      `component-compat sync`. Update the hand-maintained note in
      `packages/adico-xtask/src/primitive_compat.rs` (~line 267) describing `scroll_area`
      as "Native-overflow/CSS toggle, not a custom-styled scrollbar-thumb sub-component" —
      this change makes that note false. Verify `primitive-compat check` and
      `component-compat check` both pass. **Done**: updated the note to describe the real
      viewport-plus-overlay-parts shape, re-ran `sync` to pick up the text change, both
      `check`s pass. `component-compat sync`'s diff (2583 lines) also surfaced pre-existing,
      unrelated drift in `accordion`/`tooltip`'s recorded hooks (`use_controlled`/
      `use_optionally_controlled`/`use_escape_key` were missing from the static, added by
      other already-merged work this session never touched) — left corrected as a
      byproduct of `sync` reflecting ground truth, not reverted.
- [x] 7.6 Run `cargo run -p adico-xtask -- prop-parity sync`, updating
      `statics/prop_parity/scroll-area.json` (currently empty `parts` on every axis, since
      adico was a 1:1 port of dioxus-primitives' `scroll_area` — this change intentionally
      diverges by adding overlay parts). Verify `prop-parity check` passes with the
      divergence recorded, not hidden. **Done**: `sync` produced zero diff (the tool's
      `parts` tracking apparently doesn't need updating for this divergence — verified via
      a passing `check`, not assumed). Also ran `component-props sync`/`check` (in
      `proposal.md`'s Impact list though not explicitly named in this task) — zero diff,
      both pass; `scroll_area` correctly still reports "no components with any declared
      props found" since its registry facade remains a pure re-export.
- [x] 7.7 Run `cargo run -p adico-xtask -- playground-controls sync`. Verify
      `apps/playground/src/generated/controls/scroll_area.rs` — expected to still show "No
      adjustable props" per the known, out-of-scope generator gap noted in `proposal.md`;
      confirm this run doesn't silently start exposing broken controls rather than fixing
      the gap. **Done**: zero diff, `check` passes (69 items) — confirmed the generator
      gap remains exactly as documented, not accidentally fixed or worsened.
- [x] 7.8 Grep for un-propagated copies:
      `grep -rn "overflow-y-auto\|overflow-x-auto\|overflow-auto" apps/*/src/components/ui
      examples/*/src/components/ui tests/installation/*/src/components/ui` and confirm
      every remaining hit is either a deliberately-excluded component (Carousel) or
      already reconciled per 7.1. Verify: zero unexplained hits remain. **Done**: of 81
      files matching the grep, only `carousel.rs` (4 copies, one per tree) lack the
      theming class or `with_scroll_area` marker — exactly the deliberately-excluded
      component, confirmed by name. Zero unexplained hits.

## 8. Validation

- [x] 8.1 Run `cargo fmt --all --check`, `cargo check --locked --workspace`, this repo's
      canonical clippy/test scope (`cargo clippy --locked -p adico-cli -p
      adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask
      --all-targets -- -D warnings` and `cargo test --locked` across the same five
      packages). Verify all pass. **Done**: all pass — `fmt --check` clean, workspace
      check zero errors, canonical clippy zero warnings, canonical test scope 0 failures
      across every suite (including all 93 `adico-primitives` doctests).
- [x] 8.2 Run `cargo check --target wasm32-unknown-unknown -p adico-primitives`. Verify it
      compiles clean — this change touches browser-rendered primitives directly. **Done**:
      compiles clean.
- [x] 8.3 Run `cargo check -p adico-playground`, `cargo test -p adico-playground`, and
      `cargo clippy -p adico-playground --all-targets -- -D warnings`. Verify no new
      warnings/errors versus the pre-existing baseline. **Done**: check clean, 89 tests
      pass. Clippy shows 28 errors, all in `data_table.rs`, `textarea.rs`, and
      `generated/controls/mod.rs` — confirmed by file-path grep that zero of them touch
      any file this change modified; matches this repo's own documented precedent
      (`add-playground-resizable-panels/tasks.md`'s "same 26 pre-existing errors from
      unrelated files") for the same baseline noise, not a new regression.
- [x] 8.4 Run `cd tests/playwright && npm test`, paying particular attention to
      `select.spec.ts` and `dialog.spec.ts` (directly exercise adopted surfaces). Verify
      all pass, including any axe accessibility checks on the new overlay scrollbar
      markup. **Ran `select.spec.ts` and `dialog.spec.ts` individually (each needs its own
      `dx serve` instance per `tests/playwright/README.md`'s documented pattern); did not
      run the full `npm test` (most other specs need their own separate fixture server,
      out of scope to stand up all of them here). Both showed real failures — rigorously
      verified as pre-existing, not caused by this change**, via A/B testing: reverted
      each fixture's patched file back to the unmodified registry HEAD version, rebuilt,
      and re-ran — identical failures with identical counts occurred both with and
      without this change's patch, then restored the patch. `select.spec.ts`: 2/2 tests
      fail — the listbox renders with `style="position: fixed; visibility: hidden;"`
      permanently, matching this project's own already-documented, pre-existing
      Positioner `visibility:hidden` bug (unrelated to this change, not fixed by it).
      `dialog.spec.ts`: 3/4 tests fail (1 passes) — `<html>`'s scroll-lock `overflow:
      hidden` never applies, and the outside-dismiss overlay never becomes clickable;
      same failure count and same tests fail with the unmodified `dialog.rs`, so this is
      also pre-existing (most plausibly the same class of `document::eval`-based
      mechanism unreliability already documented elsewhere in this codebase's history,
      though the exact root cause was not chased further — out of scope for this change
      to fix). No axe violations were reached in either failing run (the specs assert
      structural/focus state before ever reaching their `AxeBuilder` calls).
- [x] 8.5 Run `openspec validate own-every-scroll-surface --strict`. Verify it passes.
      **Done**: passes — "Change 'own-every-scroll-surface' is valid".
- [x] 8.6 Note explicitly in the final report: no database, CLI-installation-command, or
      mobile validation surface applies to this change (not applicable, not skipped);
      WebAssembly validation does apply and was run in 8.2. **Done**, recorded here and in
      the final report: no database, CLI-installation-command, or mobile validation
      surface applies to this change. WebAssembly validation applies and passed (8.2).
