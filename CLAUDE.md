# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

This is the only agent-guidance file in this repository (no `AGENTS.md` /
`.agents/` — Claude Code only).

## Commands

Baseline validation, run from the repository root for every Rust/package change:

```sh
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings
cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask
openspec validate build-adico-component-ecosystem --strict
```

Single test / single package:

```sh
cargo test -p adico-cli <test_name>            # one test by name
cargo test -p adico-registry-core -- --exact <test_name>
```

`adico-xtask` commands (no npm/Node involved; run from repo root):

```sh
cargo run -p adico-xtask -- registry build       # regenerate registry/generated/* from registry/ui|hooks|lib source
cargo run -p adico-xtask -- registry validate     # check registry/generated/* isn't stale
cargo run -p adico-xtask -- provenance check      # verify provenance/records/* against UPSTREAMS.md obligations
cargo run -p adico-xtask -- catalog fetch <axis|all> [--revision <sha>]      # only network-touching command; refreshes statics/catalogs/<axis>.json (shadcn, base-ui, dioxus-components, dioxus-primitives)
cargo run -p adico-xtask -- primitive-compat sync|check|diff   # regenerate/verify statics/primitive_compatibility.json (vs Base UI + dioxus-primitives)
cargo run -p adico-xtask -- component-compat sync|check        # regenerate/verify statics/component_compatibility.json (vs shadcn + dioxus-components)
```

Surface-specific checks (only run when the change actually touches that surface — see `docs/validation.md` for the full matrix and current per-surface status):

```sh
cargo check --target wasm32-unknown-unknown -p <crate>   # web/wasm
cd tests/playwright && npm test                          # browser/keyboard/a11y (Playwright + axe)
dx serve                                                  # from an app/example dir; auto-compiles tailwind.css → assets/tailwind.css
```

Registry components are Dioxus-source, not a compiled dependency: never `cargo check`
a `registry/ui/*.rs` file in isolation — validate it through `registry validate`
and through an installed consumer (`examples/basic-spa`, `examples/basic-ssr`,
or a `tests/installation/*` fixture), since that's the only path real consumers use.

## Project overview

This repository is a Rust monorepo for Awesome Dioxus Components (`adico`), a
shadcn-style Dioxus component ecosystem. Its product boundaries are:

- `packages/adico-primitives`: owned reusable headless runtime behavior.
- `packages/adico-registry-core`: registry schemas, resolution, and installation planning.
- `packages/adico-cli`: the `adico` executable.
- `registry/`: source-owned component distribution content.
- `apps/docs`, `apps/playground`, and `examples/`: maintained Dioxus applications and consumer fixtures.

Read `docs/architecture.md` before changing boundaries and
`docs/development.md` before changing build or test workflows. Inspect the
package names in the Cargo manifests instead of assuming they have retained
template defaults.

## Architecture rules

