## Why

`apps/web` is the project's shop window — a developer evaluating adico judges the
registry by the site that demonstrates it. Today that site loads **no typeface at
all** (no `@font-face`, no font `<link>`, no `--font-*` token), so every page
renders in the browser's default UI sans while the registry it advertises is a
shadcn-style component library. Alongside that, three concrete defects are
visible on the running app: literal backticks render as text in registry prose,
`/docs` card descriptions overflow their card border, and literal `dark:`
utilities follow the operating system's theme instead of the app's own
`ModeToggle`.

This change lays the visual foundation — typography, type scale, and those three
fixes — before any docs restructuring, because the missing typeface is the single
largest reason the site reads as unfinished.

## What Changes

- **Self-hosted typeface.** Add a UI/display face and a mono face as
  `@font-face` rules over local files under `apps/web/assets/fonts/`, with
  `font-display: swap` and a real fallback stack. Self-hosted rather than a CDN
  link so the site has no third-party request and stays offline-capable.
- **Font and type-scale tokens.** Declare `--font-sans`, `--font-mono`, and
  `--font-display` plus a small heading/lead/body/caption scale. All of it is
  written **above** `/* adico:theme:start */` in `apps/web/tailwind.css`, because
  the CLI regenerates the marker region wholesale on every `adico add` and would
  otherwise destroy it.
- **Fix: `dark:` utilities track the theme toggle, not the OS.** Add
  `@custom-variant dark (&:is(.dark *))` above the marker region. Today the repo
  declares no such variant, so Tailwind v4's default applies and
  `apps/web/assets/tailwind.css:3490-3492` compiles `dark:hover:bg-accent/50`
  into `@media (prefers-color-scheme: dark)`. A visitor whose OS theme disagrees
  with their chosen app theme sees wrong hover, ring, and border colors.
- **Fix: inline code renders as code.** Registry `documentation` prose uses
  markdown inline-code backticks; `apps/web` renders them raw. Add an
  app-level prose wiring component that splits on backticks and emits a styled
  `<code>`, used by the docs prose blocks and the landing hero.
- **Fix: `/docs` card descriptions overflow their card.** The longest registry
  descriptions (`bubble`, `message`, `message-scroller`) run past the card
  border at desktop width.
- **Landing page layout polish.** Containerize the hero (it is currently pinned
  left with a large dead zone), give the "Why adico" and "Install" card grids
  equal-height rows, stop truncating install commands mid-command, and derive
  the component and primitive counts from the embedded registry manifest rather
  than hardcoded literals that silently go stale.

Non-goals, deferred to later changes in this sequence: docs example galleries
with live previews and per-component variants; the theming/spacing/typography/
dark-mode/Tailwind guide pages; the docs sidebar; and playground shell work. No
`registry/ui/*.rs` file is modified, and no new route is added.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `adico-web-structure`: the capability that governs `apps/web`'s shell, landing
  page, docs route tree, and Tailwind pipeline. Two requirements change
  behavior:
  - the per-project Tailwind pipeline requirement gains the rule that app-owned
    CSS (fonts, type scale, custom variants) lives **outside** the
    `adico:theme:start`/`adico:theme:end` markers, and that the app declares the
    `dark` custom variant so class-based theme switching governs literal `dark:`
    utilities;
  - the docs route tree requirement gains the rule that registry prose
    containing markdown inline code renders as code rather than literal
    backticks, and that long registry descriptions wrap within their container.

**Dependency, stated explicitly:** `adico-web-structure` is not yet present in
`openspec/specs/`. It is an ADDED-only delta inside the active, unarchived change
`2026-09-24-merge-apps-into-web`, which is 32/33 complete and blocked on its task
9.8 (three Playwright failures root-caused to one-shot `onmounted` geometry
measurement in `registry/ui/carousel.rs` and `registry/ui/time_picker.rs` — out
of that change's declared scope). This change's delta therefore targets a
capability whose base lands in `openspec/specs/` only when that change is
archived. There is repo precedent for declaring such a dependency in prose
(`archive/2026-08-30-playground-uses-registry-theme-and-sidebar`). Task 1 of this
change verifies `openspec validate --all --strict` still passes with the delta in
place, and falls back rather than forcing it if not.

## Impact

**Modified**
- `apps/web/tailwind.css` — new content **above line 4 only**; the marker region
  and everything after it is left byte-identical.
- `apps/web/src/pages/index.rs` — hero container, grid alignment, install
  command display, counts read from the manifest.
- `apps/web/src/pages/docs/index.rs` — card description wrapping.
- `apps/web/src/pages/docs/component.rs` — prose blocks routed through the new
  prose component.
- `apps/web/src/components/mod.rs` — register the new module.

**Added**
- `apps/web/assets/fonts/*` — self-hosted font files (binary assets, licensed
  under the OFL or equivalent; the license file ships alongside them).
- `apps/web/src/components/prose.rs` — app-level inline-code prose wiring.

**Not touched**
- `registry/ui/*.rs`, `registry/lib/*.rs` — no registry component is modified
  for the app's convenience.
- `apps/web/src/components/ui/*`, `apps/web/src/adico_lib/*`, `components.json`,
  `adico.lock` — all CLI-owned.
- `packages/*` — no CLI or primitive change here. The durable fix for the `dark`
  custom variant belongs in `packages/adico-cli/src/css.rs::theme_region()` so
  every consumer inherits it; that is filed as its own change, and the app-level
  declaration is the interim.

**Risk: one visible behavior change.** Declaring the `dark` custom variant
changes rendering for any visitor whose OS theme and app theme disagree — that
is the point of the fix, but it is a sitewide visual change, not a no-op.

**Risk: asset weight.** Self-hosted fonts add binary files to the repo and to
the served bundle. Subset to latin, ship variable or a minimal weight set, and
use `woff2` only.

**Validation surfaces:** `cargo check --locked -p adico-web`, the same on
`--target wasm32-unknown-unknown`, `adico css build` + `adico css check` from
`apps/web`, and the app-shell Playwright suites. The `h-dvh` definite-height
contract in `apps/web/src/main.rs` must be left intact —
`tests/playwright/playground-resizable-split.spec.ts` is its regression guard.
