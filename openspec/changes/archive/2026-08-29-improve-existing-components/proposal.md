## Why

The 21 currently installable components prove the registry pipeline, but their
public APIs, shadcn-style variants, Dioxus behavior, playground examples, and
live controls are incomplete and inconsistent. Before adding another component,
adico needs a dependable existing component set that users can inspect, adjust,
and copy into real applications with confidence.

## What Changes

- Audit and improve all current registry UI items: Button, Badge, Card, Input,
  Textarea, Skeleton, Item, Pagination, Dialog, Sheet, Select, Combobox,
  Tooltip, Popover, Hover Card, Dropdown Menu, Context Menu, Menubar,
  Calendar, Date Picker, and Sidebar.
- Bring every current registry component to its applicable Dioxus Components
  feature surface: public composition parts, controlled/uncontrolled props,
  variants, sizes, states, semantic theme, keyboard/pointer/focus behavior,
  accessibility, responsive behavior, documentation, and consumer examples.
  Use the Dioxus Components catalog's compositional primitive parts,
  accessible keyboard interactions, live examples, and per-component CLI
  installation model as the Dioxus design reference; use current shadcn
  contracts for the semantic visual language.
- Make every playground route a useful component workbench: display the
  component's own example at a logical centered size and expose explicitly
  chosen live controls for its supported props, options, types, values, and
  states.
- Start with Button, exposing its child composition, variant, size, disabled,
  type, and other native button options in the playground before progressing
  through the remaining components in dependency-coherent batches.
- Treat Button, Pagination, Select, Combobox, Calendar, Date Picker, and
  Sidebar as the first hardening wave. The selection and date components keep
  `adico-primitives` as their owned Dioxus behavior layer; registry source adds
  the shadcn-style visual composition and documents every intentional Dioxus
  API difference instead of replacing that behavior with a React-shaped API.
- Correct shared behavior in `adico-primitives`, composition/styling in
  registry source, and refresh consumer copies only through the `adico` CLI.
- Maintain a feature ledger for all 21 items. A feature may be omitted only
  when the record names an intentional idiomatic-Dioxus alternative or a
  concrete target/platform constraint, including its validation disposition.

**BREAKING**: A copied component's public Rust API may change when required to
close a documented parity gap. Each change SHALL have a migration note and a
consumer-fixture verification before release.

## Capabilities

### New Capabilities

- `adico-existing-components`: Evidence-driven hardening and interactive
  playground controls for all currently installed adico registry components.

### Modified Capabilities

- None.

## Impact

- Affected source: `registry/ui`, relevant `adico-primitives` modules,
  registry metadata/checksums, and CLI-refreshed consumer copies.
- Affected applications: playground routes, consumer-style examples, and
  component validation fixtures.
- The first wave adds documented, typed public props only where they map to
  real Dioxus semantics: Button variants/sizes/native attributes; Pagination
  link and label configuration; selection/date controlled state and supported
  primitive options; and Sidebar layout state.
- The playground is a required consumer-validation surface for every
  remediated component, not a follow-up showcase. A component is incomplete
  until its CLI-installed playground copy exposes its meaningful typed props
  through live controls and the preview reflects each supported choice.
- This change does not add any of the additional components in the 45-item
  Dioxus Components catalog. It completes the applicable feature surface of
  adico's existing 21 registry items before that expansion begins.
- No database or deployment changes are expected. New shared dependencies need
  an explicit design and provenance decision.
