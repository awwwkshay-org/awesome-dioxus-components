## Context

See `proposal.md` - Why. Relevant existing mechanics, all confirmed against
source before this design was written:

- `registry/ui/resizable.rs` has no dedicated `adico-primitives` module —
  it deliberately avoids `adico_primitives::move_interaction`/`pointer.rs`
  (documented as broken on web) and drags via native per-element
  `onpointerdown`/`onpointermove`/`onpointerup`, with a temporary
  `fixed inset-0` overlay while dragging.
- `ResizablePanel` sizes itself via `flex: 0 0 {size}%` — a percentage of
  its `ResizablePanelGroup`'s own measured container rect, not pixels, not
  a fraction of remaining space. Sizing state (`Signal<Vec<PanelConstraints>>`)
  lives in a private context scoped to each `ResizablePanelGroup` instance
  (`use_context_provider`/`use_context`) — not shared, not persisted; a page
  reload always resets to each `ResizablePanel`'s own `default_size`.
- `ResizablePanel` requires `index: ReadSignal<usize>` (0-based, matching
  sibling order); `ResizableHandle` requires `handle_index: ReadSignal<usize>`
  (the index of the panel immediately before it). `ResizablePanelGroup`
  takes `direction: ResizableDirection` (`Horizontal` default, `Vertical`)
  and needs a definite height (`Vertical`) or width (`Horizontal`) from its
  own parent, since it measures its own rendered rect.
- `apps/playground/src/pages/resizable.rs` already demos this correctly:
  `ResizablePanelGroup { class: "h-64 rounded-md border", ResizablePanel {
  index: 0usize, default_size: 50.0, ... } ResizableHandle { handle_index:
  0usize, with_handle: ... } ResizablePanel { index: 1usize, default_size:
  50.0, ... } }`.
- `apps/playground/src/components/demo.rs`'s current preview/controls split
  (lines 95-147) is one `div { class: "mt-3 grid min-h-0 flex-1 gap-3",
  style: "grid-template-rows: minmax(0, 3fr) minmax(0, 1fr);", ... }`
  wrapping the pannable preview canvas `div` (lines 96-134, its own
  `onpointerdown` pan-start and its own conditional `fixed inset-0` pan-drag
  overlay) and the `ui::Card` controls panel (lines 135-146).
- `apps/playground/src/routes.rs`'s `Layout` (lines 245-297) composes
  `SidebarProvider { Sidebar { SidebarHeader, SidebarContent >
  SidebarGroup > SidebarGroupContent > SidebarMenu (the `nav_items()` loop),
  SidebarFooter, SidebarRail } SidebarInset { header row with
  SidebarTrigger, Outlet } }`.
- `registry/ui/sidebar.rs`'s `SidebarProvider` renders
  `style: "--sidebar-width: 16rem; --sidebar-width-icon: 3rem;"` as a
  hardcoded inline style on its own wrapper `div`, with no prop to override
  either value. `Sidebar`'s own `aside` picks one of
  `w-[--sidebar-width]`/`w-[--sidebar-width-icon]`/`w-0` via a Tailwind
  arbitrary-value class keyed to those two fixed CSS variables, based on
  `(open, collapsible)` state. `SidebarInset` is a `main` with `flex-1` and
  no width of its own — a pure consequence of `Sidebar`'s width.
- Confirmed by reading every render body in `registry/ui/sidebar.rs`:
  `SidebarHeader`, `SidebarContent`, `SidebarGroup`, `SidebarGroupContent`,
  `SidebarMenu`, `SidebarMenuItem`, `SidebarMenuButton`, `SidebarFooter` call
  no `use_context`/`use_sidebar()` anywhere — they are plain,
  attribute/prop-driven wrapper elements (`div`/`ul`/`li`/`button`).
  `Sidebar`, `SidebarTrigger`, and `SidebarRail` are the only three that call
  the private `use_sidebar()` (`use_context::<SidebarCtx>()`, which panics
  with no `SidebarProvider` ancestor).

## Goals / Non-Goals

