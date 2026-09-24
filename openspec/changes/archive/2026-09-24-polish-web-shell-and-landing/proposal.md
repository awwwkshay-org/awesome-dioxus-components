## Why

Three problems, all measured on the running site rather than guessed at.

**The playground shell says everything twice.** `SiteLayout`'s header already
carries the adico wordmark and logo, a `ThemeSwitcher`, and a `ModeToggle`.
`PlaygroundLayout`, nested directly inside it, renders the *same logo* under
the heading "Adico Playground" and the *same* `ThemeSwitcher` and `ModeToggle`
again in its sidebar footer — in both the `>= md` and `< md` presentations.
A visitor sees one brand twice and one pair of theme controls twice, stacked
vertically about 60px apart, and the duplication eats the sidebar space the
navigation actually needs.

**The navigation is 69 items with no way to narrow it.** Reaching `tooltip`
means scrolling past everything from `accordion`.

**The preview canvas reads as an empty box.** The demoed component occupies
**0.77%** of it (4,441px² of 574,894px² at 1440×900 for `button`). The canvas
*is* pannable and does draw a grid, but at `hsl(var(--border) / 0.08)` that
grid is invisible, so nothing signals the space is a workspace rather than
dead area.

Separately, the landing page still never shows a component. The product is
components; a visitor reads four prose cards and an install command.

## What Changes

- **De-duplicate the playground shell.** `PlaygroundLayout` stops rendering
  the logo, the "Adico Playground" heading, the `ThemeSwitcher`, and the
  `ModeToggle` that `SiteLayout` already provides. `ThemeBuilderLauncher`
  stays: it is playground-specific and has no equivalent in the site header.
- **Filter the navigation.** The reclaimed sidebar header holds a filter
  composed from the installed `Input`, narrowing the list as you type while
  preserving its flat alphabetical order.
- **Make the canvas legible as a canvas** by raising the grid's contrast, so
  the pannable workspace looks like one.
- **Show components on the landing page**, composed from installed registry
  components.

Non-goals: no change to the resizable splits' seeded sizes or bounds (preview
60–80% / controls 20–40%, nav 12–30% / content 70–88%); no change to
`registry/ui/*.rs`; no change to the docs route tree.

## Capabilities

### Modified Capabilities

- `adico-web-structure`: the playground shell requirement drops the chrome
  `SiteLayout` already provides and gains a navigation filter; the
  flat-alphabetical requirement is amended to permit filtering while
  preserving order; the below-`md` requirement follows the same
  de-duplication; the preview-zone requirement gains the rule that the
  pannable area is visually identifiable; and the landing-page requirement
  gains the rule that it shows real components.

## Impact

**Modified**
- `apps/web/src/routes.rs` — `PlaygroundLayout`'s sidebar header and footer in
  both presentations.
- `apps/web/src/components/nav.rs` — the shared nav list gains filtering.
- `apps/web/src/components/demo.rs` — canvas grid contrast.
- `apps/web/src/pages/index.rs` — the showcase.

**Not touched**
- `registry/ui/*.rs`, `apps/web/src/components/ui/*`, `adico.lock`,
  `components.json`, `packages/*`, the docs route tree, `main.rs`.

**Risk: this edits the `>= md` playground layout**, which
"The `>= md` playground layout is unaffected by mobile support" froze. That
requirement constrains *adding mobile support* — it is not a permanent
freeze — but the resizable geometry it protects is the same geometry that
broke once before, so `playground-resizable-split.spec.ts` and both shell
suites are the gate.

**Risk: filtering hides navigation entries.** The filter must never be the
only way to reach a page: cleared, the list is exactly what it is today.
