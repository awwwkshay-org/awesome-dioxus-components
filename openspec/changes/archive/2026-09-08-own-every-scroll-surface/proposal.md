## Why

`ScrollArea` (`packages/adico-primitives/src/scroll_area.rs`) exists, and commit
`4fa3031` gave it a themed, contrast-correct scrollbar (`scrollbar-color` derived from
`--foreground`/`--background`, keyed to the `dx-scroll-area-auto-hide`/
`dx-scroll-area-always-show` classes it emits). But no `registry/ui/*.rs` component
composes it — the only call sites are three demo pages. Every real scroll container in
the ecosystem (`Select`'s option list, `Command`'s list, `Sidebar`'s content region,
`Table`'s horizontal scroller, `Resizable`'s panels, `TimePicker`'s columns,
`MessageScroller`'s viewport) is a bare `div` with a Tailwind `overflow-*` class, so it
renders with the browser's unthemed default scrollbar instead. Consumers installing these
components get inconsistent scrollbar styling depending on which component they use, with
no way to fix it short of hand-patching registry source — which this ecosystem's own rule
says should never happen for app convenience.

Investigating adoption surfaced that `ScrollArea` itself is not yet adoptable as-is:

- `ScrollType::Hidden` writes `scrollbar-width` as a quoted HTML attribute
  (`scroll_area.rs:127`), not a CSS declaration — `dioxus-html-0.7.9` declares no such
  style attribute, so it renders inert. The existing test encodes this bug rather than
  catching it (`test_scroll_area.rs:67` asserts the attribute form).
- `always_show_scrollbars` only swaps a class name; both classes carry identical CSS
  today, so the prop has no visible effect.
- `ScrollArea` has no `class` prop. Verified against the vendored `dioxus-rsx`/
  `dioxus-ssr`/`dioxus-interpreter-js` source: passing a class through the attribute
  spread collides with the internal visibility class, and the collision resolves
  **oppositely** on SSR (keeps the internal class, drops the caller's) versus CSR (keeps
  the caller's, drops the internal one) — meaning naive adoption would silently render
  differently on first paint versus after hydration.

Separately, keyboard navigation in `Select`/`Combobox`/`Command`/`DropdownMenu`/
`Menubar`/`ContextMenu` has never scrolled the active item into view inside its own
`max-h-72`/`max-h-[300px]` container — `scrollIntoView` does not exist anywhere in this
repository. And a bounded set of anchored/overlay surfaces (`DropdownMenu`, `ContextMenu`,
`Menubar`, `NavigationMenu`, `Popover`, `HoverCard`, `Drawer`, `Dialog`, `AlertDialog`,
`Sheet`) have no scroll escape at all today — content taller than the surface simply
clips or overflows the viewport, which is why the playground's own theme-builder launcher
has to work around `DialogContent` caller-side (`theme_builder_launcher.rs:17`).

## What Changes

- `ScrollArea` gains a merging `class` prop, a real (style-namespaced) `scrollbar-width`
  declaration for `ScrollType::Hidden`, and distinct CSS for `always_show_scrollbars` —
  closing the three defects above before anything adopts it.
- `ScrollArea` gains overlay scrollbar parts (`ScrollAreaViewport`, `ScrollAreaScrollbar`,
  `ScrollAreaThumb`, `ScrollAreaCorner`) driven by the viewport's own `scrollTop`/
  `scrollLeft`, never by `transform` on an ancestor. `ScrollArea` itself keeps bundling
  viewport + scrollbar as a one-line wrapper for simple containers; the parts are also
  exported for composition, and for the handful of call sites that must apply the same
  styling contract *to an element the component already renders* rather than wrapping it
  in a new one (see Design: two adoption shapes).
- A shared scroll-into-view behavior is added and wired into `Select`, `Combobox`,
  `Command`, and the shared `menu`/`listbox`/`typeahead` primitives that back
  `DropdownMenu`/`Menubar`/`ContextMenu`, so keyboard navigation keeps the active item
  visible.
- Eight `registry/ui` components adopt the shared scroll contract in place of their own
  `overflow-*` Tailwind class: `Select`, `Combobox`, `Command`, `Sidebar`, `Table`,
  `Resizable`, `TimePicker`, `MessageScroller`. **Implementation note**: all eight adopt
  the contract's class-only layer (themed `scrollbar-color`/`scrollbar-gutter` via the
  shared visibility class) rather than the full custom overlay thumb — `Select`/
  `Combobox` route through `Positioner`, which has no `onscroll`/`onresize` passthrough
  without changes to a primitive shared by 10+ anchored components; `Sidebar`/`Resizable`
  are simultaneously flex children and flex containers, making a wrapper split costly for
  uncertain benefit; `Command` was kept at the same layer deliberately, for UX consistency
  with `Select`/`Combobox` as a set. See `design.md`'s D2 correction for the full
  reasoning. The overlay thumb itself (built and live-verified working, including
  drag-to-scroll) remains available via `ScrollArea`/`ScrollAreaViewport` directly.
  **Carousel is explicitly excluded** — its
  tracks combine `scroll-snap-type` with a pointer-drag gesture, and `carousel.rs:163-165`
  already documents that snapping fights programmatic scroll; converting it risks
  regressing both for a cosmetic gain.
- A bounded set of anchored menus (`DropdownMenu`, `ContextMenu`, `Menubar`,
  `NavigationMenu`, `Popover`, `HoverCard`) gain an available-height cap — published from
  the positioner's own existing collision-aware measurement (`positioner.rs`'s
  `space_for_side`, already computed for its flip logic) as a CSS custom property, not a
  hardcoded viewport-percentage guess — plus the scroll contract, so they scroll instead
  of clipping. `Drawer`, `Dialog`, `AlertDialog`, and `Sheet` gain a scrolling body for the
  same reason.
