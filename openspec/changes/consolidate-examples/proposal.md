## Why

`examples/` has grown to seven workspace members, but only three
(`basic`, `fullstack`, `desktop`) are real CLI-installed consumer fixtures;
`dashboard`, `forms`, `kitchen-sink`, and `web` are unwired `dioxus::launch`
stubs with no `adico.lock`/`components.json` and no registry components
installed. This inflates the workspace and CI surface without adding
validation evidence. adico needs exactly two example fixtures — one SPA
(web-only) and one SSR/hydration fixture — that stay CLI-installed and keep
carrying real parity evidence, so the example surface matches what is
actually validated today instead of the enlarged catalog planned in
`build-adico-component-ecosystem`.

## What Changes

- Rename `examples/basic` to `examples/basic-spa` (crate `adico-example-basic-spa`),
  preserving its CLI-installed, wasm32 web-only consumer-fixture role.
- Rename `examples/fullstack` to `examples/basic-ssr` (crate `adico-example-basic-ssr`),
  preserving its CLI-installed, server+web dual-feature SSR/hydration role.
- Delete `examples/dashboard`, `examples/forms`, `examples/kitchen-sink`, and
  `examples/web`: unwired placeholder crates with no installed components and
  no parity evidence.
- **BREAKING**: Delete `examples/desktop` (crate `adico-example-desktop`),
  the only fixture backing Button's `desktop` parity dimension
  (`parity.json`: `passed: true`, evidence `examples/desktop`). This removes
  the repository's only native-desktop-target build evidence. Button's
  `desktop` dimension moves to `passed: false` with a note recording the
  fixture's removal and the dimension as an explicit, named gap rather than a
  silent drop — consistent with this project's existing "record unavailable
  checks explicitly" convention (see `docs/adico/m2-vertical-slice.md`,
  `parity.json`).
- Update `Cargo.toml` workspace members, `parity.json` evidence paths (drop
  `examples/desktop`; rename `examples/basic`/`examples/fullstack` references
  to `examples/basic-spa`/`examples/basic-ssr`), `scripts/refresh-basic-example.sh`,
  `tests/playwright/README.md` and `fullstack.spec.ts` comments,
  `docs/adico/m2-vertical-slice.md`, and `docs/validation.md` to match the
  renamed/removed fixtures.
- Amend the still-open `build-adico-component-ecosystem` change (tasks.md
  items 4.7, 6.4, 8.5, 9.2, 9.4, 9.5, 10.4, 11.3 and design.md §10), which
  currently names `kitchen-sink`, `dashboard`, `forms`, and `web` examples as
  required future validation surfaces, so its remaining milestones point at
  the two surviving fixtures (plus `tests/installation/*`) instead of
  directories this change deletes.

## Capabilities

### New Capabilities

- `adico-example-fixtures`: Defines the fixed set of consumer-style example
  fixtures under `examples/` — exactly `basic-spa` and `basic-ssr` — and the
  CLI-installed, evidence-bearing role each one plays.

### Modified Capabilities

- None. No existing `openspec/specs/*` capability currently governs the
  `examples/` directory's membership or fixture roles.

## Impact

- Affected code: root `Cargo.toml` workspace members; `examples/basic` →
  `examples/basic-spa`; `examples/fullstack` → `examples/basic-ssr`; deletion
  of `examples/dashboard`, `examples/desktop`, `examples/forms`,
  `examples/kitchen-sink`, `examples/web`; `Cargo.lock` regeneration.
- Affected config/evidence: `parity.json` (button's `desktop`, `examples`,
  `web`, `ssrHydration` dimensions; dialog's and select's `examples` `web`
  and `ssrHydration` evidence paths).
- Affected docs/scripts: `scripts/refresh-basic-example.sh`,
  `tests/playwright/README.md`, `tests/playwright/fullstack.spec.ts` (comment
  only), `docs/adico/m2-vertical-slice.md`, `docs/validation.md`.
- Affected planning: the open `openspec/changes/build-adico-component-ecosystem`
  change (tasks.md and design.md §10) references the deleted example
  directories as future validation surfaces for M4–M11 milestones; those
  references must be re-pointed, not silently orphaned.
- No database changes. No deployment/CI workflow changes are expected beyond
  the workspace member list, since `.github/workflows/ci.yml` operates on
  `cargo ... --workspace` rather than naming individual example crates.
- Native desktop-target build validation has no fixture after this change;
  it is recorded as a named gap, not reintroduced elsewhere, per explicit
  scope ("for now" — two examples only).
