## Status: Rejected (2026-09-07)

See `proposal.md`'s "Status: Rejected" section for the full rationale. In
short: subcomponents can't be used or navigated to independently of their
root, so a dedicated nav entry/route for one adds a navigation surface with
no independent destination — everything below documents a design that was
built, live-verified, and then reverted for that reason, kept for reference
so the same approach isn't re-explored without first reading why it didn't
stick.

## Context

See `proposal.md` - Why. Relevant existing mechanics, all confirmed against
source before this design was written:

- `apps/playground/src/routes.rs` today is completely flat: `Route` has one
  `#[route("/x")] XPage {}` variant per page (69 total), only one
  `#[layout(Layout)]` is used anywhere, and no `#[nest]`/`#[child]`/dynamic
  segment appears in this app. `nav_items()` is a hand-written
  `Vec<(&'static str, Route)>`; `Layout` renders it with a single flat `for`
  loop, one `SidebarMenuItem` per entry — no collapsible/nested nav exists.
  `pages/mod.rs` has one `mod x;` + `pub use x::XPage;` per file, no
  subdirectories under `pages/`.
- `dioxus-router` **0.7.9** is pinned (macro **0.7.10**). The macro's
  vendored source (`route_tree.rs`) fully supports `#[nest]`/`#[end_nest]`/
  `#[child]`/dynamic segments — unused in this app, but `apps/docs/src/main.rs:97`
  already has one dynamic route in the monorepo, `#[route("/components/:name")]`,
  a single dynamic route rather than one static route per component. That's
  the closest existing precedent to build on.
- `apps/playground/src/components/controls.rs`'s `ControlGroup` (added by
  `scope-playground-controls-to-component-parts`) already wraps every
  control — generated and hand-written — under a label naming the exact
  component that declares it. This is the seam this change filters through;
  no new wrapping component is introduced.
- Nearly every interactive subcomponent's context read
  (`AccordionItem`/`Trigger`/`Content`, `DialogTrigger`/`Content`/`Close`/...,
  `SelectTrigger`/`List`/`Option`/..., `Sidebar`/`Trigger`/`Rail`) uses the
  panicking `use_context::<T>()` form, confirmed with zero
  `try_consume_context`/`Option`-fallback instances anywhere in
  accordion/dialog/select/sidebar. Rendering a subcomponent outside its real
  ancestor composition crashes; there is no precedent anywhere in this
  codebase for isolated subcomponent rendering.
- The registry surface and a page's actually-rendered groups diverge
  sharply: `select.rs` exports 9 registry components, its page renders 3
  groups; `calendar` exports 25, its page renders 0 (three ungrouped hand
  leaves); `dialog.rs` exports 10, its page renders 10 (a 1:1 case).
  Confirmed by diffing each generated file's emitted `<Part>Controls`
  functions against each page's actual `ControlGroup`/generated-panel
  references. Exactly 9 of 69 pages have zero ungrouped leaf controls:
  Accordion, Attachment, Bubble, Dialog, InputGroup, Item, Select, Sidebar,
  Toolbar.
- `apps/playground/src/adico_lib/cn.rs`'s `cn` is a plain space-joiner, not
  a tailwind-merge-style conflict resolver — class-order cannot be relied on
  to win a styling conflict.
- `registry/ui/sidebar.rs` has no `SidebarMenuSub`/`SidebarMenuSubItem`/
  `SidebarMenuSubButton`/`SidebarMenuAction` parts today — confirmed absent
  from both the registry source and every reference to it in the repo.

## Goals / Non-Goals

**Goals:**
- A user can navigate from an item's nav entry to a dedicated route for any
  one of its rendered control groups, and back.
- The mechanism requires no change to any existing page's controls, no
  generator change, and no regeneration of any of the 69 generated control
  files.
- No existing route, nav entry, or page behavior changes for the ~60
  not-yet-wired items.
- The item↔page and item↔parts mappings are correct by construction or
  caught by an automated test — never silently allowed to drift.

