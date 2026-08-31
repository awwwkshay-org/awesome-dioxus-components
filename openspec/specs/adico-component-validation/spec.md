# adico-component-validation Specification

## Purpose
Define the consumer-realistic examples and layered validation that make adico
components trustworthy across interaction and Dioxus platform boundaries.

## Requirements

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

### Requirement: Playground customizes the full semantic theme contract
The playground SHALL provide a bottom-anchored customization launcher that
opens an installed-component modal with independently selectable primary,
secondary, and tertiary palette presets, a light or dark appearance, and
editable values for every installed shadcn-style semantic variable:
surfaces and foregrounds, role colors and foregrounds, border, input, ring,
radius, and sidebar values. Every combination SHALL update the shared
CSS-variable theme used by routed, installed components without requiring
component-specific theme props or source changes. The modal SHALL allow users
to copy the active light or dark canonical shadcn CSS variables as a
paste-ready CSS block.

#### Scenario: User changes a palette in dark mode
- **WHEN** a user selects primary, secondary, and tertiary palettes and turns
  on dark appearance
- **THEN** every current playground page renders with the corresponding dark
  semantic variables, including primary, secondary, tertiary, foreground,
  focus-ring, and sidebar values

#### Scenario: User customizes a semantic value
- **WHEN** a user changes a semantic color, foreground, structural, or sidebar
  value in the customization tray
- **THEN** every affected routed component updates through the shared theme
  contract and no copied component source changes

#### Scenario: User generates a theme combination
- **WHEN** a user asks the customization tray to generate a theme
- **THEN** it selects a new primary, secondary, and tertiary palette
  combination with light and dark semantic values, which the user can further
  customize through the same tray

#### Scenario: User copies the active appearance
- **WHEN** a user selects Copy CSS variables from the theme modal
- **THEN** the active appearance is copied as a `:root` or `.dark` CSS block
  containing the canonical shadcn variables

#### Scenario: User changes appearance without changing palettes
- **WHEN** a user switches an existing palette combination between light and
  dark appearance
- **THEN** the same selected palette roles remain in effect while their
  light- or dark-mode values and accessible foreground values update live, and
  direct customizations for each appearance remain available

### Requirement: Current migrated components are hardened before new migration
Before the project resumes migration of new registry components, it SHALL audit
the currently migrated set in a consumer-style playground for public
composition/API, semantic-theme coverage, visual states, keyboard and pointer
behavior, accessibility, responsive layout, and focused example coverage. Each
identified defect SHALL be corrected or recorded as an explicit, bounded
exception with a follow-up dependency.

#### Scenario: A migrated component lacks theme coverage
- **WHEN** the hardening audit finds that an installed component does not react
  to a referenced semantic color, foreground, radius, or sidebar token
- **THEN** the project corrects the shared theme or component source and adds
  focused evidence that the consumer-style component now responds

#### Scenario: A migrated component regresses under interaction
- **WHEN** the hardening audit finds a keyboard, pointer, focus, or accessible
  naming defect in an installed interactive component
- **THEN** the project corrects the shared primitive or copied source and adds
  focused validation before starting another migration wave

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
