## Why

`apps/playground`'s app shell (`routes.rs`'s `Layout`) lays its nav column
and page content out with a horizontal `ResizablePanelGroup` whose nav
panel has `min_size: 12.0` (percent). Confirmed live at a 375×812 viewport
(`dx serve`, real browser pass, an injected same-origin iframe used as a
viewport proxy since window resize is unreliable in this environment): the
nav column renders at ~114px wide and every nav label truncates to a
single letter plus ellipsis ("A...", "B...", "C..."), making navigation
unusable on a phone. `main.rs`'s root (`h-screen w-screen overflow-hidden`)
compounds this — `h-screen` (100vh) doesn't account for a mobile browser's
URL bar, and nothing at the document level can scroll if a mobile layout
ever needs more vertical space than the viewport gives it.

This is a known, explicitly-deferred gap, not a new discovery: the
recently-archived `2026-09-09-make-registry-components-mobile-first`
change swept every `registry/ui/*.rs` component to a mobile-first baseline
but explicitly left `Sidebar`'s own mobile behavior deferred, because
Dioxus's `document::eval` recv-loop pattern for JS viewport/media-query
detection is confirmed non-functional in this runtime
(`registry/ui/sidebar.rs:9-18`) and building a real fix needs a working
primitive that change didn't build. That constraint still holds. This
change fixes the shell using pure CSS breakpoints instead — no JS
viewport detection — closing the gap for the one shell playground actually
runs, without touching the registry.

By contrast, `components/demo.rs`'s vertical `ResizablePanelGroup` (the
per-page preview/controls split used on all ~69 component pages) was
checked live at 375px and already stacks and scrolls acceptably with no
horizontal overflow — it is out of scope here absent new evidence it's
broken.

## What Changes

- `Layout` (`routes.rs`) renders two CSS-breakpoint-selected markup trees:
  below `md`, a mobile top bar (logo, hamburger trigger, theme controls)
  with the nav list opening in a `Sheet` overlay driven by a plain
  open/closed `Signal<bool>`; at `md` and above, today's
  `ResizablePanelGroup` layout, unchanged. The signal is only ever
  click-driven (hamburger open, Sheet dismiss/navigate close) — never used
  to detect viewport width.
- The `SidebarMenu`/`nav_items()` loop that renders the nav list is
  extracted out of `Layout` into one shared component
  (`apps/playground/src/components/nav.rs`) so both trees render from the
  same source instead of forking the nav list.
- `main.rs`'s root sizing moves off `h-screen w-screen` to viewport units
  that behave correctly on mobile browsers (`dvh`/`svh`-based height,
  `w-full` instead of `w-screen`), with scroll behavior decided
  deliberately for the mobile layout.
- New/extended Playwright coverage: a mobile-viewport (375×812) shell
  spec (nav reachable via hamburger, dismissible, no horizontal overflow)
  plus a desktop-invariance geometry check proving the `>=md` layout is
  pixel-identical to today's.
- Two pre-existing, out-of-scope issues are documented (in this change's
  `tasks.md`) rather than fixed: `Sheet` likely shares Dialog's confirmed
  broken focus-trap/scroll-lock/`aria-hidden` machinery
  (`packages/adico-primitives/src/{lib,dialog}.rs`), and a
  Popover-anchored Calendar overflows the viewport on `/calendar` despite
  both components already carrying individually-correct mobile-first/clamp
  classes — a likely Positioner anchor/edge-avoidance gap, unrelated to
  this shell.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `adico-playground-structure`: the nav column's sizing/composition
  requirement ("composed from the installed `resizable` component") is
  scoped to `>= md` viewports; a new requirement is added for the `< md`
  mobile shell (hamburger-triggered `Sheet` nav, no JS viewport
  detection, desktop layout provably unchanged).

## Impact

- Affected code: `apps/playground/src/routes.rs`, `apps/playground/src/main.rs`,
  new `apps/playground/src/components/nav.rs`, `apps/playground/src/components/mod.rs`
  (module wiring), `tests/playwright/` (new/extended specs).
- Not affected: `registry/ui/*.rs`, `packages/adico-primitives/**`,
  `components/demo.rs` and any page under `apps/playground/src/pages/`.
- No public contract, database, or deployment changes. No breaking change
  to any registry item's API. Runtime risk is confined to the playground
  app itself (a consumer fixture/demo app, not a distributed package), and
  the `>= md` desktop path is required to render byte-identical markup to
  today, so existing desktop usage of the playground is not expected to
  change at all.