**Non-Goals:**
- No visual isolation of a subcomponent — established infeasible (panics).
- No wiring for any page with an ungrouped leaf control — that's the
  explicit follow-up, not this change.
- No new registry parts (`SidebarMenuSub*`) — a legitimate, separate future
  change; the nested-nav treatment here is the deliberate interim, not an
  oversight to flag as a defect later.
- No change to `classify_prop_type`, prop extraction, or anything in
  `packages/adico-xtask` — this change's whole premise is that the existing
  generated output needs no modification.

## Decisions

**Routing: one dynamic dispatcher route, not one `#[nest]`-wrapped route per
item.** `#[route("/:item/:part")] ComponentPartPage { item: String, part: String }`,
declared **last** in `Route`. Root-level dynamic branches in the generated
route-tree match code are tried, in declaration order, only after every
static branch has already failed to match the full path — confirmed
directly against `dioxus-router-macro` 0.7.10's `route_tree.rs` codegen —
so a two-segment URL like `/accordion/accordion-item` falls through all 68
existing single-segment static routes with zero changes to any of them
before reaching this route. One new file, `pages/component_part.rs`, holds
one new page component that matches `item` against the 9 wired items and
renders that item's existing page component completely unchanged, plus an
explicit not-found branch (since this route would otherwise silently
swallow any unmatched two-segment URL).

Alternative considered: `#[nest("/accordion")] #[route("/")] AccordionPage {} #[route("/:part")] AccordionPartPage {} #[end_nest]`
per item. Rejected: `part` is a plain `String` in both approaches, so
nesting buys no type safety; it costs one new near-boilerplate file per
item (~9 now, heading toward ~50+ once the follow-up wires the remaining
pages), each nearly identical to the last; and it requires touching each
item's existing flat route declaration to nest it, which the dispatcher
avoids entirely. The dispatcher costs exactly one match arm per newly-wired
item, forever.

**Focus mechanism: `ControlGroup` reads the current route directly, not a
context provider.** `ControlGroup` calls `use_route::<Route>()`, resolves
the focused part (if any) through a `focused_part(&Route) -> Option<FocusedPart>`
helper in `routes.rs`, and renders nothing when a focus exists and its own
`part` label doesn't match it. `Demo` reads the same resolver for a focused
title suffix and a "Show all controls" `Link` back to the unscoped route.

Alternative considered: a context provided once by `ComponentPartPage` and
consumed by `ControlGroup`. Rejected — `use_context_provider`'s initializer
runs once per component scope, so navigating between two sibling part
routes of the same item (`/select/select-trigger` → `/select/select-list`)
would reuse the same `ComponentPartPage` scope and serve a stale focus
without additional `Signal` + `use_reactive!` plumbing to keep it current.
`use_route::<Route>()` is already reactive and always current, and needs no
provider at all — the dispatcher's only job stays "pick which page to
render."

A rendered-but-hidden `ControlGroup`'s own internal `use_signal`/`use_effect`
state-sync still runs; only its subtree isn't mounted. This is what makes
"same real composition, different scoped panel" literally true rather than
approximate, and it's also why the standalone-render panic risk never
arises here — the full composition is always mounted regardless of which
group's panel is visible.

