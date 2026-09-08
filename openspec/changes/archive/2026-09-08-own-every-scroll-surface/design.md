## Context

`ScrollArea` (`packages/adico-primitives/src/scroll_area.rs`) is a single `div` that
computes `overflow-x`/`overflow-y`/`scrollbar-width` from an exhaustive 3x3
`ScrollType` x `ScrollDirection` match, and emits `dx-scroll-area-auto-hide` or
`dx-scroll-area-always-show`. `packages/adico-cli/src/css.rs`'s `SCROLLBAR_CSS` (added in
commit `4fa3031`) themes exactly those two classes via `scrollbar-color`, mirrored into
`apps/playground/tailwind.css`, `examples/basic-spa/tailwind.css`, and
`examples/basic-ssr/tailwind.css`. Nothing in `registry/ui/` composes it today; every real
scroll container there is a bare `overflow-*` div, so it gets the browser's unthemed
default scrollbar. See `proposal.md` for the full defect list this design starts from —
this document does not restate it.

`packages/adico-primitives/src/positioner.rs` renders every anchored surface
(`Tooltip`, `Popover`, `HoverCard`, `DropdownMenu`, `Select`, `Combobox`, `Menubar`,
`NavigationMenu`, `ContextMenu`, `DatePicker`, `TimePicker`, `ThemeSwitcher`) as
`position: fixed` in viewport-space coordinates, glued to its anchor via a capture-phase
`document.addEventListener('scroll', notify, true)` listener (`positioner.rs:342`) and a
one-shot collision-aware placement pass (`positioner.rs:3`, `space_for_side` at
`:108-120`, flip logic at `:137-143`). `apps/playground/src/components/demo.rs:21-40`
records two live-verified failure modes any new scroll wrapper must not reintroduce: a
`transform`ed ancestor breaks `position: fixed` anchoring outright, and a non-transformed
`overflow: hidden|auto` ancestor can still clip a `position: fixed` popover in some
browsers.

`message_scroller.rs:22-30` documents that prepend-anchoring in `MessageScroller` relies
entirely on native CSS scroll anchoring (`overflow-anchor: auto`) — a guarantee that only
holds if the scroll container is a real native-overflow element, not a virtualized or
transform-driven one.

## Goals / Non-Goals

**Goals:**
- One owned, themed scroll implementation reaching every real scroll container in
  `packages/adico-primitives` and `registry/ui`, with overlay scrollbars.
- `ScrollArea`'s own defects (dead `Hidden` type, no-op `always_show_scrollbars`, no
  mergeable `class`) fixed before anything depends on them.
- Keyboard navigation that follows the active item into view inside a scrollable
  listbox/menu, since this repo currently has no `scrollIntoView` equivalent anywhere.
- A bounded set of anchored/overlay surfaces that currently have no scroll escape
  (clip or silently overflow) gain one.

**Non-Goals:**
- Carousel adoption — excluded per `proposal.md`, snap/drag risk outweighs the cosmetic
  gain.
- `Tabs` overflow-scrolling, `Calendar`'s fixed layout, `Toast`'s stacking — each needs a
  layout redesign this change does not attempt.