- All changed `registry/ui/*.rs` source is propagated to its installed copies
  (`apps/playground`, `examples/basic-spa`, `examples/basic-ssr`, relevant
  `tests/installation/*` fixtures) with checksums bumped, and every CI-gated generated
  record (`registry/generated/items/*.json`, `statics/primitive_usage/*.json`,
  `statics/styling_usage/*.json`, `statics/prop_parity/scroll-area.json`,
  `statics/primitive_compatibility.json`, `statics/component_compatibility.json`) is
  resynced to match.
- **BREAKING for `scroll-area`'s registry classification only**: `scroll-area` moves from
  a pure re-export (`semanticTokens: false`, `radiusToken: false`, `utilities: []`,
  `tailwindOnly: true`) to a component with real styled parts, once `design.md`'s thumb-
  styling decision lands. No public prop of `ScrollArea` itself is removed or renamed —
  the new parts and the `class` prop are additive.

## Capabilities

### New Capabilities

(none — this change adds requirements to existing capabilities)

### Modified Capabilities

- `adico-primitives`:
  - **ADDED** requirement: the scroll-area primitive exposes a reusable, mergeable
    scrollbar-styling contract independent of which element it's applied to, with real
    overlay scrollbar parts.
  - **ADDED** requirement: `ScrollArea`'s visibility props (`ScrollType::Hidden`,
    `always_show_scrollbars`) have observable, distinct effect.
  - **ADDED** requirement: keyboard navigation scrolls the active item into view inside a
    scrollable listbox/menu.
  - Possible **MODIFIED**: "Anchored-overlay components share one positioning
    implementation" (`openspec/specs/adico-primitives/spec.md`), if publishing available
    space as a CSS custom property becomes part of that requirement's contract rather than
    staying an implementation detail. This change's own spec deltas do not decide it —
    the decision is made once the property's exact shape is implemented, per `tasks.md`
    task 6.2; if modified, the delta carries the full existing requirement text plus a new
    `**Correction (2026-09-08):**`-style addendum, per this repo's convention on that same
    requirement.
- `adico-existing-components`:
  - **ADDED** requirement: every registry component with a scrollable region composes the
    shared scroll contract, naming the eight adopting components and recording carousel
    as a stated, reasoned exception.
  - **ADDED** requirement: anchored menu surfaces cap their height (from the positioner's
    available-space measurement) and scroll rather than clipping.
  - **ADDED** requirement: modal and drawer surfaces scroll their body rather than
    clipping or silently overflowing the viewport.
  - Decided (not left open): the existing TimePicker requirement (`spec.md:210`, whose
    `:227` scenario already names "the scrollable columns") and the existing Carousel
    requirement (`spec.md:403`, whose pointer-drag scenario is unaffected — Carousel is
    explicitly excluded from this change) both remain accurate as written and stand
    alone; this change's new requirement adds to the picture rather than altering either,
    so neither takes a `## MODIFIED` delta.