**Goals:**
- Both splits are user-draggable, with sane min/max bounds, using only the
  already-built `resizable` registry component.
- Zero change to `registry/ui/resizable.rs`, `registry/ui/sidebar.rs`, any
  other registry item, or any generated file.
- Zero change to any of the 69 playground pages — both splits live entirely
  in the two shared components (`Demo`, `Layout`).
- All existing nav/footer/route-navigation behavior, and the preview
  canvas's own pan/center behavior, are preserved byte-for-byte internally.

**Non-Goals:**
- No size persistence across a full page reload — out of scope, `Resizable`'s
  own state model is uncontrolled/internal by design (see Context); adding
  persistence would mean either a new controlled-size prop on `Resizable`
  (a registry change, ruled out — no concrete need has been established
  beyond this one call site) or an app-level signal duplicating what
  `Resizable` already tracks internally, which is unnecessary complexity
  for a dev-tool playground. (Note: `Layout`'s nav/content split may
  naturally persist across in-app navigation regardless, since `Layout` is
  a persistent router layout not re-mounted by navigation — see Risks.)
- No mobile/narrow-viewport responsive behavior for the new nav column —
  this is a desktop dev-tool playground; `Sidebar`'s dropped offcanvas
  mobile behavior is not being replaced with an equivalent.
- No change to `nav_items()`, route navigation logic, active-state
  highlighting, footer contents, or the playground logo/home link.
- No nesting the two `ResizablePanelGroup`s into one — they are two
  independent splits in two different components.

## Decisions