`adico` is a source-distribution ecosystem, not a conventional styled
component crate — the consumer installs authored registry source into their
own project and owns the resulting files (shadcn's model, applied to Dioxus).
The dependency/data flow is one direction only:

```
Dioxus → adico-primitives → registry source + metadata → adico CLI/registry-core → consumer app
```

- Keep reusable behavior in `adico-primitives`, registry semantics in
  `adico-registry-core`, and consumer workflow in `adico-cli`.
- Keep styled components as understandable source under `registry/`; do not
  turn them into an opaque consumer-facing component crate. Never make a
  consumer example or fixture import `registry/` source via a workspace path
  — they must go through the real `adico` CLI installation, same as an
  external consumer would.
- Treat a feature as a vertical slice: registry metadata, dependency planning,
  CLI installation, copied component source, consumer fixture, and tests.
- Preserve separate Dioxus feature builds. Server-only dependencies must remain
  behind the `server` feature, and browser code must compile for
  `wasm32-unknown-unknown`. Browser-only interop belongs behind target-aware
  primitive adapters — copied component APIs must not leak browser-interop
  details to the consumer.
- The Tailwind CSS pipeline is per-project, not shared: each installed app has
  its own root `tailwind.css` (compiler input: `@import "tailwindcss"`,
  `@source`, and an `adico`-managed semantic-token/animation marker region)
  compiled to `assets/tailwind.css` (generated output, linked via
  `document::Stylesheet { href: asset!(...) }`), because Tailwind only emits
  the classes actually referenced in that project's own `src/`. `dx serve`/
  `dx build` compile this automatically using Tailwind's cached standalone
  native binary (`~/.dx/tools/`) — not the npm `@tailwindcss/cli` package.
- The official registry is embedded with the CLI; a consumer can additionally
  configure a named local or static-HTTPS organization registry in
  `components.json` and set it as default without forking the CLI.

## Spec-driven development (OpenSpec)

Use the OpenSpec workflow for new capabilities, user-visible behavior changes,
breaking contract or schema changes, and significant architecture work:

1. Explore uncertain ideas first when useful.
2. Create a proposal (`openspec propose` / hand-authored under
   `openspec/changes/<name>/`). Review and approve its proposal, design, delta
   specs, and tasks before implementation.
3. Implement approved tasks, following the architecture rules above.
4. Keep artifacts aligned with discoveries as you go — update the proposal,
   design, or tasks if reality diverges from the plan.
5. Run `openspec validate <change> --strict` and the project's validation
   commands (below).
6. Sync the accepted delta into `openspec/specs` and archive the completed
   change.

Existing behavior is documented in `openspec/specs`. Active work belongs in
`openspec/changes`; do not implement an active change whose planning artifacts
are incomplete or unapproved. Small fixes that restore already-specified
behavior may proceed directly, but must still be tested.

## Working conventions

- Use the pinned toolchain and committed `Cargo.lock`. Use `--locked` for
  validation commands.
- Add focused tests for behavior changes. API database tests use `#[sqlx::test]`
  only when a future approved API boundary introduces database behavior.
- Never commit `.env` files, credentials, tokens, production data, or generated
  build output.
- Keep the ecosystem focused. Add abstractions in response to a concrete need,
  not in anticipation of one.
- Update documentation and example environment values when commands,
  configuration, public contracts, or architecture change.

## Before finishing

Run the checks relevant to the changed surfaces — see `docs/development.md`
and `docs/validation.md` for the full command matrix (fmt, check, clippy,
test, registry/provenance validation, wasm32/desktop feature checks,
Playwright/browser suites). Always run formatting. Do not claim database,
WebAssembly, container, or mobile validation unless that check actually ran.
Report skipped checks and the reason.

## Use the project knowledge graph for codebase questions

This repository has a generated Understand Anything knowledge graph at
`.ua/knowledge-graph.json` (1200+ nodes covering every package, registry item,
primitive, test fixture, and doc; 9 architectural layers; a 12-step guided
tour). Prefer it over cold repo exploration whenever the question is about
architecture, file relationships, "what depends on X", "where is Y used", or
onboarding to an unfamiliar part of this monorepo:

- **`/understand-anything:understand-chat`** — ask questions about the
  codebase (e.g. "what depends on adico-registry-core", "which registry items
  are Dioxus-only extras", "how does `adico add` apply a plan").
- **`/understand-anything:understand-explain`** — deep-dive a specific file,
  function, or module (e.g. explain `packages/adico-cli/src/add.rs`).
- **`/understand-anything:understand-diff`** — analyze a git diff or PR against
  the graph to see affected components and risk.
- **`/understand-anything:understand-dashboard`** — launch the interactive
  visual graph explorer.
- **`/understand-anything:understand-onboard`** — generate an onboarding guide
  for a specific area.
- **`/understand-anything:understand`** — refresh the graph after significant
  structural changes (new packages, large migrations, moved directories). Only
  needed when the graph is stale; it is not run automatically on every commit
  in this repo (`autoUpdate` is not enabled).

Don't re-derive answers `grep`/`find` could get wrong when the graph already
has them structured and cross-referenced. But the graph can go stale (new
migration waves, new registry items, refactors) — if an answer from the graph
looks inconsistent with the current file tree or git history, trust the live
repository state and consider re-running `/understand-anything:understand`
rather than the stale snapshot.
