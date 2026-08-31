## Purpose

Every registry UI/component item's reusable interactive behavior is supposed
to come from `adico-primitives`, keeping registry source a thin styled
facade. This delta makes that expectation an explicit, per-item declaration,
recorded as an individually reviewable file, and a mechanically verified,
CI-gated invariant — closing the gap that `registry/ui/*.rs` is not a
compiled workspace member and so is otherwise invisible to `cargo check`.

## ADDED Requirements

### Requirement: Every registry item declares its behavior-ownership classification in its own record
Each `registry:ui` or `registry:component` item SHALL have its own committed
classification record, independent of `registry.json`'s consumer-facing
schema, declaring it as exactly one of: `delegated` (its interactive behavior
is fully owned by the `adico-primitives` module(s) it imports),
`presentational` (it has no interactive behavior that a primitive would own,
with a recorded reason), or `exception` (it has behavior an existing
`adico-primitives` module could own but deliberately does not delegate to
it, with a recorded reason and a named follow-up). A registry item MAY be
`exception` while still importing one or more `adico-primitives` modules for
its non-exceptional behavior.

#### Scenario: A component fully delegates its behavior
- **WHEN** a registry item's interactive behavior is entirely owned by the
  `adico-primitives` module(s) it imports, with no documented gap
- **THEN** its record's classification is `delegated`, and its record lists
  those imported modules

#### Scenario: A component has no interactive behavior
- **WHEN** a registry item's source contains no interactive-behavior markers
  (keyboard handling, focus management, scroll locking, controlled state, or
  an `adico_primitives` import)
- **THEN** its record's classification is `presentational`, with a recorded
  one-line reason

#### Scenario: A component deliberately does not delegate available behavior
- **WHEN** a registry item has behavior that an existing `adico-primitives`
  module could own, but the item does not delegate to it for a documented
  reason (for example, a browser-interop limitation, or a styled-facade
  gap with no matching headless primitive yet)
- **THEN** its record's classification is `exception`, with a recorded reason
  and follow-up, not silently left as `delegated` or `presentational`

### Requirement: Behavior-ownership records are mechanically verified and CI-gated
An adico-xtask command SHALL verify every registry item's record against the
actual state of `registry/ui/*.rs` and `registry.json`, running fully offline
with no dependency on fetched upstream catalogs, and independent of a given
item's `delegated`/`exception` label. This command SHALL run as part of
continuous integration, not only as a locally invocable check. In addition to
the aggregate check, each registry item SHALL have its own named regression
test asserting its record still agrees with its real source.

#### Scenario: A record disagrees with the declared dependency
- **WHEN** a registry item's `registry.json` entry declares the
  `adico-primitives` cargo dependency but that item's record lists no
  `adico-primitives` modules, or the reverse
- **THEN** the verification command fails and identifies the item, whether
  that item's record is classified `delegated` or `exception`

#### Scenario: A recorded module import does not exist
- **WHEN** an item's record lists an `adico_primitives` module that has no
  corresponding file under `packages/adico-primitives/src/`
- **THEN** the verification command fails and identifies the missing module

#### Scenario: A presentational item gains interactive behavior
- **WHEN** an item recorded as `presentational` is later changed to contain
  an interactive-behavior marker (keyboard handling, focus management,
  scroll locking, controlled state, or an `adico_primitives` import) without
  its record being updated
- **THEN** the verification command fails, rather than silently accepting the
  stale record, and that item's own named regression test fails independently
  of the aggregate check

#### Scenario: Registry source duplicates primitive-owned page scroll locking
- **WHEN** a registry item's source injects a page-level scroll or overflow
  style (such as `html { overflow: hidden; }`) while that item's record lists
  a primitive module that already owns scroll locking for that component
- **THEN** the verification command fails and identifies the duplicated style

#### Scenario: An exception or presentational record has no reason
- **WHEN** a registry item's record is classified `presentational` or
  `exception` and its `reason` field is empty, or classified `exception` with
  an empty `followUp` field
- **THEN** the verification command fails and identifies the incomplete
  record

