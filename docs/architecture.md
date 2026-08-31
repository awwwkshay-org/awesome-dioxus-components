# Architecture

Awesome Dioxus Components (`adico`) is a source-distribution ecosystem for
Dioxus, not a conventional styled component crate. The user installs authored
registry source into their own project and owns the resulting files.

```text
Dioxus
  ↓
adico-primitives
  ↓
registry source + metadata
  ↓
adico CLI / registry core
  ↓
consumer Dioxus application
```

## Boundaries

- `packages/adico-primitives` owns stable, reusable headless behavior. It can
  fork licensed Dioxus Components primitives, preserving provenance.
- `packages/adico-registry-core` owns registry schemas, source loading,
  dependency resolution, compatibility checks, and installation plans.
- `packages/adico-cli` owns project discovery and safe changes to consumer
  files, `Cargo.toml`, CSS, and managed Rust module regions.
- `registry/` contains styled source, metadata, hooks, libraries, and blocks.
  It is distribution input, not a Cargo styled-component package.
- `apps/docs`, `apps/playground`, and `examples/` are maintained Dioxus apps.
  Consumer-focused fixtures must exercise the CLI installation path rather than
  import registry UI source through workspace paths.

## Platform rules

Web, desktop, SSR, and fullstack are supported design targets. Browser-only
behavior belongs behind target-aware primitive adapters; copied component APIs
must not expose browser interop details. Native and SSR validation may be
progressive, but skipped coverage is recorded rather than treated as a pass.

## Registry rules

The official registry is embedded with the CLI. A consumer can also configure a
named local or static-HTTPS organization registry in `components.json` and make
it the default. Explicit item references such as `@adico/button` select the
official registry when an organization default is active. See the active
[registry design](../openspec/changes/build-adico-component-ecosystem/design.md).

`registry/` stays at the repository root rather than moving under `packages/`.
`packages/` is this workspace's Cargo-member namespace — every directory in it
is a crate with a `Cargo.toml` — while `registry/` is deliberately not a crate:
`adico` distributes registry source the way shadcn does (consumer-owned copies
via CLI install), not as a compiled dependency, and filing it under `packages/`
would put a non-crate, non-member directory directly beside a real crate like
`packages/adico-registry-core`, inviting exactly the failure mode this doc
already warns against — something importing registry source via a workspace
path instead of the CLI installer. See
[`enforce-registry-facade-standards`'s design](../openspec/changes/enforce-registry-facade-standards/design.md)
for the full analysis.

Every `registry/ui`/`registry/component` file styles exclusively through
Tailwind utility classes and semantic design tokens (`bg-background`,
`text-foreground`, `bg-primary`, `border-input`, `ring-ring`, `bg-sidebar*`,
etc.), never raw CSS (`style { ... }` blocks or a `style:` attribute) and
never a hardcoded color where a token applies. A genuinely runtime-computed
value (a progress bar's computed width, a generated CSS custom property) or a
legitimate non-token color (an exact match to upstream shadcn, an inherently
theme-independent affordance) is a recorded, reasoned exception, not silently
allowed — see `cargo xtask styling-usage check` in
[`docs/development.md`](../docs/development.md).
