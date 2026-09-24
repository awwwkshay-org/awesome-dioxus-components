# Awesome Dioxus Components

[![CI](https://github.com/awwwkshay-org/awesome-dioxus-components/actions/workflows/ci.yml/badge.svg)](https://github.com/awwwkshay-org/awesome-dioxus-components/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`adico` is a [shadcn/ui](https://ui.shadcn.com)-style component ecosystem for
[Dioxus](https://dioxuslabs.com). Components aren't a compiled dependency —
`adico add button` copies `button.rs`'s actual Rust source into your project,
where you read it, own it, and can change it freely. A small headless runtime
crate (`adico-primitives`) is the one thing you depend on normally; everything
styled is source you install.

```
Dioxus → adico-primitives → registry source + metadata → adico CLI/registry-core → your app
```

## Why source-distributed components

- **No black box.** Every component is plain Dioxus/Rust you can read, step
  through, and edit — there's no compiled crate hiding the implementation.
- **No forced upgrades.** Once installed, a component is your file. Nothing
  changes underneath you until you run `adico add` again.
- **Headless behavior, styled by you.** Interaction, keyboard handling, ARIA
  roles, and positioning live in `adico-primitives`; visual styling lives in
  the installed component source, built on Tailwind semantic tokens you can
  restyle however you like.

## What's in the registry

69 components across the usual shadcn-style surface — form controls
(`button`, `checkbox`, `radio_group`, `select`, `combobox`, `input_otp`,
`date_picker`, `time_picker`, …), overlays (`dialog`, `sheet`, `drawer`,
`popover`, `tooltip`, `hover_card`, `context_menu`, `dropdown_menu`,
`navigation_menu`), layout and data display (`card`, `table`, `data_table`,
`carousel`, `accordion`, `tabs`, `sidebar`, `resizable`, `virtual_list`), and
more. Browse the full, current list in [`registry/ui/`](registry/ui/) — every
file there is exactly what `adico add <name>` installs.

## Quickstart

`adico` doesn't have its first tagged release yet (see [Project status](#project-status)),
so build it from source for now:

```sh
git clone https://github.com/awwwkshay-org/awesome-dioxus-components
cd awesome-dioxus-components
cargo install --path packages/adico-cli --locked
```

Then, in a Dioxus project:

```sh
adico init                    # creates components.json, registers the official registry
adico add button card dialog  # installs the requested components + their dependencies
```

See [`packages/adico-cli/README.md`](packages/adico-cli/README.md) for the
full command reference, and
[`docs/adico/organization-registry.md`](docs/adico/organization-registry.md)
for pointing `adico` at your own component registry.

## Workspace

| Path | Purpose |
| --- | --- |
| `packages/adico-cli` | The `adico` command-line installer. |
| `packages/adico-primitives` | Owned headless runtime behavior used by installed components — see its own [README](packages/adico-primitives/README.md). |
| `packages/adico-registry-core` | Registry schemas, source resolution, and installation planning. |
| `registry/` | Authored source and metadata distributed to consumer projects. |
| `apps/docs`, `apps/playground`, `apps/home` | Maintained Dioxus documentation, development, and landing-page applications (`dx serve` to run locally). |
| `examples/` | Consumer-style Dioxus validation applications, installed via the real CLI. |

## Project status

Pre-`0.1.0`. The registry, CLI, and 69 components are implemented and tested
(~596 Rust tests, 8 CI-gated consistency checks), but nothing is published
yet: no crates.io release, no GitHub release binaries, no hosted docs or
registry site — `apps/home` (the landing page), `apps/docs`, and
`apps/playground` exist in-repo but are not yet deployed anywhere. That work
is tracked toward a `v0.1.0` release; this section will be replaced with
real install instructions once it ships.

## Development

```sh
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings
cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask
openspec validate --all --strict
```

See [`docs/architecture.md`](docs/architecture.md),
[`docs/development.md`](docs/development.md), and
[`docs/validation.md`](docs/validation.md) for repository conventions and
platform-specific validation. [`CONTRIBUTING.md`](CONTRIBUTING.md) covers the
spec-driven (OpenSpec) workflow this project uses for non-trivial changes.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option. Forked or ported upstream source
retains its own license and is tracked by [`UPSTREAMS.md`](UPSTREAMS.md) and
the records under [`provenance/`](provenance/).

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this project, as defined in the Apache-2.0
license, shall be dual-licensed as above without any additional terms or
conditions.

## AI coding agents

Repository guidance is in [`CLAUDE.md`](CLAUDE.md). Significant changes use
the OpenSpec workflow: propose, review, implement, validate, then archive.
