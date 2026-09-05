## MODIFIED Requirements

### Requirement: Every installed component has a hardening record
The project SHALL maintain an evidence-backed hardening record for each of the
66 current registry UI items, not only the original 21 (Button, Badge, Card,
Input, Textarea, Skeleton, Item, Pagination, Dialog, Sheet, Select, Combobox,
Tooltip, Popover, Hover Card, Dropdown Menu, Context Menu, Menubar, Calendar,
Date Picker, and Sidebar). The record SHALL assess public composition/API,
variants and states, semantic themes and dark mode, keyboard/pointer/focus
behavior, accessibility, responsive behavior, consumer examples, and
applicable platform checks. For upstream prop-surface completeness
specifically, `statics/prop_parity/<item>.json` (generated and CI-gated by
`cargo xtask prop-parity sync|check|diff`) SHALL be the mechanical evidence
source, superseding manual re-reading of upstream source for that dimension.

#### Scenario: Component audit identifies a gap
- **WHEN** the audit finds a missing or divergent applicable dimension
- **THEN** its record identifies the evidence, owning source boundary, and a
  bounded remediation task or explicit dependency block

#### Scenario: Upstream feature inventory is complete
- **WHEN** a maintainer reviews any current registry component
- **THEN** its record maps every applicable Dioxus Components part, public
  prop, state, interaction, and accessibility behavior to an adico API,
  intentional Dioxus alternative, or named target/platform block

#### Scenario: A component is missing an upstream capability
- **WHEN** the Dioxus Components reference exposes a capability not available
  from the corresponding adico registry component
- **THEN** the change includes a bounded primitive or registry-source task for
  it before the component can be marked complete, unless the ledger records a
  user-visible reason that the capability cannot apply

#### Scenario: A registry item's prop-parity record shows a genuine gap
- **WHEN** `statics/prop_parity/<item>.json` classifies an upstream prop
  `missing` for a registry item
- **THEN** the item's hardening record either implements that prop or
  records it `intentional_difference` with a written reason before the
  item is considered hardening-complete, matching the mechanical record
  rather than a separately maintained manual assessment

### Requirement: Current registry scope reaches complete applicable parity
All 66 registry UI items, not only the original 21 (Button, Badge, Card,
Input, Textarea, Skeleton, Item, Pagination, Dialog, Sheet, Select, Combobox,
Tooltip, Popover, Hover Card, Dropdown Menu, Context Menu, Menubar, Calendar,
Date Picker, and Sidebar), SHALL each reach complete applicable parity with
their Dioxus Components reference surface and their shadcn semantic visual
contract before this requirement is satisfied, as verified by
`cargo xtask prop-parity check` reporting no unresolved `missing`
classification for any item (every upstream prop is `present`,
`intentional_difference`, or was never applicable). This requirement SHALL
NOT imply that adico adds any catalog item which is not already in the
registry, and SHALL NOT require an adico-only extension prop (for example
`radius` or `loading`) to have an upstream counterpart at all.

#### Scenario: Existing-registry hardening is complete
- **WHEN** this change is proposed for completion
- **THEN** all 66 entries have a complete feature ledger, a CLI-refreshed
  source fixture, a typed live playground example, and proportionate behavior,
  accessibility, and target validation; any remaining unavailable capability
  is visibly recorded as a named block rather than counted as parity

#### Scenario: Prop-parity check gates completion
- **WHEN** `cargo xtask prop-parity check` is run against the full 66-item
  registry
- **THEN** it passes with zero `missing`-classified props remaining
  unresolved across every item, as the mechanical completion gate for this
  requirement's upstream-parity dimension
