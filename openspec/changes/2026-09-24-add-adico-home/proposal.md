## Why

The workspace declares `homepage = "https://adico.awwwkshay.com"`, but
nothing in the repo serves that domain root today — `apps/docs` and
`apps/playground` exist as maintained Dioxus apps, yet neither is a landing
page, and the root `README.md`'s own "Project status" section flags this
explicitly: "no hosted docs or registry site" is a named gap toward `v0.1.0`.
A visitor to the bare domain has nowhere to land before reaching `/docs` or
`/playground`. This change builds that landing page app now, ahead of and
independent from the (separate, later) work of actually deploying and
hosting it.

## What Changes

- Add a new maintained Dioxus application, `apps/home` (package
  `adico-home`), registered as a workspace member alongside `apps/docs` and
  `apps/playground`.
- Scaffold it on `apps/playground`'s conventions (the living, currently
  followed pattern), not `apps/docs` (a stylistically orphaned, pre-Tailwind
  artifact with hardcoded CSS and no theme tokens): explicit pinned `dioxus`
  dependency, per-project Tailwind pipeline, and — critically — initialized
  and kept current through the real `adico` CLI (`adico init`, `adico add`),
  never hand-authored component source.
- A single-route (`/`) landing page: hero with an install command (via the
  installed `CopyButton` component) and CTA links to `/docs`, `/playground`,
  and the GitHub repo; a feature-highlights section stating real facts (69
  components, 67 primitives, dual `MIT OR Apache-2.0` license); an install
  section surfacing the three real release channels shipped in `v0.1.0`
  (Homebrew tap, GitHub release binaries, `cargo install --git`).
- The page shell composes real installed registry components (button, card,
  navigation-menu, copy-button, mode-toggle, theme-switcher, etc.) rather
  than app-specific reimplementations.
- Update `apps/README.md` and root `README.md` (workspace table, and the
  "Project status" gap line — the landing page now exists in-repo, but is
  not yet hosted/deployed, so the wording must not claim it's live).

**Non-goals** (explicitly out of scope for this change):
- No shared header/nav component extracted across `apps/docs`,
  `apps/playground`, and `apps/home`. No such shared piece exists today;
  `apps/home` gets its own shell, the same way `apps/playground` has its
  own. Building a shared piece is a separate, larger decision (a registry
  `block`-style item or a new workspace crate).
- No changes to `apps/docs` (not modernized onto Tailwind or registry
  components, not restyled).
- No deployment or infra work of any kind: no Dockerfile, no `cd.yml`
  changes, no changes to the separate `awwwkshay-infra` repository
  (Kubernetes manifests, ArgoCD, Terraform DNS). That is a separate, later
  effort once this app exists and builds.

## Capabilities

### New Capabilities
- `adico-home-structure`: routing, shell, and CLI-installation conventions
  for the new `apps/home` landing page app — router location, single-page
  scope, CLI-managed component source, per-project Tailwind pipeline, and
  the "shell composes real registry components" principle, mirroring what
  `adico-playground-structure` already states for `apps/playground` but
  scoped to `apps/home`.

### Modified Capabilities
(none — no existing capability's requirements change)

## Impact

- **New code**: `apps/home/` (Cargo.toml, Dioxus.toml, `components.json`,
  `adico.lock`, CLI-installed `src/components/ui/*`, `src/adico_lib/*`,
  `src/routes.rs`, `src/pages/index.rs`, `src/main.rs`, `tailwind.css`,
  committed `assets/tailwind.css`, `assets/web/*` favicons).
- **Modified files**: root `Cargo.toml` (`[workspace].members`),
  `apps/README.md`, root `README.md`.
- **CI**: no new CI job in this change; `cargo check --locked --workspace`
  already covers all workspace members, so `apps/home` is automatically
  included once registered. No wasm32/Playwright job exists for apps today,
  so none is added here either — consistent with current per-surface CI
  scope.
- **Dependencies**: no new external dependencies beyond what
  `apps/playground` already uses (`dioxus`, `adico-primitives`, pinned
  `web-sys`, `time` if a component needs it) — nothing enters the workspace
  that isn't already present elsewhere.
- **Deployment/runtime**: none. This change produces a buildable,
  `dx serve`-able app; it does not deploy, containerize, or host anything.
