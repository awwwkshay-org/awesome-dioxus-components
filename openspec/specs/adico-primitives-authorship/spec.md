# adico-primitives-authorship Specification

## Purpose

Establish that every file in `adico-primitives` is authored from an independent behavioral
specification rather than ported from another project's source, and that each primitive is
represented as a single file, so the crate can stand as adico's own primitive library instead
of a tracked fork.

## Requirements

### Requirement: A primitive's implementation carries no upstream-derived attribution
A source file under `packages/adico-primitives/src` SHALL NOT declare that its contents are
forked, derived, or ported from another project's implementation once its rewrite is
complete, and SHALL NOT be referenced by a `provenance/records/*.json` entry.

#### Scenario: A file's rewrite is complete
- **WHEN** a primitive file has been re-authored against its independent specification and
  its parity gaps closed
- **THEN** the file contains no "Forked from" / "Derived from" upstream attribution header,
  and no provenance record's `localPaths` lists that file

#### Scenario: A file still carries an upstream attribution header
- **WHEN** a file under `packages/adico-primitives/src` still declares upstream attribution
- **THEN** it SHALL be listed in exactly one `provenance/records/*.json` entry, and that
  entry's revision SHALL still appear in the file's header (enforced by
  `cargo xtask provenance check`)

### Requirement: A primitive's public behavior is specified independently of upstream source
Each rewritten primitive's observable behavior (roles, keyboard interaction, states, and
public API) SHALL be derived from the WAI-ARIA Authoring Practices Guide and this repo's own
pinned reference inventories (`statics/catalogs/base-ui.json`,
`statics/catalogs/dioxus-primitives.json`, `statics/primitive_compatibility.json`), not from
reading the upstream fork's implementation.

#### Scenario: A primitive is rewritten
- **WHEN** a primitive file is re-authored under this change
- **THEN** its target behavior is recorded (in the file's doc comment or an accompanying task
  record) as citing the ARIA APG pattern and/or the compatibility-report row it was written
  against

### Requirement: A rewritten primitive has feature parity with both reference libraries
After rewriting, a primitive's public API SHALL include the union of the features and props
that `cargo xtask primitive-compat diff` reports for that primitive against both the Base UI
and dioxus-primitives axes, unless a gap is explicitly recorded as intentionally excluded. When
a registry facade's props type is a bare re-export of a primitive's own props type
(`pub use adico_primitives::<module>::<Type>Props`), a gap `cargo xtask prop-parity diff`
reports against that registry item SHALL also be closed in the primitive's props type rather
than by introducing a registry-owned wrapper struct that would diverge from the primitive's
own type, so the facade can remain a re-export.

#### Scenario: A parity gap exists after rewrite
- **WHEN** `primitive-compat diff` reports a feature present in Base UI or dioxus-primitives
  but absent from the rewritten adico primitive
- **THEN** the feature is either implemented before the primitive's rewrite task is marked
  complete, or the exclusion is explicitly recorded with a reason

#### Scenario: A re-exporting facade's prop gap belongs in the primitive
- **WHEN** `prop-parity diff` reports a missing prop (for example a `class`
  field) against a registry item whose props type is a bare re-export of
  its primitive's own props type
- **THEN** the prop is added to the primitive's props type, and the
  registry facade continues to re-export it unchanged rather than gaining
  a registry-owned wrapper struct

### Requirement: A rewritten primitive's behavior is covered by tests before rewrite lands
A primitive file SHALL NOT be rewritten without either existing or newly added automated
test coverage (Rust unit tests and/or a `tests/playwright/*.spec.ts` suite) that pins its
specified behavior.

#### Scenario: A zero-coverage file is rewritten
- **WHEN** a primitive file has no existing `#[test]` or Playwright coverage
- **THEN** coverage is authored from the primitive's specification before or alongside the
  rewrite, not deferred to a later change

### Requirement: A primitive is represented as exactly one file
Each primitive's public module SHALL be implemented as a single `.rs` file under
`packages/adico-primitives/src`, not split across a directory of sub-modules private to
that primitive.

#### Scenario: A primitive currently spans a directory
- **WHEN** a primitive's source is organized as `<name>/mod.rs` plus sibling or nested
  sub-module files scoped only to that primitive
- **THEN** its rewrite collapses the directory into a single `<name>.rs` file exposing the
  same public API

#### Scenario: Behavior is genuinely shared across multiple primitives
- **WHEN** logic is used by more than one primitive (for example positioning, menu anatomy,
  layering, or collection/roving-focus management)
- **THEN** it remains its own top-level primitive file rather than being inlined into every
  consumer, and the one-file rule applies to it in its own right

#### Scenario: One primitive's behavior is absorbed into another's
- **WHEN** two primitives are found to implement the same behavior with only differing
  default values or presentational details (for example, one hover-triggered disclosure
  differing from another only in its default open/close delay, default placement side, and
  ARIA role), and one is designated the shared implementation the other composes
- **THEN** the absorbed primitive's file remains a single file exposing its own component
  names and prior public API, implemented as a facade that supplies its own per-component
  defaults to the shared implementation — not necessarily a bare `pub use` re-export (as
  `dropdown_menu` and `autocomplete` already are) — provided its resulting public surface is
  still the full union of both primitives' props, defaults, ARIA attributes, and behavior
  before the merge, with none dropped
