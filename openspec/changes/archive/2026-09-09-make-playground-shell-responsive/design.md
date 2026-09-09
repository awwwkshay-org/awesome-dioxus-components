## Context

See `proposal.md` - Why. Two hard constraints shape every decision below:

1. **No JS viewport detection.** `registry/ui/sidebar.rs:9-18` documents
   that Dioxus's `document::eval` recv-loop pattern for detecting the
   viewport is confirmed non-functional in this runtime, and that's the
   documented reason `Sidebar`'s own mobile mode is deferred. This change
   cannot use that pattern either, directly or by depending on a registry
   feature that would need it.
2. **The `>= md` layout must stay provably unchanged.** `Layout`
   (`routes.rs:256-339`) already satisfies `adico-playground-structure`'s
   existing "nav column is composed from the installed `resizable`
   component" requirement; that composition is not being redesigned, only
   scoped to `>= md`.

`Layout` today composes `Sidebar`'s structural sub-parts (`SidebarHeader`,
`SidebarContent`, `SidebarMenu`, etc.) directly inside a `ResizablePanel`,
bypassing `SidebarProvider`/`Sidebar`'s own root and `use_sidebar()`
context entirely (per `adico-playground-structure`'s existing
requirement, since `Sidebar`'s CSS-variable-driven width can't cooperate
with a drag-resize handle). That means `SidebarTrigger` (which requires
`use_sidebar()`) isn't available to this shell without adopting the
provider it deliberately avoids — the mobile hamburger button is a plain
installed `Button`, not `SidebarTrigger`.

`Sheet` (`registry/ui/sheet.rs`) is `adico_primitives::dialog::DialogRoot`
under a slide-in styled skin — same controlled-open primitive as `Dialog`,
so it takes an `open`/`on_open_change` (or `default_open`) prop the same
way `Dialog`/`Popover`/etc. already do elsewhere in this playground.

## Goals / Non-Goals

**Goals:**
- Nav is reachable and fully legible at 375px, with today's `>= md`
  layout unchanged.
- The nav list itself (labels, active-route highlighting, click-to-
  navigate) is defined once and rendered by both the mobile and desktop
  trees.
- The fix is pure CSS breakpoint selection (`hidden`/`flex` pairs) plus a
  click-driven open/closed signal — nothing that depends on measuring the
  viewport at runtime.

**Non-Goals:**
- Reworking `components/demo.rs`'s preview/controls split — checked live
  at 375px and already acceptable.
- Fixing `Sheet`'s (shared with `Dialog`) focus-trap/scroll-lock/
  `aria-hidden` defect, or the Popover-anchored-Calendar overflow found
  on `/calendar` — both documented in `tasks.md` as known, pre-existing,
  out-of-scope issues.
- Any change to `registry/ui/*.rs` or `packages/adico-primitives/**`.
- A real collapsible/icon-rail `Sidebar` mode, or reinstating
  `SidebarProvider`/`use_sidebar()` — out of scope, and would re-litigate
  `adico-playground-structure`'s existing resizable-nav-column
  requirement.

## Decisions

**Two chrome trees, CSS-selected, one shared nav body — but one shared
content pane, not a duplicated one.**
`Layout` renders both nav chromes (mobile top bar + `Sheet`, desktop
`ResizablePanelGroup` nav column) unconditionally; Tailwind's
`hidden md:flex` / `flex md:hidden` pair decides which is visible at any
given viewport, so no Rust-side viewport branching exists at all. The
`SidebarMenu`/`nav_items()` render loop moves out of `Layout` into a new
`apps/playground/src/components/nav.rs` (`NavList`, taking the current
route and an `onnavigate` callback), called from inside both the mobile
`Sheet` content and the desktop `ResizablePanel`'s `SidebarContent`.

**Revised during implementation**: an earlier version of this decision
described the routed content (`Outlet::<Route>`) as living inside each of
two fully-duplicated trees. That's wrong and was corrected before
implementing it: `Outlet::<Route>` must mount exactly once. A page can
render positioner-portalled content (e.g. an "open by default" Popover,
confirmed live on `/calendar`) that appends to `document.body` outside
its own DOM ancestry — mounting `Outlet` twice, with one copy under a
`hidden` ancestor, would still show that portalled content on screen,
since `hidden`'s `display:none` never reaches a portal. So only the NAV
chrome forks into two trees; the content pane (with `Outlet` inside it,
exactly where it already lived in the `>= md` `ResizablePanelGroup`) stays
the single, always-mounted one. Below `md`, the nav `ResizablePanel` and
its `ResizableHandle` get `class: "hidden md:flex"` (removing them from
the flex row and freeing their space); the content `ResizablePanel` gets
an added `max-md:flex-1!` class, an `!important` override that wins
against its own `style="flex: 0 0 {size}%"` (an inline style set by the
resizable-drag mechanism) specifically below `md`, letting it fill the
freed row width. Confirmed live in a real browser (an injected
same-origin iframe used as a 375px viewport proxy, per this change's own
verification plan) that Tailwind v4.1.5's trailing-`!` important modifier
does override that inline style at `< md` and does not apply at `>= md`
(computed `flex` there stays the unmodified `0 0 82%`) — this toolchain
has shown other v4-specific footguns (`w-[--sidebar-width]` compiling to
invalid CSS), so this was verified empirically, not assumed from CSS
cascade rules alone. This is the one structural move that keeps the nav
chrome from drifting the next time a page is added — everything else
about each chrome tree is independent markup, and the content pane never
forks at all.

