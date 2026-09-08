## MODIFIED Requirements

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
