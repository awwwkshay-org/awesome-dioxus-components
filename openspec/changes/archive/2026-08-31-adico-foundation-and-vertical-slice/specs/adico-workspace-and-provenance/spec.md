## Purpose

Define the independently maintainable adico workspace and the provenance rules
that make reused Dioxus Components code legally and operationally sustainable.

## ADDED Requirements

### Requirement: Adico has explicit product boundaries
The workspace SHALL separate maintained applications, reusable packages,
consumer-style examples, registry source, and validation fixtures. It SHALL
provide packages for the `adico` CLI, owned primitives, registry core, test
utilities, and repository automation without making copied UI source depend on
a monolithic styled-component crate.

#### Scenario: Consumer installs a registry component
- **WHEN** a consumer installs a component that needs headless behavior
- **THEN** the installed source depends only on documented public runtime APIs
  and its declared ordinary Cargo dependencies

### Requirement: Primitives are owned and independently releasable
The project SHALL own an `adico-primitives` runtime layer for behavior needed by
registry components, including any initially reused Dioxus primitive code. The
runtime SHALL not require unmerged upstream changes for fixes, additions, or
releases.

#### Scenario: An upstream primitive defect blocks a component
- **WHEN** a defect is found in reused primitive behavior
- **THEN** adico can patch, test, and release the required behavior without an
  upstream merge

### Requirement: Reused code has auditable provenance
Every imported, forked, or materially ported upstream source unit SHALL retain
license notices and be traceable to an upstream repository, immutable source
revision, applicable license, import date, and local changes. The repository
SHALL retain the required MIT and Apache-2.0 licensing and attribution for
Dioxus Components-derived code.

#### Scenario: Maintainer audits a forked source file
- **WHEN** a maintainer inspects a reused source unit
- **THEN** they can determine its upstream origin, license obligations, and
  local divergence without relying on tribal knowledge

### Requirement: Platform-specific behavior is isolated
Registry component public APIs SHALL be usable across supported Dioxus targets
without directly embedding browser-only implementation details. Target-specific
behavior required for web, desktop, SSR, or fullstack operation SHALL be
isolated behind the runtime or documented as an explicit compatibility limit.

#### Scenario: A component is built for SSR
- **WHEN** a consumer builds an installed component in an SSR-enabled Dioxus app
- **THEN** browser-only behavior does not execute during server rendering
