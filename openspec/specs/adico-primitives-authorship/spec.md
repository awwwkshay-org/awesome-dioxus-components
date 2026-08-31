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
and dioxus-primitives axes, unless a gap is explicitly recorded as intentionally excluded.

#### Scenario: A parity gap exists after rewrite
- **WHEN** `primitive-compat diff` reports a feature present in Base UI or dioxus-primitives
  but absent from the rewritten adico primitive
- **THEN** the feature is either implemented before the primitive's rewrite task is marked
  complete, or the exclusion is explicitly recorded with a reason

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
`packages/adico-primitives/src`, not split across a directory of sub-modules private to that
primitive.

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
