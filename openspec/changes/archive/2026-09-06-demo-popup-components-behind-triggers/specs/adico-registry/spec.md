## ADDED Requirements

### Requirement: Documented usage examples name real exports
A registry item's `documentation.usage` field SHALL reference only component
names the item itself exports (directly, or via a declared
`registryDependencies` item). It SHALL NOT name a symbol that exists only as
hand-written, uninstalled application code (e.g. a playground-only launcher
component).

#### Scenario: A usage example is copy-pasted by a consumer
- **WHEN** a consumer copies a registry item's `documentation.usage` example
  into their own project after installing that item (and its declared
  dependencies)
- **THEN** every symbol referenced in the example resolves to code the
  installation actually produced, with no additional undocumented component
  required