#### Scenario: The verification command runs without network access
- **WHEN** the verification command runs with no network access available
- **THEN** it completes successfully using only committed repository state,
  and does not read or require any `statics/catalogs/*.json` file

#### Scenario: A pull request removes a required primitive dependency
- **WHEN** continuous integration runs on a change that removes the
  `adico-primitives` cargo dependency from a registry item's
  `cargoDependencies` without updating that item's record
- **THEN** the continuous integration job fails, and that item's own named
  regression test is the one that identifies it

### Requirement: Every registry item declares its styling classification
Each `registry:ui` or `registry:component` item SHALL have its own committed
styling record, independent of `registry.json`'s consumer-facing schema,
declaring two independent classifications: whether it styles exclusively
through Tailwind utility classes (`tailwindOnly`) or has one or more
documented, bounded exceptions for a value that cannot be expressed as a
static Tailwind class (a runtime-computed value), and whether every themable
color it uses is a semantic design token (`tokenCompliant`) or has one or more
documented, bounded exceptions, each recorded with a reason and, where
applicable, the exact upstream source it reproduces.

#### Scenario: A component has no dynamic styling values
- **WHEN** a registry item's styling is fully expressed through static
  Tailwind utility classes, with no runtime-computed style value
- **THEN** its record's `tailwindOnly` is true, with no style exception
  entries

#### Scenario: A component has a genuinely dynamic styling value
- **WHEN** a registry item must set a runtime-computed value that cannot be a
  static Tailwind class (for example, a computed progress-indicator width or a
  generated custom property)
- **THEN** its record's `tailwindOnly` is false, with a recorded exception
  describing the value and why it cannot be static

#### Scenario: A component uses a themable color through a semantic token
- **WHEN** a registry item's themable color is expressed as a semantic
  design-token class (for example `bg-primary`, `text-foreground`,
  `border-input`)
- **THEN** its record's `tokenCompliant` is true for that use, with no color
  exception needed

#### Scenario: A component uses a non-token color for a documented reason
- **WHEN** a registry item uses a raw color value or a Tailwind
  default-palette color class instead of a semantic token
- **THEN** its record's `tokenCompliant` is false, with a recorded exception
  giving the value used and the reason it is not a token (for example, an
  exact match to a cited upstream shadcn source, or an inherently
  theme-independent affordance)

### Requirement: Styling classification is mechanically verified and CI-gated
An adico-xtask command SHALL verify every registry item's styling record
against the actual state of `registry/ui/*.rs`, running fully offline with no
dependency on fetched upstream catalogs. This command SHALL run as part of
continuous integration, not only as a locally invocable check. In addition to
the aggregate check, each registry item SHALL have its own named regression
test asserting its styling record still agrees with its real source.

#### Scenario: A tailwind-only item contains a static raw style
- **WHEN** an item's record declares `tailwindOnly: true` but its source
  contains a `style { ... }` block or a `style:` attribute with no dynamic
  content and no matching exception
- **THEN** the verification command fails and identifies the item and the
  offending style

#### Scenario: A token-compliant item contains an unrecorded non-token color
- **WHEN** an item's record declares `tokenCompliant: true` but its source
  contains a raw hex/rgb color or a Tailwind default-palette color class with
  no matching exception
- **THEN** the verification command fails and identifies the item and the
  offending color

#### Scenario: A styling or token exception has no reason
- **WHEN** a registry item's style or color exception entry has an empty
  reason
- **THEN** the verification command fails and identifies the incomplete
  exception

#### Scenario: The verification command runs without network access
- **WHEN** the verification command runs with no network access available
- **THEN** it completes successfully using only committed repository state,
  and does not read or require any `statics/catalogs/*.json` file

#### Scenario: A pull request reintroduces a raw style with no exception
- **WHEN** continuous integration runs on a change that adds a static `style`
  block or attribute to a `tailwindOnly` item without recording a matching
  exception
- **THEN** the continuous integration job fails, and that item's own named
  regression test is the one that identifies it
