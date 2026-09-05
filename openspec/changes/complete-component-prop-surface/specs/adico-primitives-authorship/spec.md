## MODIFIED Requirements

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
