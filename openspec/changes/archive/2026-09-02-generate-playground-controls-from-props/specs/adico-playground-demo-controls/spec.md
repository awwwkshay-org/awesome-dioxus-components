## Purpose

Keep every playground demo page's control panel for an enum-valued prop in
exact agreement with the real component's variants, so a page can never
silently offer a stale or incomplete subset of what the component actually
supports.

## ADDED Requirements

### Requirement: Enum-valued prop controls are generated from the installed component source
For every enum-typed prop with a `#[default]` variant on a playground UI
component that has a demo control panel, the option list offered to the user
SHALL be generated from that component's actual installed source, not
hand-authored. The generated option list SHALL include every public variant
of the enum, in declaration order, each labeled by mechanically converting
its identifier to space-separated words (for example, `IconXs` becomes
`Icon Xs`). Registry source doc comments are prose, not short labels, and
SHALL NOT be used as label text.

#### Scenario: A component's demo page offers enum options
- **WHEN** the Badge demo page renders its Variant control
- **THEN** the control offers all variants declared on `BadgeVariant` in
  `apps/playground/src/components/ui/badge.rs`, including any not previously
  reachable from the playground

#### Scenario: A component's demo page offers Item's variants
- **WHEN** the Item demo page renders its Variant control
- **THEN** the control offers all variants declared on `ItemVariant`,
  including `Outline`

### Requirement: Generated option lists cannot silently go stale
An added, removed, or renamed variant on a playground UI component's
enum-typed prop, without its generated option list being regenerated, SHALL
cause the playground application to fail to build. Detecting the staleness
only through a separate, optional check command is not sufficient.

#### Scenario: A variant is added without regenerating
- **WHEN** a new variant is added to an enum-typed prop's declaration in
  `apps/playground/src/components/ui/*.rs` and the corresponding generated
  option list is not regenerated
- **THEN** `cargo check --locked --workspace` fails

#### Scenario: A variant is removed without regenerating
- **WHEN** a variant referenced by a generated option list is removed from
  its source enum and the generated option list is not regenerated
- **THEN** `cargo check --locked --workspace` fails

### Requirement: Generated output is verifiable without modifying the tree
A command SHALL exist that verifies every generated option list still
matches what regenerating from the current component source would produce,
without writing any file, and SHALL run offline with no network access. This
command SHALL be part of the project's committed validation commands.

#### Scenario: Generated output matches source
- **WHEN** the verification command runs against a tree where every
  generated option list agrees with its source enum
- **THEN** it completes successfully and modifies no file

#### Scenario: Generated output is hand-edited or stale
- **WHEN** a generated option list file has been hand-edited, or its source
  enum has changed without regeneration
- **THEN** the verification command fails and identifies the affected
  component

#### Scenario: Regeneration is idempotent
- **WHEN** the generation command runs twice in succession against an
  unchanged tree
- **THEN** the second run produces no file changes

### Requirement: Unsupported prop shapes are explicitly excluded, not silently dropped
A playground UI component's prop that cannot be represented by an existing
demo control (its type is `children`, `attributes`, `class`, an
`EventHandler<_>`, a `Signal`/`ReadSignal`-wrapped value, or a numeric type)
SHALL be excluded from generation with a recorded reason, not silently
omitted with no trace.

#### Scenario: A component has a prop no control can represent
- **WHEN** a playground UI component declares a prop whose type is not
  `bool`, `Option<bool>`, `String`, or an enum with a `#[default]` variant
- **THEN** that prop does not appear in the generated output, and the
  generation record for that component states why
