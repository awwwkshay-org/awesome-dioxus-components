# upstream-catalog-tooling Specification

## Purpose

Gives adico's compat tooling (`primitive-compat`, `component-compat`) a
single, revision-pinned, offline-readable source of truth for what each of
the four upstream projects (shadcn, Base UI, dioxus-components,
dioxus-primitives) actually exposes — inventory, primitive composition, and
props — refreshed only through an explicit, network-touching `catalog fetch`
command.

## Requirements

### Requirement: Catalog fetch is the sole network-touching command
`cargo xtask catalog fetch <axis>` SHALL be the only adico-xtask command that
performs network access. `axis` SHALL be one of `shadcn`, `base-ui`,
`dioxus-components`, `dioxus-primitives`, or `all`. Every other adico-xtask
command, including `primitive-compat sync|check|diff` and `component-compat
sync|check`, SHALL run without network access, reading only committed
`statics/catalogs/*.json` files.

#### Scenario: Fetching one axis
- **WHEN** a maintainer runs `cargo xtask catalog fetch shadcn`
- **THEN** the command fetches shadcn's current component inventory over the
  network and writes `statics/catalogs/shadcn.json`, and no other
  `statics/catalogs/*.json` file is modified

#### Scenario: Compat check runs offline
- **WHEN** a maintainer runs `cargo xtask primitive-compat check` or
  `cargo xtask component-compat check` with no network access available
- **THEN** the command completes using only the committed
  `statics/catalogs/*.json` files and does not fail due to lack of
  connectivity

#### Scenario: Base UI drift check is fetch-only
- **WHEN** a maintainer runs `cargo xtask catalog fetch base-ui`
- **THEN** the command performs the live comparison against Base UI's current
  published component list as part of producing
  `statics/catalogs/base-ui.json`
- **AND** `cargo xtask primitive-compat sync|check|diff` SHALL NOT perform
  this live comparison

### Requirement: Every fetched catalog snapshot is revision-pinned
Each `statics/catalogs/<axis>.json` file produced by `catalog fetch` SHALL
record the upstream source identifier (repository URL or documentation site),
an immutable revision marker for that fetch (a commit SHA for the two Dioxus
axes; a fetch timestamp plus, when available, an upstream version/release
identifier for shadcn and Base UI), and the date the snapshot was refreshed.
Re-running `catalog fetch` for the same axis SHALL overwrite only that axis's
file.

#### Scenario: Snapshot records provenance
- **WHEN** `cargo xtask catalog fetch dioxus-primitives` completes
  successfully
- **THEN** `statics/catalogs/dioxus-primitives.json` records the exact commit
  SHA fetched and the date of the fetch

### Requirement: Shared catalog schema across all four axes
All four `statics/catalogs/<axis>.json` files SHALL conform to one shared
schema. For each component or primitive entry, the schema SHALL record: a
stable identifier/name, its composition (the other primitives/parts it is
built from, when applicable), and its prop set with an explicit
`props_source` per prop group, where `props_source` is one of `explicit` (a
concrete list of prop name/type/default/description), `inherits_from:
<axis>.<component>.<part>` (props are a passthrough of another axis's
component and are not re-listed), or `unavailable` (no prop data could be
determined for this entry).

#### Scenario: shadcn passthrough component
- **WHEN** `catalog fetch shadcn` records shadcn's `Dialog` `Trigger` part,
  whose only declared type is `React.ComponentProps<typeof
  DialogPrimitive.Trigger>`
- **THEN** its catalog entry has `props_source: inherits_from:
  base-ui.dialog.trigger` (or the axis shadcn's Dialog composes on for that
  release) rather than a duplicated prop list

#### Scenario: shadcn augmented component
- **WHEN** `catalog fetch shadcn` records shadcn's `Dialog` `Content` part,
  which adds a `showCloseButton` prop on top of the underlying primitive's
  props
- **THEN** its catalog entry records `showCloseButton` explicitly in addition
  to noting the inherited base

