# adico-registry Specification

## Purpose
Define an extensible, local-first registry that distributes understandable Rust
source files and the installation metadata necessary to make them work.

## Requirements

### Requirement: Registry items describe installable source
Each registry item SHALL have a stable name, item category, description, source
files, target-path intent, registry dependencies, Cargo dependency requirements,
and applicable style/theme requirements. The schema SHALL support UI,
component, hook, library, block, page, theme, file, and template categories,
even when an initial release supports only a subset.

#### Scenario: A UI item has supporting requirements
- **WHEN** a consumer requests a registry UI item with a utility, runtime, and
  theme requirement
- **THEN** the registry can express all required source and non-source inputs
  without undocumented installer behavior

### Requirement: Registry resolution is complete and deterministic
The registry SHALL resolve the transitive dependency graph for requested items,
deduplicate shared dependencies, preserve a deterministic installation order,
and reject missing, cyclic, incompatible, or unsupported dependencies before
mutating a consumer project.

#### Scenario: Multiple requested components share a dependency
- **WHEN** a consumer adds two components that require the same registry item
- **THEN** the shared item appears once in the resolved install plan

#### Scenario: The registry contains a cycle
- **WHEN** an item dependency graph contains a cycle
- **THEN** resolution fails with the involved item names and no installation is
  performed

### Requirement: Organizations can select curated registry sources
The registry model SHALL support a built-in official source and named,
organization-curated sources from a local path or static HTTPS endpoint. A
consumer project SHALL select one configured source as its default registry.
Bare item names SHALL resolve against that default; namespaced item addresses
SHALL resolve against the named source so a project can explicitly request an
official or organization item regardless of its default.

#### Scenario: Awwwkshay selects its company registry
- **WHEN** an Awwwkshay project configures `@awwwkshay` as its default registry
  and runs `adico add button`
- **THEN** the CLI resolves `button` from `@awwwkshay` rather than the official
  adico registry

#### Scenario: Project explicitly requests the official item
- **WHEN** a project with a company default registry runs `adico add @adico/button`
- **THEN** the CLI resolves Button from the official registry without changing
  the configured company default

### Requirement: Curated registries are independently validatable
An organization-curated registry SHALL use the same versioned manifest and item
schema as the official registry and declare its identity, compatibility, and
source-file checksums. The CLI SHALL reject unsupported source kinds,
non-HTTPS network endpoints, unreadable sources, malformed manifests, and
incompatible registry versions before it plans consumer-project changes.

#### Scenario: Company registry is incompatible
- **WHEN** a project selects a company registry whose format is unsupported by
  the installed CLI
- **THEN** the CLI identifies the registry, its source, and the compatibility
  mismatch without modifying the project

### Requirement: Registry compatibility is explicit
The registry SHALL declare a format version and item compatibility information.
The installer SHALL reject a registry or item it cannot interpret and explain
the required compatible CLI/runtime range.

#### Scenario: Older CLI reads a newer incompatible registry
- **WHEN** the CLI encounters a registry format it does not support
- **THEN** it exits before file installation with an actionable compatibility
  error

### Requirement: Registry build output is reproducible
The authored registry source and generated consumable metadata SHALL be
validated and reproducible from checked-in inputs. Generated output SHALL not
require a live network request to resolve local items during ordinary tests or
CI.

#### Scenario: CI validates registry metadata
- **WHEN** repository validation runs without external-network availability
- **THEN** it can validate all checked-in registry metadata and source mappings
