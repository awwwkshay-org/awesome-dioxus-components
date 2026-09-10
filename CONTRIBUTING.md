# Contributing to adico

Thanks for your interest in contributing. This project follows a
spec-driven workflow that's a bit more structured than a typical "open a PR"
repo — read this before you start, since a PR that skips it will get sent
back for one that doesn't.

## Setup

```sh
git clone https://github.com/awwwkshay-org/awesome-dioxus-components
cd awesome-dioxus-components
rustup show   # installs the pinned toolchain from rust-toolchain.toml
cargo check --locked --workspace
```

You'll also want [OpenSpec](https://github.com/Fission-AI/OpenSpec) installed
(`npm install -g @fission-ai/openspec` or see its own install docs) — this
repo's proposal/spec workflow depends on it.

## Before you write code: is this a spec-driven change?

Use the OpenSpec workflow (`openspec propose` → review → implement →
`openspec validate <change> --strict` → sync the delta into `openspec/specs`
→ archive) for:

- new capabilities
- user-visible behavior changes
- breaking contract or schema changes (registry format, `components.json`,
  CLI flags)
- significant architecture work (crate boundaries, dependency direction)

**Skip the workflow only for small fixes that restore already-specified
behavior** — a bug fix that makes existing behavior match its own spec, not a
new decision. Those still need tests.

`openspec/specs/` documents current behavior; `openspec/changes/` holds
in-flight proposals. Don't implement an active change whose proposal, design,
or tasks aren't approved yet — see `openspec/changes/<name>/` for its current
state.

Full detail: [`CLAUDE.md`](CLAUDE.md)'s "Spec-driven development" section.

## Adding or changing a registry component

A registry item (`registry/ui/*.rs`) is a vertical slice, not just a `.rs`
file: registry metadata (`registry/registry.json`), the component source, a
playground demo, and tests. After editing a registry item, regenerate and
verify its derived records before committing:

```sh
cargo run -p adico-xtask -- registry build
cargo run -p adico-xtask -- registry validate
cargo run -p adico-xtask -- primitive-usage sync && cargo run -p adico-xtask -- primitive-usage check
cargo run -p adico-xtask -- styling-usage sync && cargo run -p adico-xtask -- styling-usage check
cargo run -p adico-xtask -- prop-parity sync && cargo run -p adico-xtask -- prop-parity check
cargo run -p adico-xtask -- playground-controls sync && cargo run -p adico-xtask -- playground-controls check
```

[`docs/development.md`](docs/development.md) explains what each of these
checks against and when to run it.

Never `cargo check` a `registry/ui/*.rs` file in isolation — it's Dioxus
source distributed to consumers, not a compiled workspace member. Validate it
through `registry validate` and through an installed consumer
(`examples/basic-spa`, `examples/basic-ssr`, or a `tests/installation/*`
fixture).

## Validation before opening a PR

```sh
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings
cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask
openspec validate --all --strict
```

Run surface-specific checks (wasm32, Playwright, SSR/hydration) only when your
change actually touches that surface — see
[`docs/validation.md`](docs/validation.md) for the full matrix. If you skip a
check, say so in the PR description along with why.

## Pull requests

- Keep a PR to one vertical slice or one focused fix.
- Add tests for behavior changes — see each crate's existing test layout
  before adding your own (`adico-primitives`'s
  [conventions](packages/adico-primitives/README.md#conventions) are a good
  model: black-box tests under `tests/`, not inline `#[cfg(test)]`).
- If your change touches a public contract (registry schema,
  `components.json`, CLI flags), update the relevant `openspec/specs/` delta
  as part of the same PR, not as a follow-up.

## Code of conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).