**Preview/controls split: replace the `grid-template-rows` wrapper with a
vertical `ResizablePanelGroup`, panel/handle internals otherwise
unchanged.** `ResizablePanelGroup { direction: ResizableDirection::Vertical,
class: "mt-3 min-h-0 flex-1", ResizablePanel { index: 0usize, default_size:
70.0, min_size: 40.0, max_size: 85.0, <preview canvas div, unchanged> }
ResizableHandle { handle_index: 0usize, with_handle: true } ResizablePanel {
index: 1usize, default_size: 30.0, min_size: 15.0, max_size: 60.0, <ui::Card
controls panel, unchanged> } }`. The preview canvas's own pan-drag overlay
and `ResizablePanelGroup`'s own resize-drag overlay are both `fixed inset-0
z-[100]`, but gated by two independent signals (the canvas's own `pan`
signal vs. `Resizable`'s private `ctx.drag`) that are never both live at
once in practice — a resize-drag starts only from `ResizableHandle`'s own
`onpointerdown`, which the canvas's pan-start handler never sees. No
alternative was seriously considered here — this is a direct, conflict-free
swap.

**Nav/content split: drop `SidebarProvider`/`Sidebar`/`SidebarTrigger`/
`SidebarRail`, keep every other `sidebar` sub-component, wrap both sides in
a horizontal `ResizablePanelGroup`.** `Sidebar`'s width is two hardcoded CSS
variables set once on `SidebarProvider` with no override prop — `Resizable`
sizes via `flex: 0 0 {size}%` on the `ResizablePanel` wrapper it renders,
not on whatever's nested inside it. Wrapping `ResizablePanel` around
`Sidebar` unchanged would nest two non-communicating width-control systems:
the panel's flex-basis on the outside, `Sidebar`'s own fixed-width class on
the `aside` inside it, actively fighting over which one wins. There is no
shared coordinate (CSS variable or context) both systems read from.

Alternative considered: extend `Sidebar`/`SidebarProvider` with a prop to
accept an externally computed width (e.g. a `width` or `style` override),
making it cooperate with `Resizable`. Rejected — the project's own
established rule ("Registry components are never modified solely for
playground's convenience," `adico-playground-structure`) forbids changing a
registry component to accommodate one specific consumer's composition
choice when the existing API isn't broken, only inconvenient for this one
use. Confirmed via source read that `SidebarHeader`/`SidebarContent`/
`SidebarGroup`/`SidebarGroupContent`/`SidebarMenu`/`SidebarMenuItem`/
`SidebarMenuButton`/`SidebarFooter` have no context dependency at all (see
Context) — they can be reused verbatim in a plain `div`, so nothing about
the nav's actual content or structure needs to change, only its outer
sizing shell.

Asked the user directly how to resolve the fork between "keep `Sidebar`'s
collapse/rail behavior, skip resizing this split" and "drop `Sidebar`'s
orchestration, gain drag-resize." They chose the latter — the new `Layout`
is:
```
ResizablePanelGroup { direction: ResizableDirection::Horizontal, class: "h-full w-full",
    ResizablePanel { index: 0usize, default_size: 18.0, min_size: 12.0, max_size: 30.0, class: "flex h-full flex-col",
        <nav column: SidebarHeader (logo/home link), SidebarContent > SidebarGroup > SidebarGroupContent > SidebarMenu (nav_items() loop, unchanged), SidebarFooter (ModeToggle/ThemeSwitcher/ThemeBuilderLauncher, unchanged)>
    }
    ResizableHandle { handle_index: 0usize, with_handle: true }
    ResizablePanel { index: 1usize, default_size: 82.0, min_size: 70.0, max_size: 88.0, class: "flex h-full min-h-0 flex-col",
        <main column: today's header row MINUS SidebarTrigger, then Outlet::<Route> {}, unchanged>
    }
}
```
`SidebarTrigger`'s `"☰"` button is removed: its sole purpose (toggling
`Sidebar`'s collapse state) no longer exists in this design, and there is
nothing left for it to control. Its containing header row is removed
entirely too (refined during implementation, once live-verified: an empty
`div` with only padding/border read as a purposeless visual gap above every
page's own title, not an intentional element worth keeping empty).
`SidebarInset` is not reused for the main column (its `variant`-prop-driven
behavior doesn't add anything a plain `div` doesn't already give here) —
replaced with a plain `div` carrying equivalent layout classes.

**Percentage bounds chosen so panel pairs sum consistently at every
extreme**, avoiding a state where one panel could theoretically be forced
past 100% of the group: nav 12–30 pairs with content 70–88 (both sum to
100); preview 40–85 pairs with controls 15–60 (both sum to 100). These are
the same style of paired min/max already used by `resizable.rs`'s own
mutual-clamp logic (`clamp_delta`), just chosen to fit each split's actual
content (a controls panel needs less minimum height than a live component
canvas; a nav column needs a narrower range than a content area).

## Risks / Trade-offs

- **[Losing `Sidebar`'s built-in collapse-to-icon/offcanvas animation and
  its toggle affordance]** → Accepted per the user's explicit choice;
  drag-resize (including dragging the nav column down toward its 12%
  minimum) is the replacement affordance for "make the nav smaller."
- **[Realized risk, found live post-completion: multi-word nav labels and
  the logo wrapped/clipped instead of eliding at a narrow nav column or a
  high browser zoom]** → The old fixed-`16rem` `Sidebar` never surfaced this
  (`rem` doesn't scale with viewport width or zoom, so it always had enough
  CSS-pixel width for these labels); the new percentage-based
  `ResizablePanel` can resolve to fewer CSS pixels than the unwrapped text
  needs. Fixed by wrapping each nav label and the logo text in their own
  `min-w-0 truncate` span and removing the logo `Link`'s `shrink-0` (moved
  onto just the fixed-size logo image) — a playground-side composition fix
  (task 2a), not a `registry/ui/sidebar.rs` change, since `SidebarMenuButton`
  accepts a generic `children: Element` and was never responsible for
  truncating whatever a caller passes it.
- **[No mobile/narrow-viewport handling for the new nav column]** → Accepted
  as a non-goal; this is a desktop dev-tool playground, and `Sidebar`'s
  dropped mobile behavior was viewport-responsive UI aimed at production
  consumer apps, not this playground's own usage pattern.
- **[`ResizablePanelGroup` needs a definite cross-size from its parent to
  measure against]** → Both integration points already sit inside
  definite-size ancestors: the preview/controls group is a `min-h-0 flex-1`
  child of `Demo`'s own `section` (already establishing a definite height
  today via the existing grid); the nav/content group replaces
  `SidebarProvider`'s own `flex min-h-svh w-full` root, so it becomes the
  new root and defines its own `h-full w-full`, same sizing contract
  `SidebarProvider` provided before.
- **[The two splits behave differently across in-app navigation]** →
  `Demo` is re-mounted fresh by every page navigation (same as every other
  per-page demo-state signal in this playground), so its preview/controls
  split always resets to its coded `default_size`s on navigation. `Layout`
  is declared `#[layout(Layout)]` in the `Route` enum — a persistent router
  layout mounted once, not re-mounted by in-app navigation — so its
  nav/content split's resized size naturally persists across navigation
  within the same session, resetting only on a full page reload. This is
  an accepted, documented difference (see the added spec requirement's
  scenarios), not a defect to reconcile — forcing them to behave identically
  would mean either making `Demo`'s split persist via new app-level state
  (unneeded complexity, see Non-Goals) or artificially remounting `Layout`
  on every navigation (which would undo its whole purpose as a layout
  route).
