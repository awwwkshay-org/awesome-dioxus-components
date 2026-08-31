## Purpose

Define the user-facing adico installation workflow that turns registry entries
into source-owned, buildable Dioxus component code in a consumer project.

## ADDED Requirements

### Requirement: Registry discovery and inspection are source-aware
The CLI SHALL provide `adico list` and `adico view <item>` as read-only registry
commands. `list` SHALL enumerate the items available from the configured default
registry, or from an explicitly selected configured namespace. `view` SHALL
resolve bare and namespaced item addresses by the same rules as `add` and report
the stable item address, description, item type, source/target files, registry
dependencies, Cargo dependencies, style requirements, compatibility, and
provenance metadata. Neither command SHALL mutate the consumer project.

#### Scenario: Company registry lists its curated components
- **WHEN** an Awwwkshay project sets `@awwwkshay` as its default registry and
  runs `adico list`
- **THEN** the CLI lists the compatible items from `@awwwkshay` and identifies
  their namespace

#### Scenario: User inspects an official component from a company project
- **WHEN** a project runs `adico view @adico/dialog`
- **THEN** the CLI reports Dialog's official source identity and all declared
  installation requirements without changing project files

### Requirement: Initialization prepares a supported project
The `adico init` command SHALL detect and validate a supported Dioxus project,
create required component directories, establish `components.json`, prepare
managed module files, and install or configure the selected theme/CSS entry
point without replacing unrelated user code.

#### Scenario: Fresh supported project initialization
- **WHEN** a user runs `adico init` in a supported Dioxus project
- **THEN** the command reports each completed preparation step and leaves the
  project ready for `adico add`

### Requirement: Add installs source-owned component files
`adico add <component...>` and `adico add --all` SHALL resolve requested
registry items, install their source into the configured consumer destinations,
and update the required module exports, Cargo dependencies, and styling
requirements. Installed source SHALL be ordinary consumer-project source that
the user can modify without editing an adico package.

#### Scenario: User adds Button and Dialog
- **WHEN** a configured consumer runs `adico add button dialog`
- **THEN** source files, transitive registry items, module exports, declared
  Cargo dependencies, and required theme changes are installed as one resolved
  request

### Requirement: Add honors configured registry selection
The CLI SHALL resolve bare component names from `components.json`'s configured
default registry and resolve namespaced addresses from the referenced named
registry. Its reviewable plan and result SHALL identify the source registry for
every requested and transitive item.

#### Scenario: Company default registry is used
- **WHEN** an Awwwkshay project runs `adico add card` with `@awwwkshay` as its
  default registry
- **THEN** the planned and installed Card source is identified as originating
  from `@awwwkshay`

### Requirement: Cargo edits are structured and idempotent
The installer SHALL update Cargo.toml through structured TOML edits, avoid
duplicate dependencies, preserve unrelated content as far as the TOML format
allows, and support ordinary workspace dependency layouts where the requirement
can be resolved unambiguously. It SHALL report incompatible existing versions
or unresolved workspace ownership rather than silently replacing them.

#### Scenario: Dependency is already compatible
- **WHEN** an item requires a Cargo dependency already declared compatibly
- **THEN** the installer leaves one declaration and does not duplicate it

#### Scenario: Existing dependency conflicts
- **WHEN** an item requires a version incompatible with the consumer manifest
- **THEN** the CLI reports the conflicting declaration and does not silently
  alter its version

### Requirement: Installation avoids destructive overwrites
Repeated installation of unchanged adico-managed files SHALL be idempotent. If
a destination file differs from the registry-owned version or cannot be safely
merged, the CLI SHALL preserve it, report a conflict, and require an explicit
user-selected overwrite path before replacement.

#### Scenario: User modified an installed Button
- **WHEN** a later add would replace that modified Button source file
- **THEN** the CLI does not overwrite it automatically and identifies the file
  as a conflict

### Requirement: Installation reports a reviewable result
Before making changes, the CLI SHALL be capable of presenting a resolved plan;
after a successful add it SHALL report installed, unchanged, and conflicted
items plus Cargo, module, and theme actions. Failed planning or validation
SHALL not leave a partial installation.

#### Scenario: Dependency planning fails
- **WHEN** an add request cannot resolve all required items
- **THEN** the CLI reports the failure before modifying consumer source or
  manifests
