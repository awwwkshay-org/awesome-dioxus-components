# Awesome Dioxus Components

`adico` is building a shadcn-style component ecosystem for Rust Dioxus.
Components live in a registry and are copied into a consuming application's
source tree, where the application owns and can customize them.

## Workspace

| Path | Purpose |
| --- | --- |
| `packages/adico-cli` | The future `adico` command-line installer. |
| `packages/adico-primitives` | Owned headless runtime behavior used by installed components. |
| `packages/adico-registry-core` | Registry schemas, source resolution, and installation plans. |
| `registry/` | Authored source and metadata distributed to consumer projects. |
| `apps/docs`, `apps/playground` | Maintained Dioxus documentation and development applications. |
| `examples/` | Consumer-style Dioxus validation applications. |

The active OpenSpec change is
[`build-adico-component-ecosystem`](openspec/changes/build-adico-component-ecosystem/).
It defines the delivery sequence: own primitives, prove `adico init` and
`adico add` with existing Dioxus components, then close the current first-party
shadcn catalog gap.

## Current status

The workspace foundation and upstream inventory are in place. The CLI and
registry installation pipeline are not available yet. See the
[implementation tasks](openspec/changes/build-adico-component-ecosystem/tasks.md)
for milestone status.

## Development checks

```sh
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
openspec validate build-adico-component-ecosystem --strict
```

See [`docs/architecture.md`](docs/architecture.md),
[`docs/development.md`](docs/development.md), and
[`docs/validation.md`](docs/validation.md) for repository conventions and
platform-specific validation.

## Licensing and upstream provenance

The project is licensed under [MIT](LICENSE.txt). Forked or ported upstream
source is tracked by [`UPSTREAMS.md`](UPSTREAMS.md) and the records under
`provenance/`.

## AI coding agents

Repository guidance is in [`CLAUDE.md`](CLAUDE.md). Significant changes use
the OpenSpec workflow: propose, review, implement, validate, then archive.
