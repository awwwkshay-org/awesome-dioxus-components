## Why

`/docs/components/:name` currently renders prose, an introspected props table,
and a single one-line usage string — for all 69 components, from one template.
It never shows the component. A reader evaluating `Button` cannot see that it
has five shape variants orthogonal to five colors and eight sizes; they can only
read that it does.

That is backwards for a component registry, where variants and compositions are
what a reader spends their time on. `/playground/<name>` does render a live
instance, but it carries no prose, shows one configuration at a time, and
nothing links the two.

## What Changes

- **Live examples on each component doc page.** Per-component example modules
  under `apps/web/src/pages/docs/examples/`, each declaring a set of named
  examples — typically a variant matrix, a size scale, and one or two real
  compositions — rendered live on the page.
- **Source for every example, guaranteed to be the source that rendered it.**
  Each example's code is read out of the example module's own compiled source
  via `include_str!` and a marked region, so the displayed snippet is the exact
  bytes the compiler saw. There is no sync step and therefore no staleness
  window.
- **A preview/code presentation** composed from the installed `Tabs`,
  `CopyButton`, and `Card`, with lightweight syntax highlighting.
- **An installation block** showing `adico add <name>`, with a copy button.
- **A link to the matching `/playground/<name>`** page. The 69 docs names and
  69 playground slugs are an exact set match, and today neither route links to
  the other.
- **Graceful fallback.** A component with no example module renders exactly what
  it renders today, so the page is never worse than before while the set fills
  in.

Non-goals: examples for all 69 components in this change (a first tranche, with
the rest behind the fallback); the guide pages (theming, spacing, typography,
dark mode, Tailwind) and the docs sidebar, which are their own change; any
change to `registry/ui/*.rs`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `adico-web-structure`: the docs route tree requirement gains the rule that a
  component's doc page renders the component itself — live, across its variants
  and compositions — with the displayed source for each example being the same
  source that produced it, plus its install command and a link to its
  playground page.

**Dependency:** stacks on `redesign-web-visual-foundation`, which already
carries a MODIFIED delta against this capability's docs-route-tree requirement.
Both stack on the unarchived `2026-09-24-merge-apps-into-web`, where
`adico-web-structure` currently lives. Validation passes with the delta in place
(verified for the previous change); archiving still requires the merge change to
land in `openspec/specs/` first.

## Impact

**Added**
- `apps/web/src/pages/docs/examples/mod.rs` — the `DocExampleMeta` type, the
  self-source extractor, and item → examples dispatch.
- `apps/web/src/pages/docs/examples/<item>.rs` — one per component in the first
  tranche.
- `apps/web/src/components/code_block.rs` — app-level wiring: a highlighted,
  copyable code block.
- `apps/web/src/components/doc_example.rs` — app-level wiring: the
  preview/code presentation.

**Modified**
- `apps/web/src/pages/docs/component.rs` — renders examples, install command,
  and the playground link.
- `apps/web/src/components/mod.rs`.

**Not touched**
- `registry/ui/*.rs`, `apps/web/src/components/ui/*`, `adico.lock`,
  `components.json`, `packages/*`.

**Risks**
- Example modules ship their own source text in the wasm bundle. This is the
  cost of the sync guarantee, and it is close to free: the snippet text would
  have shipped as a string or a JSON blob either way.
- A variant matrix renders many instances at once. For popup-family components
  (tooltip, popover, dropdown) this intersects a pre-existing bug where anchored
  floating content can stick at `visibility: hidden`; those examples stay
  closed-by-default and that bug is not in scope here.
