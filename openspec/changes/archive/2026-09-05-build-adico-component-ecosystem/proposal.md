## Why

Dioxus developers need a coherent, source-owned way to adopt accessible,
shadcn-style components without coupling their applications to a monolithic UI
crate or waiting for upstream component work to merge. This change establishes
**Awesome Dioxus Components** (`adico`): an independently releasable primitive
layer, a Rust-native component registry, and an installer CLI that copies
editable component source into a consumer's project.

The work must prove a reliable distribution pipeline with the components that
already exist in `DioxusLabs/dioxus-components` before expanding toward full
first-party shadcn catalog parity. This sequencing minimizes duplicate work and
makes the platform useful while parity is progressively delivered.

## What Changes

- Reframe the repository as the `adico` monorepo, preserving the existing Rust
  workspace while adding the conceptual `apps`, `packages`, `examples`,
  `registry`, `tests`, `scripts`, and parity-tracking boundaries.
- Establish `adico-primitives` as an owned, independently releasable headless
  runtime layer. It will begin with appropriately reused/forked Dioxus
  primitives, recorded with source commit, license, notices, port history, and
  local divergence metadata; upstream contribution remains optional.
- Introduce `adico-registry-core` and a checked-in registry source/build model
  for source-owned UI items, their Rust/Cargo dependencies, their registry
  dependencies, module-export requirements, and theme/CSS requirements. An
  organization can curate a compatible local or static HTTPS registry and make
  it the consumer project's default without forking the CLI.
- Introduce the `adico` CLI with production-quality `init`, `add <items...>`,
  `add --all`, `list`, and `view <item>` behavior. It creates/reads `components.json`, safely plans
  and installs source, edits Cargo.toml structurally, and manages Rust module
  exports only inside explicit adico-owned regions.
- Deliver a first vertical slice using Button, Dialog, and one existing
  complex component selected after the Dioxus Components inventory. The slice
  validates source installation, `adico-primitives`, CSS/theme setup, Cargo
  changes, module management, builds, and interaction behavior in a real
  consumer-style example.
- Inventory the current upstream Dioxus Components styled catalog and primitive
  catalog, classify each reusable item, and migrate appropriate existing items
  to the registry before implementing missing shadcn components.
- Add a checked-in, refreshable snapshot of the current first-party shadcn
  catalog plus a machine-readable parity manifest and `cargo xtask parity`
  reporting. Catalog refreshes are explicit and reviewable rather than a live
  network dependency of normal CI.
- Define the subsequent staged path to full current shadcn parity: close gaps
  in inherited components, then implement missing components by shared
  primitive dependency groups, followed by application and newer chat/agent
  components.
- Add examples, compile/install fixtures, browser/keyboard/accessibility,
  SSR/hydration, desktop, and visual test organization so examples validate
  the same CLI installation path users follow.
- Add a cross-platform theme-mode primitive (Light/Dark/System, detected via
  the `dark-light` crate on web and desktop, with a deterministic SSR-safe
  fallback) and two installable registry components built on it: a
  shadcn-equivalent `mode-toggle` and an adico-original `theme-switcher`
  palette picker, both persisted across reloads and installed into the
  examples through the real CLI.
- Add `adico css build`/`--check`, a Node-free compile step for a consumer's
  Tailwind CSS using Tailwind's own standalone native CLI (fetched and cached
  by `adico-cli`, correcting `m0-toolchain-decisions.md`'s stale npm-package
  pin), wired into `adico init`/`adico add` so a fresh or updated project
  always renders styled output without a separate manual compile step — the
  `npx shadcn add`-equivalent developer experience.

## Capabilities

**Note (2026-08-31):** `adico-workspace-and-provenance`, `adico-registry`,
`adico-project-configuration`, `adico-cli-installation`, and
`adico-component-validation` were extracted into their own change
(`adico-foundation-and-vertical-slice`), synced into `openspec/specs/`, and
archived, since their requirements were fully established by the now-complete
M0–M3 milestones (see `openspec/changes/archive/2026-08-31-adico-foundation-and-vertical-slice/`).
This change's remaining scope (M4–M10) only still produces/modifies the two
capabilities below.

### New Capabilities

- `adico-primitives`: Defines `adico-primitives` as a documented, public,
  shared behavior layer (positioning, menu anatomy, layering, roving focus,
  direction/RTL, controllable state, independent primitive-level testing)
  that registry components compose, per the M6 shared-primitive-expansion
  work (design.md §8a).

### Modified Capabilities

- `adico-example-fixtures`: Removes the "Removed platform coverage is
  recorded as a named gap" requirement's dependency on the now-removed
  `parity.json` tracking system (task 7.x), replacing it with a
  change-notes-based record of the same intent.

## Impact

- **Workspace and public APIs:** Adds `adico-cli`, `adico-primitives`,
  `adico-registry-core`, `adico-test-utils`, and `adico-xtask`, plus registry
  source and consumer examples. `adico` becomes a public executable and
  `adico-primitives` a public runtime dependency for installed components.
- **Consumer contracts:** Introduces `components.json`, including named
  registry sources and `defaultRegistry`, adico-owned regions in generated
  `mod.rs` files, and a documented Tailwind/CSS-variable theme entry point.
  Existing user code outside managed regions remains untouched.
- **Compatibility:** Web, desktop, SSR, and fullstack support are architectural
  requirements; initial validation is deliberately milestone-scoped. Browser
  interop is isolated behind primitives rather than copied UI source.
- **Dependencies and licensing:** New Rust tooling dependencies include a
  structured TOML editor; final Dioxus, Tailwind, icon, and browser-test
  dependency versions are selected during the recorded compatibility audit.
  Forked Dioxus Components code retains Apache-2.0 and MIT obligations and
  auditable provenance. `adico-primitives` also gains the MIT/Apache-2.0
  `dark-light` crate for cross-platform (web/desktop) theme-mode detection.
  `adico-cli` gains the ability to download and cache Tailwind's standalone
  native CLI release artifact; `m0-toolchain-decisions.md`'s prior pin on the
  npm-distributed `@tailwindcss/cli` is corrected to this standalone binary,
  which `dx serve`/`dx build` already use in practice. No Node/npm dependency
  is introduced for `adico` itself or for any downstream consumer project.
- **Non-goals:** This change does not require authenticated registry transport,
  registry discovery/marketplaces, MCP integration, every future CLI command,
  every shadcn block, or immediate completion of every missing shadcn
  component. It does require an architecture that supports a configured local
  or static HTTPS organizational registry and an explicit plan through full
  parity.
