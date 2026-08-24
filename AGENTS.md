# Repository instructions for coding agents

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
package names in the Cargo manifests instead of assuming they have retained the
template defaults.

## Architecture rules

- Keep reusable behavior in `adico-primitives`, registry semantics in
  `adico-registry-core`, and consumer workflow in `adico-cli`.
- Keep styled components as understandable source under `registry/`; do not
  turn them into an opaque consumer-facing component crate.
- Treat a feature as a vertical slice: registry metadata, dependency planning,
  CLI installation, copied component source, consumer fixture, and tests.
- Preserve separate Dioxus feature builds. Server-only dependencies must remain
  behind the `server` feature, and browser code must compile for
  `wasm32-unknown-unknown`.

## Agent Skills

Skills use the open `SKILL.md` format and live under `.agents/skills`. Load the
matching skill before starting one of these tasks, even when the current agent
does not discover repository skills automatically:

- Feature work: `.agents/skills/implement-fullstack-feature/SKILL.md`
- Testing, CI reproduction, or pre-handoff validation:
  `.agents/skills/validate-fullstack-project/SKILL.md`

OpenSpec workflow skills are also installed in `.agents/skills/openspec-*` and
are managed by the OpenSpec CLI. Do not hand-edit generated OpenSpec skills; run
`openspec update` after upgrading the CLI.

## Spec-driven development

Use the OpenSpec workflow for new capabilities, user-visible behavior changes,
breaking contract or schema changes, and significant architecture work:

1. Explore uncertain ideas with the `openspec-explore` skill when useful.
2. Create a proposal with `openspec-propose`. Review and approve its proposal,
   design, delta specs, and tasks before implementation.
3. Implement approved tasks with `openspec-apply-change`, loading the relevant
   project skill above for stack-specific guidance.
4. Keep artifacts aligned with discoveries by using `openspec-update-change`.
5. Run `openspec validate <change>` and the project validation skill.
6. Sync the accepted delta into `openspec/specs` and archive the completed change.

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

Load `.agents/skills/validate-fullstack-project/SKILL.md` and run the checks
relevant to the changed surfaces. Always run formatting. Do not claim database,
WebAssembly, container, or mobile validation unless that check actually ran.
Report skipped checks and the reason.
