## Purpose

Define the consumer-realistic examples and layered validation that make adico
components trustworthy across interaction and Dioxus platform boundaries.

## ADDED Requirements

### Requirement: Consumer-style fixtures exercise the actual installer
The repository SHALL maintain a minimal basic example and installation fixtures
that are prepared through the same `adico init` and `adico add` path available
to external users. Those fixtures SHALL not satisfy installation coverage by
directly importing registry source through workspace paths.

#### Scenario: Basic example validation
- **WHEN** the basic example is refreshed for Button, Card, and Dialog
- **THEN** it is installed through the CLI and compiles as an independent
  consumer-style application

### Requirement: Registry switching has consumer coverage
The installation suite SHALL exercise an organization-curated registry selected
as the project default and explicit cross-registry item addresses. It SHALL
validate that item dependencies retain their declared source identity and that
an incompatible configured source makes no consumer-project changes.

#### Scenario: Fixture switches to a company registry
- **WHEN** an installation fixture selects `@awwwkshay` instead of the official
  registry and adds a bare item plus an explicit `@adico` item
- **THEN** both sources resolve as configured and the resulting project builds

### Requirement: Examples provide progressive integration coverage
The repository SHALL maintain web, desktop, fullstack, forms, dashboard, and
kitchen-sink examples as relevant components become available. Kitchen-sink
SHALL render every installable registry component; focused examples SHALL
exercise their named platform or composition concerns.

#### Scenario: A component becomes installable
- **WHEN** a new registry component is declared available
- **THEN** it is added to kitchen-sink and to any focused example needed to
  validate its primary composition or platform behavior

### Requirement: Interactive components have layered behavior coverage
Interactive components SHALL have proportionate Rust, compile, browser,
keyboard, accessibility, and platform coverage before being marked complete in
the parity manifest. Dialog-like overlays SHALL additionally validate opening,
Escape, outside interaction, focus behavior, ARIA associations, nesting, and
scroll locking where applicable.

#### Scenario: Dialog behavior regresses
- **WHEN** a dialog is exercised in browser validation
- **THEN** the suite verifies trigger opening, keyboard dismissal, focus
  management, accessible naming/description, and declared layer behavior

### Requirement: Platform results are reported honestly
Validation reporting SHALL distinguish checks run for web, desktop, SSR,
hydration, fullstack, and visual regression from checks designed but not run.
No component or release report SHALL claim a target passed without executing
its required target-specific validation.

#### Scenario: Desktop validation is unavailable
- **WHEN** a change validates web and SSR but cannot run desktop tests
- **THEN** the report marks desktop validation as skipped with its reason and
  does not count it as passed parity
