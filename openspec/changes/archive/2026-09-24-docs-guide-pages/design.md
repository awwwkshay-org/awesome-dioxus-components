## Context

See `proposal.md` — Why. Design-relevant current state:

- `routes.rs` has one flat `Route` enum with `#[layout(SiteLayout)]` wrapping
  everything, `#[layout(PlaygroundLayout)]` nested inside it, and two
  deliberately shell-free harness routes after `#[end_layout]` ×2.
- **The document does not scroll.** `main.rs` is
  `h-dvh … overflow-hidden`; `routes.rs`'s `main` is `overflow-y-auto` and is
  the scrollport. This exists because `Demo`'s percentage `flex-basis` needs a
  definite ancestor height; merge-change task 9.8 records `min-h-dvh` silently
  breaking the playground's 70/30 split.
- `tailwind.css`'s `:root` and `.dark` blocks hold 39 raw token declarations,
  regenerated wholesale by `adico add`.
- `ThemeBuilder` (34 editable tokens) and `ThemeSwitcher` (6 palette presets)
  are installed; `ThemeBuilderLauncher` is mounted only in `PlaygroundLayout`.
- `Prose` and `CodeBlock` already exist from the two preceding changes.

## Goals / Non-Goals

**Goals**
- Guides that answer the questions the README answers today, on the site.
- Documented token values that cannot drift from installed ones.
- A docs shell that leaves the landing and playground shells byte-identical.

**Non-Goals**
- Search, versioning, or an authoring pipeline.
- Any change to the playground's resizable geometry, which is spec-pinned.

## Decisions

### D1 — The theming guide parses the project's own `tailwind.css`

`apps/web/src/pages/docs/tokens.rs` does
`include_str!("../../../tailwind.css")` and parses the `:root { … }` and
`.dark { … }` blocks into `(name, light_value, dark_value)` triples behind a
`OnceLock`.

The guide therefore lists exactly the tokens the CLI installed into *this*
project, with both appearance values, and renders a live swatch per token from
`hsl(var(--name))`. Installing a registry item that introduces a token updates
the guide with no edit.

This is the same guarantee the examples use (`docs-component-examples`, D1):
derive from the artifact rather than restate it. It is worth more here than
usual, because the token block is CLI-generated and will change out from under
any hand-written list.

*Alternative rejected — a hand-written token table.* It is the obvious approach
and it is wrong for exactly the reason the theming page exists: the set is not
owned by this app.

*Alternative rejected — reading the tokens from the DOM at runtime.*
`adico_primitives::theme_mode::read_root_properties` could enumerate live
values, but it only reports the *currently resolved* appearance, cannot show
light and dark side by side, and would make the page's content depend on
whatever a `ThemeBuilder` session had already overridden.

### D2 — `DocsLayout`, nested inside `SiteLayout`, sibling to `PlaygroundLayout`

```
#[layout(SiteLayout)]
  /                         Home
  #[layout(DocsLayout)]
    /docs                   DocsIndex
    /docs/installation …    guides
    /docs/components/:name  DocsComponent
  #[end_layout]
  #[layout(PlaygroundLayout)]
    /playground …
```

A second nested scope rather than folding the sidebar into `SiteLayout`:
`SiteLayout` also wraps `/` and the entire playground tree, and
`adico-web-structure` pins both of those shells. A nested layout confines the
change to `/docs/*`.

### D3 — The sidebar is `sticky` inside `main`, not a second scrollport

`position: sticky` resolves against the nearest scrollport, which is `main`. So
the sidebar sticks correctly with no JavaScript and no second scrollbar.

*Alternative rejected — giving the sidebar its own `overflow-y-auto` column.*
That nests a scroll container inside `main`'s, producing two scrollbars and a
wheel-capture trap. *Alternative rejected — `ResizablePanelGroup`, as the
playground uses.* Its percentage `flex-basis` needs a definite height; docs
content is variable-height, and this is precisely the mechanism task 9.8 records
breaking.

Nothing here changes `main.rs` or `main`'s classes, so the `h-dvh` contract is
untouched. `playground-resizable-split.spec.ts` is the regression guard.

### D4 — Guides reach mobile through `/docs`, not a second Sheet

The sidebar is `lg:` and up. Below that, `/docs` carries a guides section, so
every guide stays reachable without adding a second mobile nav mechanism
alongside the playground's existing `Sheet`.

*Note on `lg:`:* `docs/adico/mobile-first-rules.md` R0 bans `lg:`/`xl:`/`2xl:`,
but states it is "the checklist every `registry/ui/*.rs` component is audited
against". These are app pages, not registry source, and a three-column docs
layout is the case a third breakpoint exists for.

### D5 — A live `ThemeBuilder` on the theming page

Composed directly rather than behind `ThemeBuilderLauncher`'s dialog: the point
is to change the theme and watch the surrounding page respond, which a modal
covering that page defeats. `ThemeBuilder` clears its overrides on unmount
(`use_drop`), so navigating away restores the site theme — the edits are
deliberately transient.

## Risks / Trade-offs

- **A second nested layout could disturb outlet resolution** → the playground's
  nesting and the two shell-free harness routes keep their exact current
  position; verified by the playground Playwright suites, which exercise the
  deepest nesting in the app.
- **The token parser is a text parser over a generated file** → it is
  format-tolerant (it reads `--name: value;` lines inside the two blocks) and
  falls back to an empty list rather than panicking, so a future format change
  degrades to "no swatches" rather than a blank page. A unit test pins the
  parse against the real committed stylesheet.
- **Guides restate things that could go stale** (CLI flags, file paths) →
  wherever the fact is machine-readable it is derived (tokens, component
  counts). Prose that names a command is the residual risk.

## Migration Plan

Additive: six new routes and one nested layout. No data, no consumer contract.
Revertable as one commit. Archiving order: after `docs-component-examples`.