- **[Realized risk, found live post-completion: `ResizableHandle`'s grip
  decoration had two independent, pre-existing registry bugs this change
  was the first to exercise in both directions]** → (1) the grip's box
  never rotated for `direction`; (2) as a flex child of the handle (itself
  `flex`, no explicit direction), the grip's default `flex-shrink: 1` +
  `min-width: auto` let it get crushed toward the handle's own 1px
  constraint whenever that constraint fell on the SAME axis as the flex
  row's main axis (`Horizontal` direction) — confirmed via
  `getComputedStyle`, a 12px-coded grip rendered at 2px. `Vertical`
  direction's 1px constraint lands on the cross axis instead, which
  `align-items: center` doesn't shrink, so it was invisible until this
  change exercised the other direction. A third, separately-found issue
  (grip fill and the line's own fill sharing one token, causing the grip's
  touching edge to blend invisibly into the line) is documented in task
  4.1. All three fixed directly in `registry/ui/resizable.rs` (task 4.2) —
  genuine defects reproducible by any consumer, not playground-specific,
  so not worked around at the call site.
- **[Realized risk, found live post-completion: the resize handle had no
  visual breathing room, sitting flush against both panels' own borders]**
  → Fixed with `gap-3` on `Demo`'s `ResizablePanelGroup` (task 4.3) — a
  flex `gap` is excluded from the space `ResizablePanel`'s percentage
  `flex-basis`es divide, so it doesn't perturb the 70/30 default sizing.
- **[Realized risk, found live post-completion: task 4's fixed grip still
  read as a plain rectangle, not a recognizable drag handle]** → Checked
  upstream shadcn/ui's actual current `resizable.tsx` source directly (`gh
  api`, not memory) rather than guessing further. Two concrete gaps: this
  registry's grip swapped its own `h-4`/`w-3` classes per direction, where
  upstream keeps the box fixed and rotates it 90deg via a CSS selector;
  and upstream renders a `GripVerticalIcon` inside the box, which is what
  actually reads as "a handle" — an empty box, however correctly shaped
  and colored, doesn't. Rewrote to match upstream exactly: fixed `h-4 w-3`
  + conditional `rotate-90` (task 5.2), `adico_primitives::icons::GripVertical`
  inside with an explicit `text-background` override (the icon's default
  `stroke="currentColor"` would otherwise be an invisible near-white
  glyph on the near-white `bg-foreground` chip). This is the first
  registry-source dependency this component has ever declared on
  `adico-primitives` (previously icon-free), so `registry.json`'s
  `cargoDependencies` and `statics/primitive_usage/resizable.json` both
  needed updating too (task 5.3) — verified against the exact convention
  already established by icon-only sibling items (`spinner`, `breadcrumb`:
  no `"web"` feature; `attachment`/`carousel`/`data-table`: the same
  "icons import is purely decorative" `reason` phrasing, two of which
  already cross-referenced "resizable.json's identical rationale" as
  though this update was anticipated).
