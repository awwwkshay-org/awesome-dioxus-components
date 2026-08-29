## Purpose

Keep `examples/` limited to a fixed, minimal set of CLI-installed consumer
fixtures — one SPA and one SSR fixture — so every example in the workspace
carries real parity evidence instead of unwired placeholder scaffolding.

## ADDED Requirements

### Requirement: Exactly two example fixtures exist
The `examples/` directory SHALL contain exactly two workspace-member
fixtures: `basic-spa` and `basic-ssr`. No other directory under `examples/`
SHALL be a Cargo workspace member.

#### Scenario: Workspace member list is inspected
- **WHEN** a maintainer reads the root `Cargo.toml` `[workspace].members` list
- **THEN** it lists `examples/basic-spa` and `examples/basic-ssr` and no other
  path under `examples/`

#### Scenario: A new example is proposed
- **WHEN** a future change proposes adding another directory under
  `examples/`
- **THEN** it SHALL either replace one of the two existing fixtures or modify
  this requirement's fixture count explicitly, rather than growing the set
  silently

### Requirement: basic-spa fixture proves the web-only CLI-installed build
`examples/basic-spa` SHALL be initialized and kept current through the real
`adico` CLI (`adico init`, `adico add`) rather than hand-edited component
source, and SHALL build for the `wasm32-unknown-unknown` target using only
the `web` Dioxus feature.

#### Scenario: basic-spa is refreshed
- **WHEN** the CLI installer's public commands are re-run against
  `examples/basic-spa`
- **THEN** the fixture's installed component source, `adico.lock`, and
  `components.json` are produced by those commands, not manual edits

#### Scenario: basic-spa build is validated
- **WHEN** `examples/basic-spa` is built for the `wasm32-unknown-unknown`
  target with default features
- **THEN** the build succeeds using only the `web` Dioxus feature, with no
  `server` or `desktop` feature enabled

### Requirement: basic-ssr fixture proves server-rendering and hydration
`examples/basic-ssr` SHALL be initialized and kept current through the real
`adico` CLI, SHALL build successfully under both the `server` feature and the
`web` feature, and SHALL serve server-rendered HTML that a client build
hydrates without console errors.

#### Scenario: basic-ssr builds under both features
- **WHEN** `examples/basic-ssr` is built with `--no-default-features
  --features server` and separately with `--no-default-features --features
  web --target wasm32-unknown-unknown`
- **THEN** both builds succeed

#### Scenario: basic-ssr hydrates without console errors
- **WHEN** `examples/basic-ssr` is served and a browser loads its
  server-rendered page
- **THEN** the installed components render in the initial server response and
  remain interactive after client hydration with zero console errors or
  warnings

### Requirement: Removed platform coverage is recorded as a named gap
When an example fixture that was the sole evidence source for a parity
dimension is removed, the corresponding `parity.json` entries SHALL be
updated to reflect the removal with an explicit note rather than left
pointing at a nonexistent path or left silently claiming a passing
validation.

#### Scenario: Desktop example fixture is removed
- **WHEN** `examples/desktop` is deleted and no other fixture builds against
  a native desktop target
- **THEN** every `parity.json` component entry whose `desktop` dimension
  evidence named `examples/desktop` is updated to `passed: false` with a note
  naming the fixture's removal, and no dimension evidence array continues to
  reference a deleted path
