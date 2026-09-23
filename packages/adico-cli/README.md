# adico-cli

The `adico` command-line installer for [Awesome Dioxus Components](../../README.md) —
a shadcn-style, source-installed component ecosystem for [Dioxus](https://dioxuslabs.com).

Unlike an ordinary compiled dependency, `adico` copies a component's actual
Rust source into your project (`src/components/ui/button.rs`, not a crate you
import) — you read it, own it, and can change it. `adico-cli` is the tool that
does the copying, dependency resolution, and drift checking; the components
themselves live in [`registry/`](../../registry/).

## Install

**Homebrew (macOS/Linux):**

```sh
brew install awwwkshay-org/tap/adico
```

**Prebuilt binary:** download the archive for your platform from the
[latest release](https://github.com/awwwkshay-org/awesome-dioxus-components/releases/latest)
(macOS arm64/x64, Linux arm64/x64, Windows x64) and verify it against the
published `.sha256` checksum.

**From source**, against this repository:

```sh
cargo install --git https://github.com/awwwkshay-org/awesome-dioxus-components \
  --locked --package adico-cli
```

or from a local clone:

```sh
cargo install --path packages/adico-cli --locked
```

`adico` isn't yet published on crates.io, so `cargo install adico-cli` isn't
live — that will follow once `adico-primitives` and `adico-registry-core` are
published ahead of it.

## Usage

```sh
adico init [--default-registry <@namespace>] [--registry <@namespace>=<embedded|relative-path|https-url>] [--dry-run]
adico add <component...> [--dry-run] [--replace]
adico add --all [--dry-run] [--replace]
adico list [--registry <@namespace>]
adico view <component>
adico css build
adico css check
```

- **`adico init`** creates `components.json` in a discovered Dioxus project,
  registering the official `@adico` registry (embedded in the binary, so this
  step works offline) plus any additional registries you pass with
  `--registry`.
- **`adico add <component...>`** resolves the requested items and their
  `registryDependencies` transitively, writes each file into the target
  configured for its `targetRoot` (`ui`, `lib`, …), records the install in
  `adico.lock`, and refuses to overwrite a file you've since modified unless
  you pass `--replace`.
- **`adico list`** / **`adico view <component>`** inspect a configured
  registry's catalog without installing anything.
- **`adico css build`** / **`css check`** manage the per-project Tailwind
  pipeline (`tailwind.css` → `assets/tailwind.css`) that installed components
  render against — see the [architecture rules](../../CLAUDE.md) for why this
  is per-project rather than shared.

## Registries

A project can install from more than one registry at once —
[`docs/adico/organization-registry.md`](../../docs/adico/organization-registry.md)
documents how to point `adico` at your own organization's components over a
local path or a static HTTPS URL, with no server-side logic required.

## Where to go next

- [`../../README.md`](../../README.md) — project overview and quickstart.
- [`../../CLAUDE.md`](../../CLAUDE.md) — full repository conventions.
- [`../../docs/architecture.md`](../../docs/architecture.md) — how this crate
  fits `adico-registry-core` and the registry.
