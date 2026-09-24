## Why

The repo now has three separate maintained Dioxus apps — `apps/home` (the
landing page, just added), `apps/docs`, and `apps/playground` — each its own
crate, its own `dx serve` port, and its own Tailwind pipeline. They cannot
share one URL space as three origins. `apps/home`'s own source already
hard-codes cross-app links to `/docs` and `/playground`
(`apps/home/src/routes.rs:31,34`, `apps/home/src/pages/index.rs:28-29`) —
dead today, but signaling the intended layout was always one site, not three.
Keeping them separate means three containers and three shells that will drift
from each other for what is conceptually a single public website.

## What Changes

- **BREAKING** (internal, pre-`0.1.0` structure only — no published contract
  changes): `apps/home`, `apps/docs`, and `apps/playground` are deleted as
  separate crates. A single new crate, `apps/web` (package `adico-web`),
  serves the whole site from one Dioxus router: `/` (landing, from
  `apps/home`), `/docs/*` (from `apps/docs`, restyled onto Tailwind and
  installed registry components), `/playground/*` (from `apps/playground`'s
  72 routes, prefixed).
- `apps/web` is created via `git mv apps/playground apps/web` (the largest,
  only spec-governed tree) with home's and docs' content folded in — not
  rebuilt from scratch.
- Root `Cargo.toml` workspace members collapse from three app entries to one.
- `apps/docs`'s `server`/SSR feature is dropped entirely (confirmed dead
  code: never built, tested, or deployed).
- `apps/web`'s `components.json`/`adico.lock` are playground's as-is
  (`kind: "embedded"`) — not home's `https` entry, which points at an
  unresolvable domain. No new `adico add` is needed: every one of home's 12
  installed items is already a byte-identical duplicate inside playground's
  72-item set. While touching the lock file, a pre-existing stale checksum
  for `@adico/theme-switcher` is fixed.
- `apps/docs`'s hardcoded `DOCS_STYLE` (raw CSS forcing a dark background
  sitewide, no light/dark mode) is replaced with Tailwind + already-installed
  registry components (`Card`, `Table`, `Badge`) — required by the merge
  itself, since in one document that CSS would break light mode everywhere,
  not just on docs routes.
- Two now-dangerous dead demo hrefs in playground's `breadcrumb.rs` and
  `responsive_flow.rs` (`/components`, `/components/registry`) are
  neutralized to `#`, since post-merge those paths would collide with docs'
  new component-detail routes.
- Exactly 27 Playwright `goto()` call sites across 6 spec files gain a
  `/playground` prefix; every other `goto("/")` in the suite targets
  `examples/basic-spa`/`basic-ssr`/`tests/installation/*` fixtures and is
  untouched.
- Documentation referencing the three old app paths (`apps/README.md`, root
  `README.md`, `docs/ci-cd.md`, `docs/architecture.md`,
  `docs/development.md`, `docs/validation.md`, `CLAUDE.md`) is updated.

## Capabilities

### New Capabilities
- `adico-web-structure`: governs the merged `apps/web` app — router
  structure, per-route-tree layout composition, the real-CLI-installation
  convention, the per-project Tailwind pipeline, and the docs route tree's
  registry-component composition. Supersedes both capabilities below.

### Modified Capabilities
- `adico-playground-structure`: all requirements removed — the standalone
  `apps/playground` crate this capability described ceases to exist; its
  behavior is carried forward, adapted, in `adico-web-structure`.
- `adico-home-structure`: all requirements removed — the standalone
  `apps/home` crate this capability described ceases to exist; its behavior
  is carried forward, adapted, in `adico-web-structure`.

## Impact

- **Code**: `apps/playground/**` moves to `apps/web/**` (git history
  preserved via `git mv`); `apps/docs/**` and `apps/home/**` are folded in and
  deleted; root `Cargo.toml` workspace members; `apps/web/Cargo.toml`,
  `Dioxus.toml`, `components.json`, `adico.lock`.
- **Tests**: `tests/playwright/*.spec.ts` (6 files, 27 call sites) and
  `tests/playwright/README.md`.
- **Docs**: `apps/README.md`, root `README.md`, `docs/ci-cd.md`,
  `docs/architecture.md`, `docs/development.md`, `docs/validation.md`,
  `CLAUDE.md`.
- **No changes** to `packages/adico-cli`, `packages/adico-primitives`,
  `packages/adico-registry-core`, `registry/ui/*.rs`, or `examples/*` — the
  merge is confined to the three app crates and their governing specs.
- **Non-goals**: no infra/deployment work (separate `awwwkshay-infra` effort);
  no new registry components; no `registry/ui/*.rs` changes.