**Mobile top bar + `Sheet`, not a second resizable panel.**
Below `md`: a fixed-height top bar (logo, a plain `Button`-based hamburger
toggling a `Signal<bool>`, theme controls) sits above the routed content;
the hamburger opens `Sheet`/`SheetContent { side: SheetSide::Left }`
containing `NavList` plus the same footer controls
(`ModeToggle`/`ThemeSwitcher`/`ThemeBuilderLauncher`) `Layout` already
renders in `SidebarFooter` today. Closing happens on: explicit dismiss
(`SheetContent`'s existing close affordance / overlay click, which
`Sheet` already provides since it's `Dialog` underneath), and on
navigation (the `onnavigate` callback closes the sheet before/while
pushing the route) so picking a page doesn't leave the overlay open
underneath the new page.
*Alternative considered*: reuse `Sidebar`'s own deferred mobile mode.
Rejected — it doesn't exist yet, and building it would mean solving the
exact JS-viewport-detection problem this change is designed to avoid;
`Sheet` already exists, is already used elsewhere in this playground, and
needs no viewport awareness (it's opened by a click, not a media query).

**Desktop tree is the existing `ResizablePanelGroup`, functionally
untouched.**
The `>= md` rendered result is exactly today's `Layout` body: the
nav-list-loop-to-`NavList` swap (above) is a pure refactor with identical
output, and the `hidden md:flex` / `max-md:flex-1!` classes added to the
nav panel, handle, and content panel are no-ops at `>= md` by
construction (the `md:` and `max-md:` variants are mutually exclusive
breakpoint ranges). Confirmed live: the content panel's computed `flex`
at a 1280px viewport is unchanged (`0 0 82%`), and
`responsive-desktop-shell.spec.ts` asserts this as a regression gate. This
is what makes "desktop unchanged" a checked guarantee rather than a claim
requiring manual pixel comparison of edited code.

**Root sizing:** `main.rs`'s `div { class: "h-screen w-screen
overflow-hidden ..." }` becomes `h-dvh w-full overflow-hidden` (`dvh`
tracks the mobile browser chrome instead of over-counting it; `w-full`
avoids `w-screen`'s scrollbar-gutter overcount; `overflow-hidden` is kept
because each layout already owns its own internal scroll region — the
routed content's `overflow-y-auto` wrapper on desktop, and the mobile
tree's own scrollable regions — so document-level scroll is not needed
once the mobile tree itself doesn't overflow vertically). The archived
mobile-first change already confirmed `dvh`/`svh` utilities compile
correctly on the pinned Tailwind v4.1.5 toolchain (used for
`drawer.rs`/`toast.rs`), so no new toolchain risk here.

## Risks / Trade-offs

- **`Sheet`'s focus-trap/scroll-lock/`aria-hidden` defect** (shared with
  `Dialog`, per the project's known positioner/dialog defect notes) means
  the mobile nav overlay's accessibility behavior (focus trapped inside,
  background `aria-hidden`, scroll lock) likely won't fully engage. →
  Mitigation: click-driven open/close still functions (confirmed working
  behavior class, independent of that defect), so navigation itself is
  not blocked; documented as a known limitation in `tasks.md`, not gated
  on for this change's completion.
- **Positioner-anchored footer controls** (`ModeToggle`/`ThemeSwitcher`/
  `ThemeBuilderLauncher`) can get permanently stuck
  `visibility: hidden` (pre-existing, unrelated bug) when co-mounted with
  another overlay. → Mitigation: none needed for this change; if observed
  during verification, confirm it's the pre-existing bug (check for
  `style="visibility: hidden"`) rather than treating it as a regression.
- **Duplicated footer controls markup** (mobile `Sheet` content and
  desktop `SidebarFooter` both render `ModeToggle`/`ThemeSwitcher`/
  `ThemeBuilderLauncher`) → Mitigation: acceptable duplication (two call
  sites, not two implementations); `NavList`'s extraction is what matters
  for drift-prevention, since the nav list is the part that grows with
  every new component page.

## Migration Plan

No data migration. Playground is a maintained example app, not a
distributed package — the change ships by merging and the next `dx serve`
picks it up. No rollback complexity beyond a normal revert.

## Open Questions

None — the two hard constraints (no JS viewport detection, desktop
unchanged) and the `Sheet`-based mobile nav approach are settled above.
