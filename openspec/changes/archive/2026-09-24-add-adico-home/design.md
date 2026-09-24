## Context

See `proposal.md` - Why. Two existing maintained apps set precedent, but
pull in opposite directions:

- `apps/docs`: pre-Tailwind, hand-written inline `<style>` CSS, no theme
  tokens, no registry components, `include_str!`s generated JSON at compile
  time. Predates the current conventions; not touched by this change.
- `apps/playground`: the living pattern — CLI-installed (`adico init`/
  `adico add`), per-project Tailwind pipeline, `routes.rs`/`pages/` split,
  shell composed from real registry components. Confirmed via direct
  inspection this session: neither app imports `registry/` source through a
  workspace path; `adico-primitives` is a version requirement resolved only
  through the root `[patch.crates-io]` override, identical to how an
  external consumer would depend on it once published.

`apps/home` follows `apps/playground`'s pattern. This was validated
end-to-end this session against a disposable scratch app (`adico init` →
`adico add` → real component source, `adico.lock`, and `components.json`
produced by the CLI, compiling clean for `wasm32-unknown-unknown`), so the
mechanics are proven, not assumed.

## Goals / Non-Goals

**Goals:**
- A landing page that is structurally indistinguishable, in how its source
  came to exist, from any other adico consumer project — CLI-installed, not
  hand-authored component source.
- A single route for now (`/`). The spec's `routes.rs`/`pages/` split exists
  so adding a second route later (e.g. a changelog or blog index) is "add
  one file," not a restructure.

**Non-Goals:**
- No shared header/nav crate or registry `block` item across
  `apps/docs`/`apps/playground`/`apps/home` — see proposal.md's Non-goals.
  `apps/home` gets its own `Layout`, same as playground has its own.
- No SSR for `apps/home`. It uses the `web` platform only, same as
  `apps/playground` and `examples/basic-spa` — a landing page has no
  server-rendering requirement, and adding one would be new architectural
  surface this change doesn't need.
- No deployment, containerization, or hosting — see proposal.md.

## Decisions

**Model on `apps/playground`'s Cargo.toml shape exactly, not
`.workspace = true`.** `adico add`'s Cargo editor (in
`packages/adico-registry-core`) does not parse workspace-inherited
dependency tables, only plain strings/inline tables — confirmed by the
comment already in `apps/playground/Cargo.toml`. Any app that will receive
`adico add` must declare `dioxus` explicitly. Alternative considered:
`dioxus.workspace = true` like `apps/docs` uses — rejected, since `apps/home`
must support `adico add` for its own components, and docs's dependency form
would break the CLI's editor the first time a component is added.

**Single-page scope now, not a full site IA.** The proposal's content (hero,
feature highlights, install channels) fits one page. Alternative considered:
pre-building a multi-page structure (e.g. separate `/features`, `/install`
routes) — rejected as premature; the `routes.rs`/`pages/` spec requirement
already makes adding routes later a small, additive change, so there's no
cost to deferring.

**Reuse `apps/playground`'s favicon/manifest asset set under
`apps/home/assets/web/`** rather than commissioning new brand assets. Both
apps represent the same product (`adico`); a new landing page shouldn't ship
a mismatched icon. If a future rebrand happens, both apps update together.

**Package name `adico-home`, directory `apps/home`** (unprefixed directory,
prefixed package name) — matches the existing `playground`/`adico-playground`
pairing exactly, not `docs`/`adico-docs`'s pairing, which is the same rule
applied consistently (the directory name is never itself prefixed).

## Risks / Trade-offs

- **[Risk] `/docs` and `/playground` links are dead when `apps/home` runs
  standalone via `dx serve`**, since each app is its own dev server on its
  own port. → **Mitigation**: accepted, documented trade-off (see
  proposal.md's Non-goals and this session's earlier scoping decision) — it
  resolves once path-based routing behind one host exists, which is
  deliberately separate, later work. Not fixed by, and not blocking, this
  change.
- **[Risk] Divergence between `apps/home` and `apps/playground`'s shells
  over time** (e.g. one updates to a new `mode-toggle` API, the other
  doesn't), since there's no shared header component per the Non-Goals
  above. → **Mitigation**: both are thin, CLI-installed compositions over
  the same registry components; a registry-level fix (per the "shell
  composes real registry components" requirement) propagates to both the
  next time each app runs `adico add`/`adico update`-equivalent flows. If
  divergence becomes a real maintenance cost, extracting a shared piece is
  the documented future option, not a hidden retrofit.

## Open Questions

None — the two decisions that would have changed scope (URL layout: paths
vs. subdomains; which app owns the root) were already resolved in
conversation before this change was proposed.