- `adico-registry`: no requirement change is proposed, and none is needed. The
  reclassification work in Impact/`tasks.md` task 7.3 (e.g. `table`'s `presentational`
  classification, `sidebar`/`resizable`'s `exception` records) is exactly the sanctioned
  path `openspec/specs/adico-registry/spec.md:143-150`'s "A presentational item gains
  interactive behavior" scenario describes: the verification command fails only when a
  record goes **stale** relative to source, not when a record is correctly updated to
  match a real classification change. Updating each record alongside its component change
  keeps every one current, so this requirement is satisfied by the work, not modified by
  it.

## Impact

- `packages/adico-primitives/src/scroll_area.rs`,
  `packages/adico-primitives/tests/test_scroll_area.rs` — primitive repair + new overlay
  parts + scroll-into-view.
- `packages/adico-primitives/src/listbox.rs`, `select.rs`, `combobox.rs`, `command.rs`,
  `menu.rs`, `typeahead.rs` — scroll-into-view wiring.
- `registry/ui/select.rs`, `combobox.rs`, `command.rs`, `sidebar.rs`, `table.rs`,
  `resizable.rs`, `time_picker.rs`, `message_scroller.rs` — Tier 1 adoption.
- `registry/ui/dropdown_menu.rs`, `context_menu.rs`, `menubar.rs`, `navigation_menu.rs`,
  `popover.rs`, `hover_card.rs`, `drawer.rs`, `dialog.rs`, `alert_dialog.rs`, `sheet.rs` —
  Tier 2 height cap / scrolling body.
- `apps/playground/src/components/theme_builder_launcher.rs` — in scope as cleanup enabled
  by `DialogContent`'s new built-in scrolling: its caller-side
  `max-h-[calc(100svh-2rem)] overflow-y-auto` workaround is removed once no longer needed
  (`tasks.md` task 6.5). This is the one app-level file this change touches; every other
  app-level scroll container listed under Explicitly out of scope below remains untouched.
- `packages/adico-cli/src/css.rs` (`SCROLLBAR_CSS`), and the three checked-in
  `tailwind.css` files under `apps/playground/`, `examples/basic-spa/`,
  `examples/basic-ssr/` — scrollbar theming, scoped per `design.md`'s thumb-styling
  decision.
- Installed copies under `apps/playground/src/components/ui/`,
  `examples/basic-spa/src/components/ui/`, `examples/basic-ssr/src/components/ui/`, and
  affected `tests/installation/*` fixtures — propagated, not hand-diverged.
  `examples/basic-spa`/`basic-ssr` are already-drifted older snapshots of several of these
  files (confirmed: different line numbers, and `carousel.rs` there has two variants with
  no drag support versus the registry's four); propagation must reconcile that drift
  deliberately, not blind-copy over it.
- Generated/CI-gated: `registry/registry.json`, `registry/generated/items/*.json`,
  `statics/primitive_usage/*.json`, `statics/styling_usage/*.json`,
  `statics/prop_parity/scroll-area.json`, `statics/primitive_compatibility.json`,
  `statics/component_compatibility.json`, `statics/component_props.json`.
- No database, CLI-installation-command, or mobile validation surface applies. WebAssembly
  (`wasm32-unknown-unknown`) validation does apply, since this change touches browser-
  rendered primitives.
- Explicitly out of scope: Carousel adoption, `Tabs` overflow-scrolling, `Calendar`'s
  fixed layout, `Toast`'s `max-h-screen` viewport, all app-level scroll containers outside
  `registry/ui` (playground's own `routes.rs`/`demo.rs`/`pages/sidebar.rs`, `apps/docs`),
  and the pre-existing playground-controls generator gap where `ScrollArea`'s
  `ReadSignal`-typed props render no demo controls
  (`packages/adico-xtask/src/playground_controls.rs`'s `classify_prop_type`) — recorded
  here as a known follow-up, not fixed by this change.
