## Context

See proposal.md - Why. Three facts drive the technical approach:

- `apps/playground` is the largest tree (72 routes, ~72 installed registry
  items) and the only one with a governing spec today; `apps/home` is a
  single route just added this session; `apps/docs` is 3 files.
- All of home's 12 installed items are byte-identical duplicates of items
  already in playground's 72-item `adico.lock` — confirmed by checksum
  comparison during research. No new `adico add` is required to cover the
  union.
- `apps/docs`'s only non-mechanical content is two page components (a
  component list, a props table) reading two `include_str!` JSON files.

## Goals / Non-Goals

**Goals:**
- One crate, one `dx serve` process, one compiled `assets/tailwind.css`,
  serving `/`, `/docs/*`, `/playground/*`.
- Preserve every behavior `adico-playground-structure` and
  `adico-home-structure` already guarantee, under the new paths.
- Fix docs' global-CSS defect as part of the same change, since it becomes
  sitewide once merged (see proposal.md - What Changes).

**Non-Goals:**
- No infra/deployment work (separate `awwwkshay-infra` effort).
- No new registry components, no `registry/ui/*.rs` changes.
- No redesign of playground's demo/control panel system — that machinery
  moves as-is.

## Decisions

### `apps/web` is `git mv apps/playground apps/web`, not a fresh scaffold
Moving the largest, spec-governed, currently-passing tree preserves its git
history and means home's and docs' much smaller trees are the ones folded
in, not the reverse. Alternative considered: build `apps/web` fresh and
`adico add` everything in. Rejected — it would re-run `adico add` for items
already installed identically in playground's lock file, and lose
playground's git blame.

### One flat `Route` enum, two layout scopes
```
#[layout(SiteLayout)]
  #[route("/")]                      Home {}
  #[route("/docs")]                  DocsIndex {}
  #[route("/docs/components/:name")] DocsComponent { name: String }
  #[layout(PlaygroundLayout)]
    #[route("/playground")]          PlaygroundIndex {}
    #[route("/playground/button")]   Button {}          // ×70, unchanged page components
  #[end_layout]
#[end_layout]
#[route("/responsive/flow")]         ResponsiveFlow {}
#[route("/responsive/overlay")]      ResponsiveOverlay { case: ... }
```
The two harness routes sit at the top level, outside `#[layout(SiteLayout)]`
entirely — not just outside `PlaygroundLayout` — per
`adico-web-structure`'s "Shell-free harness routes stay outside every
layout" requirement.

**Prerequisite check before implementation touches routes.rs**: confirm
dioxus-router renders nested `#[layout(...)]` scopes as nested `Outlet`s
(one inside the other), not parallel siblings. Playground's existing
single-`Outlet` invariant is load-bearing for portalled popup positioning
(Dialog/Popover/Select), and a known pre-existing bug already causes
anchored floating content to occasionally get stuck `visibility:hidden` —
this merge must not multiply that surface. If dioxus-router produces
parallel Outlets for nested layouts, fall back to one flat `SiteLayout`
that switches its own chrome by matching the current route, keeping a
single Outlet overall.

### `components.json`/`adico.lock`: playground's, unmodified
Playground's is `"kind": "embedded"`; home's is `"kind": "https"` pointing
at `adico.awwwkshay.com`, confirmed unresolvable. Taking playground's avoids
every `adico add` printing the offline-fallback warning. One incidental fix
while the lock file is being touched anyway: a stale checksum for
`@adico/theme-switcher` (found during research) is corrected to match the
installed file's real sha256.

### Docs restyle is in-scope, not deferred
`DOCS_STYLE` sets `body { background: #0a0a0f }` unconditionally. Merged
into one document, that stops being "docs looks different" and starts being
"light mode is broken everywhere, including the landing page." Its two page
components (component list, props table) are replaced with Tailwind +
already-installed `Card`/`Table`/`Badge`. The `server`/SSR feature is
dropped in the same pass — confirmed dead code (never built, tested, or
deployed; the `serve` and `launch` calls are both reachable in source order
under `--features server`, proof it never actually ran).

### Now-dangerous dead hrefs get neutralized
`breadcrumb.rs:35` and `responsive_flow.rs:83,87` hardcode `/components` and
`/components/registry` as demo link targets. Harmless as dead links today;
after the merge, `/components/registry` would resolve to whatever docs
route names a component "registry" — an unintended collision. Both become
`#`.

### Playwright: prefix only the 27 sites that actually target playground
Confirmed by grepping every `goto(` call in `tests/playwright/*.spec.ts`:
27 sites across `playground-drag-and-drop-list.spec.ts`,
`playground-resizable-split.spec.ts`, `playground-time-picker.spec.ts`,
`playground-enriched-demos.spec.ts` need `/playground` prepended. The
`/responsive/*` sites (in `responsive.spec.ts`, `responsive-desktop.spec.ts`)
stay unprefixed — those routes remain top-level. Every other `goto("/")`
targets `examples/basic-spa`, `basic-ssr`, or `tests/installation/*`
fixtures via `ADICO_PLAYWRIGHT_BASE_URL` pointed at a *different* dev
server — untouched by this merge. `playwright.config.ts`'s `baseURL` cannot
absorb a shared prefix itself, since Playwright resolves a leading-slash
`goto` against the origin, discarding any base path.

## Risks / Trade-offs

- **[Risk]** Nested layouts could produce parallel Outlets, breaking
  portalled-popup positioning. → Mitigation: verified before route-tree
  implementation begins (see the Decisions entry above); flat-`SiteLayout`
  fallback identified in advance.
- **[Risk]** `include_str!` paths in docs' data loading
  (`registry/registry.json`, `statics/component_props.json`) are relative
  to the *source file's* location; moving that logic one directory deeper
  (`apps/web/src/pages/docs/*.rs` vs. `apps/docs/src/main.rs`) needs one
  more `../` each. → Mitigation: both targets are CI-guarded
  (`registry validate`-style checks), so a wrong path fails the build
  loudly, not silently.
- **[Risk]** Docs' restyle could introduce new registry-composition defects
  under `adico-web-structure`'s new docs requirement. → Mitigation: docs'
  actual content is small (two page components), and the same
  registry-components-only constraint already governs the rest of the app.
- **[Trade-off]** Folding docs' restyle into this change enlarges its scope
  beyond a pure structural merge. Accepted because the alternative (merge
  now, restyle later) ships a sitewide light-mode regression in between.

## Migration Plan

1. `git mv apps/playground apps/web`; rename package/crate to `adico-web`.
2. Fold `apps/home`'s and `apps/docs`'s page/component source into
   `apps/web`; delete the two now-empty directories.
3. Update root `Cargo.toml` workspace members (three app entries → one).
4. Implement the route tree per the Decisions above, verifying the Outlet
   nesting behavior first.
5. Restyle docs onto Tailwind + registry components; drop the `server`
   feature.
6. Fix the two dead hrefs and the stale `theme-switcher` checksum.
7. Update the 27 Playwright call sites and `tests/playwright/README.md`.
8. Update cross-referencing docs (`apps/README.md`, root `README.md`,
   `docs/ci-cd.md`, `docs/architecture.md`, `docs/development.md`,
   `docs/validation.md`, `CLAUDE.md`).
9. Run full verification (see tasks.md and proposal.md - Impact).

No rollback beyond `git revert` is needed — pre-`0.1.0`, nothing is deployed
that depends on the old three-app layout.