- App-level scroll containers outside `registry/ui` (playground's own shell, `apps/docs`).
- Any change to `Positioner`'s scroll-follow (`use_reposition_bridge`) mechanism itself —
  this change only *reads* its existing available-space measurement, never touches how or
  when it recomputes.
- Fixing the playground-controls generator gap that hides `ScrollArea`'s `ReadSignal`-
  typed props from the demo UI — recorded as a known follow-up in `proposal.md`.

## Decisions

### D1: Repair `ScrollArea` before any adoption, not alongside it

The three defects (dead `Hidden`, no-op `always_show`, no mergeable `class`) are not
independent of adoption — they are load-bearing for it. Concretely: adoption means every
Tier 1/2 call site starts passing Tailwind classes to `ScrollArea`; without a merging
`class` prop, that class collides with the internal visibility class. Verified against the
vendored `dioxus-rsx`/`dioxus-ssr`/`dioxus-interpreter-js` 0.7.9 source (not assumed):
SSR's `dioxus-ssr` writes every dynamic attribute with no dedup, so two `class=` attributes
are emitted and the browser's HTML parser keeps the **first** one — the internal
`dx-scroll-area-*` class survives, the caller's Tailwind is dropped. CSR's
`setAttributeInner` does a plain `node.setAttribute`, so the **last** write wins — the
opposite outcome, caller's class survives, internal class is dropped. Hydration does not
reconcile this (`hydrate_node` re-registers listeners and `data-dioxus-id`, not
attributes), so an SSR app would render correctly on first paint and silently regress the
moment it hydrates, or vice versa on a CSR-only app. Shipping adoption before this fix
would make every site it touches non-deterministic across rendering modes.

Alternative considered: ship adoption with the class collision unresolved, on the theory
that most existing call sites use `overflow-*` classes that ScrollArea's own inline styles
already subsume. Rejected — several Tier 1 sites need non-overflow classes preserved too
(`overscroll-contain`, `[scrollbar-width:thin]`, `flex`/sizing classes), so the collision
is not avoidable by scoping which classes get passed.

### D1a: Single-axis wrapper adoption must not force the cross axis to `hidden`

`ScrollDirection::Vertical`/`Horizontal` currently force the *other* axis's `overflow` to
`hidden` (`scroll_area.rs`'s 3x3 match: `Vertical => (overflow-x: hidden, overflow-y:
auto)`, and the mirror for `Horizontal`). Every existing single-axis wrapper-form Tier 1
site leaves its cross axis at the browser default (`visible`) today: `table.rs:15`
(`overflow-x-auto` only). Forcing it to `hidden` is a real, previously-unaddressed
behavior change, not a cosmetic one — if an ancestor ever constrains that element's height
(now or in a future edit), content that used to overflow visibly would silently clip
instead. Phase 1 adds a way to opt out of the forced cross-axis clip so existing behavior
at single-axis wrapper sites is preserved by default rather than incidentally changed by
adoption; see `tasks.md` task 1.6. In-place-shape sites (D2) are unaffected — they keep
their own existing single-axis Tailwind class untouched and only add the visibility class
plus the overlay scrollbar sibling, never ScrollArea's computed inline `overflow`.

### D2: Two adoption shapes, both from the same primitive — not one wrapper for everything

There is no `as_child` mechanism anywhere in this codebase (verified: zero hits for
`as_child`/`AsChild`/`render_as`). Several Tier 1 sites are not "a div with overflow and
nothing else":

- `SelectList` is `role="listbox"` on the `Positioner` element itself
  (`select.rs:617-640`); `ComboboxContent` is the same shape; `CommandList` is
  `div { role: "listbox" }` (`command.rs:298`). Wrapping any of these in a new
  `<ScrollArea>` div would insert a generic node between the ARIA role and its required
  `role="option"` children.
- `MessageScrollerViewport` and the `VirtualList` container already read/write scroll
  position via `onmounted`/`onscroll` (`message_scroller.rs:181-207`). Forwarding those
  listeners through a wrapper component's `#[props(extends = GlobalAttributes)]` does not
  work — that attribute group does not cover event listeners on a component call, only on
  a plain HTML element (documented at `positioner.rs:416-419`, which is exactly why
  `Positioner` itself declares six explicit `on_*` props instead of relying on the
  spread). Wrapping would require re-deriving that instrumentation on the wrapper, adding
  a second, redundant scroll-position source of truth.

So `ScrollArea` exports two adoption shapes:

1. **Wrapper form** — `ScrollArea { ... }` bundles a viewport + overlay scrollbar as a
   one-line convenience. Used for `Sidebar`, `Table`, `Resizable`, `TimePicker`'s column,
   and the Tier 2 menu/dialog bodies, all of which are genuinely "a div with overflow."
2. **In-place form** — the same styling contract (merging class, plus an overlay
   scrollbar rendered as a sibling, driven by the existing element's own `scrollTop` via a
   shared hook rather than a new wrapper's) applied directly to `SelectList`,
   `ComboboxContent`, `CommandList`, `MessageScrollerViewport`, and the `VirtualList`
   container — the exact elements those components already render, unchanged in
   structural position.

Both shapes read from one shared implementation (the class helper plus a
scrollbar-position hook), so there is exactly one place the themed scrollbar's CSS keys
off, not two divergent ones.

Alternative considered: give `ScrollArea` a full Radix-style `as_child`/render-prop
escape hatch so every site can use the wrapper form uniformly. Rejected as
disproportionate — it would be new machinery adopted by nothing else in this codebase,
solely to avoid five call sites needing the in-place form, and would still require solving
the same listener-forwarding gap underneath.

**Correction, discovered during Tier 1 implementation: all eight `registry/ui` sites
adopt a third, lighter tier — class-only theming — not the wrapper or full in-place
shape described above.** Reading `positioner.rs`'s actual prop list
(`positioner.rs:391-454`) found it exposes `on_mounted` plus four other explicit
callbacks, but no `on_scroll`/`on_resize` — the in-place shape as designed above assumed
merging scroll-area's `onmounted`/`onscroll`/`onresize` directly onto the target element,
which is not possible for `SelectList`/`ComboboxContent` without first adding new props to
`Positioner`, a primitive shared by 10+ anchored components (`Tooltip`, `Popover`,
`HoverCard`, `DropdownMenu`, `Menubar`, `NavigationMenu`, `ContextMenu`, `DatePicker`,
`TimePicker`, `ThemeSwitcher`) — well beyond this change's intended surface.
`SidebarContent`/`ResizablePanel` hit a different obstacle: both are simultaneously a flex
child (sized by their own parent) and a flex container (laying out their own children),
so splitting them into a wrapper-plus-viewport would mean dividing `flex min-h-0 flex-1
flex-col gap-2` across two elements for uncertain benefit.

What actually shipped for all eight sites: just `scroll_area_visibility_class(false)`
appended to each site's existing `cn(&[...])` call — the *class* half of the shared
contract, giving every site the same `scrollbar-color`/`scrollbar-gutter` theming
(`SCROLLBAR_CSS`'s native-scrollbar layer, see D3 below), with **no overlay thumb**. This
was live-verified working identically across every site checked (`select`'s
`Positioner`-routed listbox, `sidebar`'s plain div, `resizable`'s flex panel, `table`'s
wrapper) — `getComputedStyle(...).scrollbarColor` resolved to the real themed color at
each one, not the browser default. `command.rs` (a plain `role="listbox"` div with
neither obstacle) could have taken the full overlay, but was deliberately kept at this
same lighter tier for UX consistency across Select/Combobox/Command as a set, rather than
one of the three looking different from the other two.

`message_scroller.rs`'s facade is the one exception: it uses the real `with_scroll_area`
opt-in (merged handlers, context provision) built for the in-place shape, since
`MessageScrollerViewport` was already instrumented and had no `Positioner`/flex-split
obstacle.

This is a real, load-bearing scope reduction from what this document originally
specified for Tier 1 — not a bug, but worth stating plainly: **the custom overlay thumb
built in Phase 2 is fully working and available (`ScrollArea`/`ScrollAreaViewport`,
already used correctly by the three original demo call sites and live-verified with
working drag-to-scroll), but Tier 1's eight registry adoptions all use the lighter
class-only layer instead.** Extending any of them to the full overlay (most plausibly
`command.rs` first, or `Positioner` gaining `on_scroll`/`on_resize` to unlock
`select`/`combobox`) is a natural, well-scoped follow-up, not required by this change as
implemented.

### D3: Thumb styling — CLI-injected `dx-*` CSS as the baseline, registry classes layered on top

Two mechanisms exist in this repo for styling a registry surface: CLI-injected `dx-*` CSS
(today's `SCROLLBAR_CSS` pattern) or a styled registry facade with Tailwind classes on the
element, like most other registry items.

**Decision: (a) CLI-injected `dx-*` CSS is the baseline for the overlay thumb/track,
extending `SCROLLBAR_CSS`; `registry/ui/scroll_area.rs` gains Tailwind classes only where
a consumer needs to override, not as the sole source of the default appearance.**

Reason: `packages/adico-primitives` has no Tailwind. The in-place adoption shape (D2) is
used from inside `adico-primitives`-owned instrumentation in several cases
(`MessageScrollerViewport`, `VirtualList`), so if the thumb's default appearance lived
solely in the registry facade, any scroll area reached through a primitive would render an
unstyled thumb. A `dx-*` baseline styles every reachable path identically; the registry
facade then layers Tailwind class overrides for consumers who want to restyle, the same
relationship `SCROLLBAR_CSS` already has with `ScrollArea`'s existing classes today.

Consequences, each an explicit task:
- `SCROLLBAR_CSS`'s current native-scrollbar rules stay in force for the containers this
  change deliberately does not convert (Carousel, `<textarea>`, page/`<html>` scroll,
  `VirtualList`'s pre-D2-adoption consumer-supplied box in `examples/basic-spa`). The
  overlay-specific rules are additive, not a replacement, in `packages/adico-cli/src/css.rs`
  and all three checked-in `tailwind.css` files.
- The webkit-pseudo-element conflict `4fa3031`'s comment documents (any
  `::-webkit-scrollbar` pseudo pulls the scroller out of standardized-scrollbar mode,
  defeating `scrollbar-width: none`) is exactly the mechanism an overlay design uses to
  *hide* the native scrollbar in some engines. Resolve this empirically in a real browser
  during Phase 1 — confirm whether `scrollbar-width: none` alone (no webkit pseudo)
  suffices to hide the native scrollbar wherever the overlay thumb is present — before
  committing to the CSS shape.
- `statics/styling_usage/scroll-area.json`'s "Pure primitive re-export with no
  adico-authored classes" note and `tailwindOnly: true`, and
  `registry/generated/items/scroll-area.json`'s `semanticTokens: false`/
  `radiusToken: false`/`utilities: []`, all become stale once the registry facade gains
  even override classes — update via `styling-usage sync` and `registry build`, and
  verify the classification note text by hand (the sync tools update data, not prose).

Alternative considered: (b), a fully styled registry facade as the sole source of the
thumb's appearance, matching shadcn's own `scroll-area.tsx` structure exactly. Rejected as
the baseline (not as ever) — it would leave every primitive-internal in-place adoption
site with an unstyled thumb, which defeats the point of a shared contract. Nothing here
prevents `registry/ui/scroll_area.rs` from carrying real classes for the consumer-override
case; it just cannot be the *only* source of default styling.

### D4: Menu height cap is a measurement the positioner already has, not new capability

`proposal.md`'s Tier 2 height cap needs the space available between an anchor and the
viewport edge on the placement side. `positioner.rs:3` already describes a
collision-aware engine, and `space_for_side` (`positioner.rs:108-120`) already computes
exactly this number per side during normal placement — consumed today only by the flip
decision at `:137-143`, never published outward. The work here is exposing an existing
value as a CSS custom property (e.g. `--adico-positioner-available-height`) on the
positioned element, not adding a new measurement capability, and it does not touch
`use_reposition_bridge`'s scroll-follow path (`positioner.rs:328-359`) at all — that path
recomputes position on scroll/resize/mutation; this only changes what gets written once a
position is computed.

Do not carry forward the framing in `openspec/specs/adico-primitives/spec.md:50-68` that
this positioner lacks collision-awareness — that passage is a 2026-09-01 Correction about
`Menubar`'s now-closed exception, superseded by the 2026-09-08 Correction at `:94-110`
("the migration adds real collision-aware placement on top of the same scroll-follow
behavior"). Whether publishing this property makes it part of *"Anchored-overlay
components share one positioning implementation"*'s contract (requiring a `## MODIFIED`
delta with the full existing requirement text) or stays an implementation detail (no
delta needed) is decided during implementation once the property's exact shape is fixed;
`proposal.md` records both branches.

Fallback if publishing the property proves messier than expected (e.g. it needs to differ
between SSR and hydrated states in a way that complicates the custom-property approach): a
viewport-relative cap (`max-h-[min(24rem,60vh)]`-style) that degrades to a sane fixed
default, with true available-height measurement as a follow-up change.

### D5: Scroll-into-view is new, shared behavior — not a per-component reimplementation

**Superseded during implementation by a simpler mechanism than planned; the outcome is
unchanged, the location is not.** This decision originally called for a hand-rolled
viewport-rect/item-rect hook alongside D2's scroll-position tracker. Implementation found
that Dioxus 0.7's `MountedData::scroll_to_with_options` (`dioxus-html-0.7.9`'s
`mounted.rs`) is a direct wrapper around the browser's native `scrollIntoView`
(`#[doc(alias = "scrollIntoView")]`), including a `ScrollLogicalPosition::Nearest` mode
that scrolls the minimum distance — exactly this decision's requirement, already built
into the framework. The "`scrollIntoView` does not exist anywhere in this repo" finding
that motivated this decision was true of *usage*, not *availability*.

It is also centralized one level lower than planned: not in `listbox.rs`, but in
`packages/adico-primitives/src/collection.rs`'s `control_mount_focus` — the function
`use_item`'s focus-tracking effect already calls once per focus change, which already
holds the newly focused item's `Rc<MountedData>` and already calls `md.set_focus(true)`
there for DOM focus. Adding `md.scroll_to_with_options(...)` beside it, in the same
`spawn` block, reaches every `use_item` consumer at once — verified by grep:
`select.rs`, `combobox.rs`, `command.rs`, `context_menu.rs`, `menu.rs`, `menubar.rs`,
`navigation_menu.rs`, plus several non-listbox roving-tabindex consumers
(`accordion.rs`, `radio_group.rs`, `segment.rs`, `tabs.rs`, `tag_group.rs`,
`toggle_group.rs`, `toolbar.rs`, `drag_and_drop_list.rs`) that get the same fix as a
side effect. This wider reach was not separately requested, but is not scope creep worth
gating on: `scrollIntoView` is a no-op on an already-visible target, so it is a strict,
low-risk improvement everywhere it lands, not a behavior change requiring its own
decision. `typeahead.rs` needed no separate wiring — it does not call `use_item`, but its
own focus changes still land in `collection.rs`'s `set_focus`, so it is covered
transitively.

Using `ScrollBehavior::Instant` rather than `Smooth`: this fires on every single
arrow-key step, and an animated scroll per keystroke would visibly lag behind rapid
repeated navigation.

This location does not fight D2's overlay-scrollbar `scrollTop` tracking — both read from
the DOM's own scroll state; `scroll_to_with_options` only ever runs in response to a focus
change (a discrete event), never on every scroll frame, and D2's `onscroll` listener picks
up whatever the browser's native `scrollIntoView` did exactly the same way it picks up any
other scroll.

## Risks / Trade-offs

- **[Positioner scroll-follow breaking]** → `positioner.rs:342`'s capture-phase listener
  fires for genuine native `scroll` events from any descendant, including a `ScrollArea`
  viewport (real `overflow`, real native scroll). Mitigation: D2's constraint that the
  viewport is always a real native-overflow element, never `transform`-driven, applies to
  both adoption shapes without exception; verified live in Phase 6 by scrolling a page
  with an open anchored menu and confirming the menu stays glued.
- **[`position: fixed` clipping by a new scrolling ancestor]** → `demo.rs:35-40` records
  that a non-transformed `overflow: hidden|auto` ancestor can still clip a
  `position: fixed` popover in some browsers. Most relevant to `DropdownMenu` submenus and
  to `Select`/`Combobox` content anchored from inside a scrollable menu. Mitigation: named
  as an explicit live-verification task in Phase 5/6 (open a submenu from inside a
  scrolled menu list), not assumed safe.
- **[SSR/hydration flash on the overlay thumb]** → the thumb needs a first measurement
  (content size vs. viewport size) that SSR cannot produce. Mitigation: render the thumb
  hidden (not at a guessed size) until the first client-side measurement lands, verified
  against `examples/basic-ssr` specifically for flash/jump on hydration.
- **[Generated-record churn cascades]** → adopting `scroll_area` inside `table`/`sidebar`/
  `resizable` changes their `primitiveModules` in `statics/primitive_usage/*.json`; `table`
  is currently classified `presentational` with reason "Static table markup; no
  interactive behavior," which the existing CI check
  (`openspec/specs/adico-registry/spec.md:143-150`) treats as a documented failure once a
  primitive import appears. Mitigation: an explicit reclassification task per affected
  item in `tasks.md`, not left for `primitive-usage sync` to silently paper over.
- **[Propagation drift]** → `examples/basic-spa`/`basic-ssr` are already-drifted older
  snapshots of several Tier 1 files. Mitigation: an explicit grep-and-diff task before
  closing out propagation, confirming zero un-propagated `overflow-y-auto` copies remain
  anywhere under `apps/*/src/components/ui`, `examples/*/src/components/ui`, and
  `tests/installation/*/src/components/ui`.

## Open Questions

- ~~Does the `adico-registry` capability gain an enforced CI check...~~ **Settled during
  implementation:** no. `proposal.md`'s Capabilities section adds no `adico-registry`
  delta — this remains a stated, unenforced convention (task 7.3 handles classification
  honesty via `primitive-usage check`, which is the closest existing gate, not a new
  scroll-specific check). Revisit only if a future PR reintroduces an un-adopted
  `overflow-*` scroll container and the convention proves insufficient in practice.
- Exact shape of the available-height custom property (D4) — name, units, and whether it
  is written on every recompute or only once per open — is an implementation detail to
  pin down in `tasks.md`, not a proposal-level decision.
