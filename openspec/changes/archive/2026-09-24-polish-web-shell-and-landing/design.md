## Context

See `proposal.md` — Why. The measurements and code sites that matter:

- `routes.rs:334-335` — `SiteLayout`'s header renders `ThemeSwitcher` +
  `ModeToggle` for every route.
- `routes.rs:404` and `:470` — `PlaygroundLayout` renders the same logo asset
  and the heading "Adico Playground", once per presentation.
- `routes.rs:430-431` and `:487-488` — it renders `ModeToggle` +
  `ThemeSwitcher` again, once per presentation.
- `demo.rs:106` — the canvas grid is
  `hsl(var(--border) / 0.08)` on `bg-muted/20`; at that alpha it does not
  resolve on screen.
- Measured: the `button` demo fills 0.77% of its preview zone at 1440×900.
- `nav.rs` is the single shared nav rendering used by both presentations, so a
  filter added there serves both.

## Goals / Non-Goals

**Goals**
- One brand, one set of theme controls, per screen.
- 69 entries reachable without scrolling past 68 of them.
- The canvas looks like the workspace it already is.

**Non-Goals**
- No change to the resizable splits' seeded sizes or bounds. The emptiness of
  the canvas is partly a *consequence* of the spec'd 70/30 split, and changing
  those numbers is a separate argument with its own evidence.
- No search ranking, fuzzy matching, or keyboard palette. A substring filter
  over 69 known labels is the whole need.

## Decisions

### D1 — Delete duplicated chrome rather than restyle it

`PlaygroundLayout` stops rendering the logo, the "Adico Playground" heading,
the `ThemeSwitcher`, and the `ModeToggle`. `SiteLayout` wraps every playground
route, so all four are already on screen.

`ThemeBuilderLauncher` stays. It is the one control in that footer with no
equivalent in the site header, and it is playground-specific by design.

*Alternative rejected — keeping the heading and shrinking it.* The heading's
only job was naming a section the user already navigated to, under a logo they
can already see. Smaller duplication is still duplication.

*Note on the freeze.* "The `>= md` playground layout is unaffected by mobile
support" constrains the change that added mobile support; it is not a
permanent freeze, and this change carries its own delta. What it protects —
the `ResizablePanelGroup` geometry — is untouched here: only the contents of
`SidebarHeader` and `SidebarFooter` change, not the panels, their bounds, or
the handle.

### D2 — The filter is an `Input` over the existing shared list

`nav.rs` already renders `nav_items()` for both presentations. It gains a
filter signal and renders an installed `Input` above the list; entries whose
label does not contain the typed text (case-insensitively) are not rendered.

Order is untouched — filtering removes entries, it never reorders them, so the
flat A→Z guarantee holds for any filter value.

*Alternative rejected — the installed `Command` component.* It is the richer
fit on paper (it is a command palette), but it brings its own list semantics,
empty state, and keyboard model, and would either replace `SidebarMenu` —
which the shell requirement explicitly names — or nest a list inside a list.
An `Input` plus the existing menu keeps one list rendering.

*Filter state lives in `nav.rs`, not the route*, so it is per-mount and
resets when the mobile sheet closes; a filter that persisted invisibly after
navigation would look like missing entries.

### D3 — The canvas grid gets contrast, and nothing else changes

`0.08` → a value that actually resolves against both themes' `--muted`
surface. The mechanism, the pan behavior, and the Center control are all
already correct and are left alone; only the alpha changes.

*Alternative rejected — a distinct canvas background colour.* The grid is the
honest signal (it moves when you pan, so it communicates the interaction);
a flat colour would just be a different empty box.

### D4 — The landing showcase renders real components, not screenshots

A section composing installed components at their real sizes, so the landing
page demonstrates the product rather than describing it. Interactive controls
stay interactive — a screenshot of a component on a component library's
landing page is a missed opportunity and immediately goes stale.

Popup-family components are excluded from the showcase: they would either sit
closed (showing nothing) or open on load (fighting the page), and
`[[project_positioner_visibility_bug]]` records anchored content sticking at
`visibility: hidden`.

## Risks / Trade-offs

- **Removing the sidebar's theme controls costs a click at `< md`**, where the
  site header's controls are still present but above the sheet. Accepted: they
  remain reachable, and the duplication cost every viewport.
- **Filtering can hide the active page's own entry** while a filter is typed.
  Accepted: the content pane still shows that page, and clearing restores it.
- **The showcase adds components to the landing page's render cost.** Small
  and static; no data fetching.

## Migration Plan

Additive and cosmetic; no route, data, or contract change. Revertable as one
commit. The gate is `playground-resizable-split.spec.ts` plus
`responsive-shell` / `responsive-desktop-shell`, since this edits the shell
those suites measure.
