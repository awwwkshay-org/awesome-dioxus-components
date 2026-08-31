# adico-project-configuration Specification

## Purpose
Define the consumer configuration and safe Rust module conventions that allow
adico to install editable source into diverse Dioxus project layouts.

## Requirements

### Requirement: Components configuration is portable and explicit
`adico init` SHALL create or update a project-root `components.json` that records
the detected Dioxus project shape, component/UI/utility destinations, CSS entry
point, selected style/theme settings, named registry sources, and a default
registry selection. The configuration format SHALL use `components.json`, not
an adico-branded file name, and SHALL be validated before use.

#### Scenario: Initializing a supported Dioxus project
- **WHEN** a user runs `adico init` in a supported project with a Cargo.toml
- **THEN** a valid `components.json` describes the destinations and theme entry
  point that subsequent adds will use

#### Scenario: Configuration is invalid
- **WHEN** a user runs an adico command with an invalid or incomplete
  `components.json`
- **THEN** the command reports the invalid fields and does not guess a
  destination or overwrite project files

#### Scenario: Company changes the default registry
- **WHEN** a project configures a compatible `@awwwkshay` source and sets it as
  `defaultRegistry`
- **THEN** subsequent bare component names resolve through that source while
  explicitly namespaced registry addresses remain available

### Requirement: Project detection fails safely
The CLI SHALL identify supported Dioxus projects from project metadata and
source layout. It SHALL clearly report unsupported, ambiguous, or missing
project prerequisites and offer no unsafe implicit conversion of arbitrary Rust
projects.

#### Scenario: No Dioxus project is found
- **WHEN** `adico init` is invoked outside a supported Dioxus project
- **THEN** it reports the detection failure and leaves the directory unchanged

### Requirement: Module exports preserve user code
The installer SHALL manage installed Rust modules only within explicit,
delimited adico-managed regions in the appropriate `mod.rs` files. It SHALL add
module declarations and re-exports idempotently and preserve all user-written
content outside those regions.

#### Scenario: Adding a second component
- **WHEN** a user adds a second component to an existing UI destination
- **THEN** its module declaration and re-export are added once while existing
  exports and code outside the managed region remain byte-for-byte untouched

### Requirement: Module changes are conflict-safe
The installer SHALL refuse to modify a malformed, missing-marker, or manually
conflicted managed region when it cannot establish a safe update. Component
removal and rename operations SHALL be deferred until an explicit command and
ownership model are specified.

#### Scenario: Managed region was manually corrupted
- **WHEN** the module marker region cannot be parsed safely
- **THEN** the CLI reports the conflict and does not perform partial module
  edits
