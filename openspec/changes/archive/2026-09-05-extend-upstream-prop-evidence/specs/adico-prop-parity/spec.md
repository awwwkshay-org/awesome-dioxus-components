## Purpose

Give a later prop-completion effort a trustworthy, per-component,
CI-gated record of which upstream props (shadcn, Base UI,
dioxus-components, dioxus-primitives) each adico registry item has, is
missing, intentionally differs from, or extends beyond — so "make sure we
have all the necessary props" is a generated, verifiable answer instead of
a manual re-read of upstream source.

## ADDED Requirements

### Requirement: Every registry item has a generated, offline-verifiable prop-parity record
For each of the 66 `registry:ui` items, `cargo xtask prop-parity sync`
SHALL generate `statics/prop_parity/<item>.json`, mapping the item to each
upstream axis's matching component and part (by kebab-case identity, using
the same hand-maintained exception tables `component-compat` already uses
for known name mismatches), and joining that item's own declared props
against each matched axis's resolved props and classifying every upstream
prop as `present`, `missing`, `intentional_difference`, or
`adico_extension`. An axis with no matching component for an item SHALL be
recorded with a null match and an empty part list, not omitted. Reason
text for `intentional_difference` and `adico_extension` classifications
SHALL be defined in adico-xtask source (fixed, hand-maintained tables,
analogous to `component-compat`'s exception tables) and joined in at
generation time; `prop-parity sync` SHALL NOT read judgment prose back out
of a previously generated `statics/prop_parity/*.json` file, so the
generated record remains fully derived from source on every run. `cargo
xtask prop-parity check` SHALL regenerate this record in memory and fail,
without writing any file, if it differs from the committed output. Both
commands SHALL run without network access, reading only committed
`statics/catalogs/*.json` and the current registry/primitive source.

#### Scenario: A component's props are classified
- **WHEN** `cargo xtask prop-parity sync` runs against the current tree
- **THEN** `statics/prop_parity/switch.json` lists every prop Base UI's
  `switch` component declares, each marked `present`, `missing`,
  `intentional_difference`, or `adico_extension`, with a reason recorded
  for every non-`present`, non-`missing` entry

#### Scenario: Generated output is verifiable without modifying the tree
- **WHEN** `cargo xtask prop-parity check` runs against a tree where every
  `statics/prop_parity/*.json` file agrees with what regenerating from the
  current source would produce
- **THEN** it completes successfully and modifies no file

#### Scenario: Generated output has drifted
- **WHEN** a `statics/prop_parity/*.json` file has been hand-edited, or the
  registry/primitive source or a `statics/catalogs/*.json` snapshot it was
  generated from has changed without regeneration
- **THEN** `cargo xtask prop-parity check` fails and identifies the
  affected item

#### Scenario: Regeneration is idempotent
- **WHEN** `cargo xtask prop-parity sync` runs twice in succession against
  an unchanged tree
- **THEN** the second run writes no file

#### Scenario: An adico extension's reason is derived, not hand-edited
- **WHEN** `cargo xtask prop-parity sync` classifies adico's `radius` prop
  on `Button` as `adico_extension` (no upstream counterpart on any axis)
- **THEN** its reason text comes from a fixed table in
  `packages/adico-xtask/src/prop_parity.rs`, and hand-editing that reason
  directly in `statics/prop_parity/button.json` without changing the
  source table causes the next `cargo xtask prop-parity check` to fail

#### Scenario: An axis with no matching component is recorded, not omitted
- **WHEN** `prop-parity` processes a registry item that has no counterpart
  on the `dioxus-components` axis (e.g. `Combobox`)
- **THEN** `statics/prop_parity/combobox.json`'s `dioxus-components` axis
  entry records a null match and an empty part list, rather than omitting
  the `dioxus-components` key entirely

### Requirement: A shadcn prop inherited from Radix resolves against the Base UI axis, or is recorded unresolved with a reason
When a shadcn catalog entry's `props_source` is `inherits_from:
"radix.<component>.<part>"`, `prop-parity` SHALL resolve that reference
against the already-fetched `statics/catalogs/base-ui.json`, applying a
documented, hand-maintained component-id and part-id alias table — each
entry verified against the committed `base-ui.json` before being added —
when the two projects' naming diverges, before comparing adico's props
against the resolved prop list. A reference with no resolvable Base UI
equivalent, or one that would require a composition judgment rather than a
mechanical rename to resolve, SHALL be recorded as unresolved with a
specific, named reason, not silently treated as zero missing props and not
resolved to a plausible-but-unverified part. An `inherits_from` reference
whose axis is not `radix` and has no catalog of its own (e.g. `vaul`,
`cmdk`) or is a shadcn-internal self-reference SHALL be recorded unresolved
with a reason naming that fact, rather than being looked up against Base
UI at all. This resolution SHALL NOT require fetching a fifth upstream axis
over the network.

#### Scenario: A direct-match reference resolves
- **WHEN** `prop-parity` encounters a shadcn part with `props_source:
  inherits_from: "radix.dialog.trigger"`
- **THEN** it compares adico's `Dialog` trigger props against
  `statics/catalogs/base-ui.json`'s `dialog.trigger` part's explicit props

#### Scenario: An aliased reference resolves
- **WHEN** `prop-parity` encounters a shadcn part with `props_source:
  inherits_from: "radix.dropdown-menu.content"`
- **THEN** it applies the component alias (`dropdown-menu` → `menu`) and
  the part alias (`content` → `popup`) and compares against
  `statics/catalogs/base-ui.json`'s `menu.popup` part

#### Scenario: A component alias and a component-scoped part alias resolve to the correct, verified target
- **WHEN** `prop-parity` encounters a shadcn part with `props_source:
  inherits_from: "radix.radio-group.root"` (Radix's group *container*, not
  its individual radio button)
- **THEN** it applies the component alias (`radio-group` → `radio`) and the
  component-scoped part alias (`root` → `group`) and compares against
  `statics/catalogs/base-ui.json`'s `radio.group` part — not `radio.root`,
  which is a different part with a different prop set

#### Scenario: A reference has no Base UI equivalent
- **WHEN** `prop-parity` encounters a shadcn part with `props_source:
  inherits_from: "radix.command.root"` and `statics/catalogs/base-ui.json`
  has no `command` entry
- **THEN** the generated record marks that part's upstream comparison
  unresolved with the reason "no resolvable Base UI equivalent", rather
  than omitting the part or reporting zero missing props

#### Scenario: A reference would require a composition judgment, not a rename
- **WHEN** `prop-parity` encounters a shadcn part with `props_source:
  inherits_from: "radix.menubar.trigger"`, and Base UI's `menubar`
  component carries only a `root` part (its actual menu items come from
  the separate `menu` component)
- **THEN** the generated record marks that part's upstream comparison
  unresolved with a reason naming the composition judgment involved, not
  a guessed mapping onto `menu`'s parts

#### Scenario: A third-party or self-referential axis is not looked up against Base UI
- **WHEN** `prop-parity` encounters a shadcn part with `props_source:
  inherits_from: "vaul.drawer.content"` or
  `"@/registry/new-york-v4/ui/button.button.root"`
- **THEN** the generated record marks that part's upstream comparison
  unresolved with a reason naming it a third-party dependency or a
  shadcn-internal self-reference, rather than a "no resolvable Base UI
  equivalent" message

### Requirement: Prop-name normalization prevents false-positive missing props
`prop-parity` SHALL compare adico's snake_case prop names against upstream
prop names using a fixed, hand-maintained normalization table covering at
minimum: camelCase-to-snake_case casing, the `className`-to-`class` rename,
React-only structural props (`render`, `style`, `nativeButton`, `inputRef`,
`asChild`) classified `intentional_difference`, and Dioxus-structural props
(`children`, `attributes: Vec<Attribute>` extension) matched against
adico's corresponding structural fields rather than compared by name. This
table SHALL NOT be inferred heuristically at comparison time.

#### Scenario: A casing-only difference is not reported missing
- **WHEN** upstream declares `onOpenChange` and adico declares
  `on_open_change`
- **THEN** `prop-parity` classifies it `present`, not `missing`

#### Scenario: A React-only structural prop is not reported missing
- **WHEN** upstream declares `render` or `asChild`
- **THEN** `prop-parity` classifies it `intentional_difference` with a
  fixed reason, not `missing`

#### Scenario: A genuine gap is reported missing
- **WHEN** upstream declares a prop with no adico counterpart after
  normalization and no entry in the structural-props table
- **THEN** `prop-parity` classifies it `missing`
