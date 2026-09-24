## Why

`apps/web` documents 69 components and nothing else. There is no page explaining
how to install adico into a project, how the theme tokens work, how light and
dark mode are switched, or how the Tailwind pipeline is wired — and that last
one is the step most likely to leave a new user staring at unstyled semantic
HTML, because `adico init` / `adico add` does not wire the root `tailwind.css`
or the `document::Stylesheet` link automatically.

Today the only written account of that lives in `apps/web/README.md`, a file a
visitor to the site never sees. The `ThemeBuilder` — which exposes all 34
editable theme tokens — is mounted only under `/playground/*`, so a reader can't
even reach it while reading about theming.

## What Changes

- **Six guide pages** under `/docs`: installation, theming, typography, spacing,
  dark mode, and Tailwind-with-Dioxus.
- **A docs sidebar** listing guides and components, composed from the installed
  `Sidebar` family, on a nested layout so `/docs/*` gains it without touching
  the landing or playground shells.
- **The theming page reads the project's real installed tokens** rather than a
  hand-maintained list, so it cannot drift from what `adico add` actually wrote
  into `tailwind.css`.
- **A live `ThemeBuilder` on the theming page**, so a reader can change the
  theme while reading about it and watch the page they are on respond.
- `/docs` gains a guides section, so the guides are reachable at every viewport
  width, including where the sidebar is hidden.

Non-goals: no new registry component; no change to `registry/ui/*.rs`; no change
to the playground shell or to the landing page; no search.

## Capabilities

### Modified Capabilities

- `adico-web-structure`: the docs route tree requirement gains the conceptual
  guide routes, the rule that documented theme tokens are derived from the
  project's own installed stylesheet rather than restated by hand, and the rule
  that the docs navigation shell is composed from installed registry components
  and does not disturb the landing or playground shells.

**Dependency:** stacks on `docs-component-examples`, which stacks on
`redesign-web-visual-foundation`. All three carry a MODIFIED delta against the
same `adico-web-structure` requirement, so they archive in that order, after
`2026-09-24-merge-apps-into-web` lands the base capability in `openspec/specs/`.

## Impact

**Added**
- `apps/web/src/pages/docs/guides/` — one file per guide page.
- `apps/web/src/pages/docs/tokens.rs` — compile-time parse of the installed
  `tailwind.css` token blocks.
- `apps/web/src/components/docs_nav.rs` — the docs sidebar nav list.

**Modified**
- `apps/web/src/routes.rs` — six new routes and a `DocsLayout` nested inside
  `SiteLayout`.
- `apps/web/src/pages/docs/index.rs` — guides section.
- `apps/web/src/pages/docs/mod.rs`, `apps/web/src/components/mod.rs`.

**Not touched**
- `registry/ui/*.rs`, `apps/web/src/components/ui/*`, `adico.lock`,
  `components.json`, `packages/*`, `PlaygroundLayout`, the landing page.

**Risks**
- `routes.rs` gains a second nested layout scope inside `SiteLayout`. The
  playground's own nested layout and the two shell-free harness routes must keep
  their exact current nesting; `main.rs`'s `h-dvh` definite-height contract and
  the playground's resizable splits are the regression surface.
- A sticky sidebar must scroll against `main`, not the document — the document
  does not scroll in this app.
