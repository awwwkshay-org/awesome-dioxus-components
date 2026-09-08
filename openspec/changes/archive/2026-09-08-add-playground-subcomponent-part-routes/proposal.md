## Status: Rejected (2026-09-07)

This change was fully implemented and live-verified (see `tasks.md`), then
**reverted** on user review of the running app. The user's call: there is no
real value in giving playground subcomponents their own left-nav entries and
dedicated routes, because a subcomponent can't meaningfully be used or
navigated to apart from its root component (nearly all of them panic if
rendered outside their real ancestor composition — see `design.md`'s
Context). The actual need this change was chasing — being able to tell which
subcomponent a given control belongs to — is already fully met by
`scope-playground-controls-to-component-parts`'s `ControlGroup` labeling,
which stays in place. Nested nav and per-subcomponent routes added a second,
unnecessary way to reach information the labeled panel already shows in
place, with no independent-use case to justify the extra navigation surface.

All code from this change (`ComponentPartPage`, the `Route`/`NavItem` nav
restructuring, `ControlGroup`'s and `Demo`'s route-based focus filtering, the
drift-test module, and the `"Accordion Demo"`/`"Dialog Demo"` label renames
this change required) has been reverted. The rest of this document is kept
as-is as the historical record of what was built and why it was rejected —
if nested subcomponent nav is proposed again, the reasoning above and in
`design.md`'s Decisions should be read first, not rediscovered. Should a
future subcomponent genuinely gain standalone usability (e.g. a new registry
part usable outside its current parent), a nav entry/route for it should be
proposed as its own scoped decision, not by reviving this change wholesale.

## Why

`scope-playground-controls-to-component-parts` labeled every control in a
page's panel by the exact component it belongs to (e.g. `/select` now shows
a "Select" group, a "Select Trigger" group, a "Select List" group instead of
one flat unlabeled list). That's still all shown on one page, all at once.
The user wants to go further: let a person drill into an individual
subcomponent from the playground's own left navigation — an expandable
"Select" entry revealing "Select Trigger" / "Select List" underneath it —
and land on a dedicated route for that subcomponent specifically.

A subcomponent cannot be demoed in visual isolation: nearly every
interactive registry subcomponent (`AccordionTrigger`, `DialogContent`,
`SelectTrigger`, `SidebarTrigger`, ...) reads ancestor context via the
panicking `use_context::<T>()` form — roving-focus state, open state, a
positioner's anchor id — with zero graceful fallback anywhere in the
codebase. So a subcomponent's dedicated page renders the exact same full,
real composition as the item's main page; what's dedicated about it is that
the control panel is scoped to just that one subcomponent's group.

The registry's full subcomponent surface turned out to be the wrong basis
for deciding which items and parts qualify. It diverges sharply from what a
page's own composition actually renders (`select.rs` exports 9 registry
components but its page renders only 3 `ControlGroup`s; `calendar` exports
25 but renders 0 — every one of its controls is an ungrouped, hand-written
leaf). Wiring nav/routes from the registry surface would put a nav entry and
a route in front of a panel that renders nothing, or — worse — a panel that
shows another subcomponent's control because it was never inside a
`ControlGroup` to begin with, which is exactly the misattribution
`scope-playground-controls-to-component-parts` exists to prevent.

## What Changes

- **Scope gate**: a page qualifies for nested nav + dedicated subcomponent
  routes only if every control it renders is inside a labeled `ControlGroup`
  — no ungrouped leaf controls. Verified against all 69 playground pages:
  exactly 9 qualify today — Accordion, Attachment, Bubble, Dialog,
  InputGroup, Item, Select, Sidebar, Toolbar. This change wires those 9.
- **One new dynamic route**, `#[route("/:item/:part")] ComponentPartPage { item, part }`,
  added last in `Route` so it only matches after every existing static route
  fails — no existing route changes. One new page,
  `apps/playground/src/pages/component_part.rs`, dispatches `item` to the
  matching existing page component (unchanged) for the 9 wired items, with
  an explicit not-found branch for anything else.
- **`ControlGroup` self-filters by the current route.** No page is edited to
  add this, no generator change, no regeneration of any of the 69 generated
  control files: `ControlGroup` reads `use_route::<Route>()`, and when the
  route names a focused part, every `ControlGroup` whose own label doesn't
  match renders nothing. `Demo` shows a focused title and a "Show all
  controls" link back to the item's unscoped page, from the same resolver.
- **Nav becomes two-level for the 9 wired items**, revealed by the current
  route (being on that item's page or one of its part routes) rather than by
  independent expand/collapse state — no new UI state, and every other
  item's nav entry is completely unaffected.
- **Two small renames** to already-shipped pilot pages, required by this
  feature (not a defect in the prior change): `accordion.rs`'s and
  `dialog.rs`'s hand-written groups both currently share their root
  component's exact label with a second, separate generated group
  (`"Accordion"` appears twice; so does `"Dialog"`) — since nav
  children/URL slugs are keyed by label, this collision is renamed away:
  `"Accordion"` → `"Accordion Demo"`, `"Dialog"` → `"Dialog Demo"` for the
  hand-written groups specifically.
- **Known limitation, explicit follow-up**: the remaining ~60 pages need
  their ungrouped leaf controls wrapped in `ControlGroup` before they can be
  wired the same way — already the prior change's own tracked known
  limitation, now with a concrete payoff for closing it.
- No breaking changes: nothing in `registry/ui/`, `adico-primitives`, the
  CLI, `adico-registry-core`, the xtask generator, or any installed-consumer
  fixture changes. This is playground-app-only routing/nav/presentation
  work.

## Capabilities

### New Capabilities
(none — this change adds requirements to existing capabilities)

### Modified Capabilities
- `adico-playground-structure`: navigation gains an optional, route-revealed
  second level for items whose page renders more than one control group; a
  dynamic-segment route and its dispatcher page follow explicit naming and
  "one page per file" rules; the item↔page and item↔parts mappings stay
  hand-maintained and test-verified, never derived from the registry surface
  or the `pages/` file tree; a page is eligible for this nesting only if
  every control it renders is grouped.
- `adico-playground-demo-controls`: a control panel can be scoped to one
  component's group by the current route, with no generator involvement.

## Impact

- `apps/playground/src/routes.rs` (new route variant, `NavItem` struct,
  `focused_part`/`resolve_part`/`part_slug` helpers, two-level `Layout` nav
  render, a `#[cfg(test)]` drift test)
- `apps/playground/src/components/controls.rs` (`ControlGroup` self-filter)
- `apps/playground/src/components/demo.rs` (focused title + back link)
- `apps/playground/src/pages/component_part.rs` (new)
- `apps/playground/src/pages/mod.rs` (one new `mod`/`pub use` pair)
- `apps/playground/src/pages/accordion.rs`, `apps/playground/src/pages/dialog.rs`
  (label rename fix-up)
- No change to any other playground page, any of the 69 generated control
  files, `packages/adico-xtask/**`, `registry/**`, `packages/adico-primitives/**`,
  the CLI, `adico-registry-core`, or any example/fixture consumer.
- No database, WebAssembly-target, or CLI-installation validation surface
  applies to this change — not applicable, not skipped.