**Nav data: hand-maintained `parts`, protected by a drift test — not
generator-emitted.** `nav_items()`'s tuple becomes
`NavItem { label, slug, route, parts: &'static [&'static str] }`. `parts`
is the exact, ordered list of `ControlGroup` labels that item's page
renders. Deriving this from the xtask generator's `introspection.components`
was considered and rejected: the registry-surface-vs-rendered-groups
divergence documented in Context means a generated table would populate the
nav with entries pointing at panels that render nothing (`select` would get
9 nav children for 3 real groups) — worse than the hand-maintained status
quo, and an abstraction (a new generator input source reading `pages/*.rs`)
built for exactly 9 items ahead of a concrete enough need. Instead, a
`#[cfg(test)]` test in `routes.rs` (this crate is a workspace member;
`cargo test --locked --workspace` already runs in CI) uses `include_str!`
to read each of the 9 wired pages' source and asserts, in both directions,
that a `parts` array matches every literal `ControlGroup { part: "..." }`
string actually present (accounting for a page's generated-panel imports by
checking the corresponding generated file's own emitted label). A future
item added to `parts` without a matching `ControlGroup` in its page fails
this test, and vice versa.

**Nav UI: route-derived reveal, nested `SidebarMenu`, no new state.**
`Layout`'s loop becomes two levels: the existing `SidebarMenuItem` +
`SidebarMenuButton` row, plus — only when `item.parts.len() > 1` and the
current route belongs to that item — a nested `SidebarMenu` (a `<ul>`
inside the existing `<li>`) with one smaller `SidebarMenuButton` per part.
Expansion is derived entirely from the current route: visiting `/select`
reveals Select's parts; navigating away collapses them. A static `›` glyph
marks a multi-part item's row so the affordance is visible before the first
click.

Alternatives considered and rejected: `ui::Collapsible` (its trigger is a
full-width bordered pill, visually wrong inside a sidebar row; and
`SidebarMenuButton` has no `as_child` to compose around it — adding one
would violate "registry components are never modified solely for
playground's convenience," an existing, unmodified requirement); a real
`SidebarMenuSub`-style registry part (doesn't exist yet in
`registry/ui/sidebar.rs` — legitimate future registry work, deliberately
out of scope here, tracked as a named follow-up rather than worked around).
Indentation is applied via the nested `SidebarMenu`'s own `class` (whose
base classes carry no padding utility, so no `cn`-ordering conflict is
possible — `cn` is a plain joiner, confirmed, not a merger), never by
overriding `SidebarMenuButton`'s own base padding.

**Label-collision fix-up: rename the hand-written duplicate, not a "Root"
placeholder.** `accordion.rs` and `dialog.rs` each render two separate
`ControlGroup`s sharing their root's exact label — a generated empty-state
root panel, and a hand-written group standing in for something the
generator can't produce (Accordion's demo-scenario toggle between
`Accordion`/`AccordionMulti`; Dialog's `open` state, a real prop the
generator can't extract from a re-exported component). Since nav children
and URL part-slugs are keyed by label, two groups sharing one label can't
resolve to two distinct routes. Considered and explicitly rejected per user
direction: a generic "Root" placeholder label for one of the two. Chosen
instead: rename only the hand-written group to a genuinely distinct label
following the project's existing sibling-naming convention (`Accordion
Item`, `Accordion Trigger`, ...) — `"Accordion"` → `"Accordion Demo"`,
`"Dialog"` → `"Dialog Demo"`. This is a two-line fix-up to the already-shipped
pilot pages, required by this feature's own routing needs, not a
retroactive defect fix to the prior change.

## Risks / Trade-offs

- **[The dynamic dispatcher route is a catch-all for any two-segment URL]**
  → Mitigated by an explicit, required not-found branch in
  `ComponentPartPage` for any `item`/`part` combination that doesn't match a
  wired item and its real parts — never a crash, never a silently wrong
  composition.
- **[Only 9 of 69 items get this treatment]** → Accepted and named
  explicitly in the proposal as a follow-up, the same staging pattern the
  prior change used (5 of ~53 pilot pages). The gate is objective (every
  control grouped) rather than an arbitrary pilot count, so "which item is
  next" is unambiguous once its page's leaf controls are grouped.
- **[`parts` is hand-maintained data that could silently drift from a
  page's real groups]** → Caught by the CI-gated drift test in `routes.rs`,
  not left to manual review.
- **[Two already-shipped pilot pages need a label change]** → Small, scoped
  to exactly the two known collisions; `cargo xtask playground-controls
  check` staying a zero-diff pass after this change proves no generated
  file needed to change as a result.
- **[No real `SidebarMenuSub`-style registry part exists for this nested
  nav]** → Accepted; the interim composition (a nested `SidebarMenu` inside
  the same `SidebarMenuItem`) is valid HTML and works today. The registry
  parity gap is named as a separate, future change rather than worked
  around with a registry modification made solely for playground's
  convenience (which an existing requirement already forbids).