#### Scenario: Base UI explicit props
- **WHEN** `catalog fetch base-ui` records a Base UI component part
- **THEN** its catalog entry has `props_source: explicit` with each prop's
  name, type, default, and description as published in Base UI's API
  reference

#### Scenario: Dioxus prop struct
- **WHEN** `catalog fetch dioxus-primitives` or `catalog fetch
  dioxus-components` records a component whose props are a Rust
  `#[derive(Props)]` struct
- **THEN** its catalog entry has `props_source: explicit` with each struct
  field's name, type, and default (when present) as declared in the fetched
  source

### Requirement: Upstream composition and adico's own composition are tracked separately
The shared catalog schema SHALL distinguish an upstream component's own
internal composition (which parts/primitives of that same upstream project it
wraps, derived from that project's source or docs) from adico's local
composition of the same concept (which `adico-primitives` items a
`registry/ui/*` component depends on, derived from `registry/ui/*.rs` and
`registry.json` without any network fetch). These SHALL NOT be merged into a
single field.

#### Scenario: Upstream composition recorded from fetch
- **WHEN** `catalog fetch shadcn` records shadcn's `Dialog` component
- **THEN** its catalog entry lists the upstream primitive parts it wraps
  (e.g. Base UI's `Dialog.Root`, `Dialog.Popup`)

#### Scenario: Adico composition derived locally
- **WHEN** `primitive-compat` or `component-compat` reports adico's own
  `Dialog` registry item
- **THEN** the primitives it depends on are derived from
  `registry/ui/dialog.rs` and `registry.json` at check time, without
  requiring any `statics/catalogs/*.json` fetch for that half of the report

### Requirement: New upstream axes are addable without changing compat-tooling internals
The set of catalog axes SHALL be an extensible registry, not a fixed list
baked into `primitive-compat`/`component-compat`. Each axis SHALL declare a
kind of either `primitive` (its catalog feeds `primitive-compat`) or
`component` (its catalog feeds `component-compat`). `primitive-compat` SHALL
iterate all registered `primitive`-kind axes, and `component-compat` SHALL
iterate all registered `component`-kind axes, without either command naming
individual axes in its own logic. Registering a new axis SHALL require
adding a fetcher for that axis and declaring its kind; it SHALL NOT require
changes to `primitive-compat`'s or `component-compat`'s comparison logic.

#### Scenario: Adding a fifth axis
- **WHEN** a maintainer adds a new `component`-kind axis (for example, a
  second component library) by implementing its fetcher and registering it
  with kind `component`
- **THEN** `cargo xtask catalog fetch <new-axis>` produces
  `statics/catalogs/<new-axis>.json` conforming to the shared schema
- **AND** `cargo xtask component-compat sync|check` includes the new axis's
  data in its report without any code change to `component_compat.rs`'s
  comparison logic

#### Scenario: Listing known axes
- **WHEN** a maintainer runs `cargo xtask catalog fetch` with no axis
  argument
- **THEN** the command's usage output lists every currently registered axis
  and its kind (`primitive` or `component`)

### Requirement: Hand-maintained judgment data survives fetch and sync
Fields that encode adico's own judgment — including but not limited to each
Base UI component's adico build `status`
(`built`/`partial`/`not_started`/`not_applicable`) and the hand-curated
`SHADCN_EXCEPTIONS`/`DIOXUS_COMPONENT_EXCEPTIONS` tables — SHALL remain
defined in adico-xtask source, not in the fetched `statics/catalogs/*.json`
files. `catalog fetch` SHALL NOT write these fields, and `primitive-compat
sync`/`component-compat sync` SHALL join the hand-maintained tables against
the fetched inventory rather than overwrite them.

#### Scenario: Fetch does not clobber build status
- **WHEN** `cargo xtask catalog fetch base-ui` is run after a component's
  hand-recorded `status` was set to `partial`
- **THEN** re-running `cargo xtask primitive-compat sync` still reports that
  component's `status` as `partial`
