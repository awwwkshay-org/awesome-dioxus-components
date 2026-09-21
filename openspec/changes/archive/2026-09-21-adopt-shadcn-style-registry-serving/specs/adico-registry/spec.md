## MODIFIED Requirements

### Requirement: Registry items describe installable source
Each registry item SHALL have a stable name, item category, description, source
files, target-path intent, registry dependencies, Cargo dependency requirements,
and applicable style/theme requirements. The schema SHALL support UI,
component, hook, library, block, page, theme, file, and template categories,
even when an initial release supports only a subset. A source file entry SHALL
support an optional inlined content representation in addition to its source
path and checksum, so that a registry can be distributed as self-contained,
per-item documents without a separate fetch for each referenced file.

#### Scenario: A UI item has supporting requirements
- **WHEN** a consumer requests a registry UI item with a utility, runtime, and
  theme requirement
- **THEN** the registry can express all required source and non-source inputs
  without undocumented installer behavior

#### Scenario: A generated item document is self-contained
- **WHEN** a registry item's file entry carries inlined content
- **THEN** the installer can obtain that file's complete bytes from the item
  document alone, without a further fetch to the file's `source` path

### Requirement: Registry compatibility is explicit
The registry SHALL declare a format version and item compatibility information.
The installer SHALL reject a registry or item it cannot interpret and explain
the required compatible CLI/runtime range. A registry format version that adds
only an optional, additive representation of already-described data (such as
inlined file content alongside an existing source-path-and-checksum
representation) SHALL remain readable by the same installer that reads the
prior format version; the installer SHALL NOT require every registry it reads
to use the newest format version.

#### Scenario: Older CLI reads a newer incompatible registry
- **WHEN** the CLI encounters a registry format it does not support
- **THEN** it exits before file installation with an actionable compatibility
  error

#### Scenario: Installer reads a registry using the prior format version
- **WHEN** the CLI resolves a registry whose items describe files only by
  source path and checksum, with no inlined content
- **THEN** resolution and installation succeed exactly as before, fetching
  each file's bytes from its source path

#### Scenario: Installer reads a registry using the additive format version
- **WHEN** the CLI resolves a registry whose items describe files with
  inlined content
- **THEN** resolution and installation succeed using the inlined bytes,
  without fetching each file's source path separately

### Requirement: Registry build output is reproducible
The authored registry source and generated consumable metadata SHALL be
validated and reproducible from checked-in inputs. Generated output SHALL not
require a live network request to resolve local items during ordinary tests or
CI. Generated output SHALL include a self-contained, servable representation
suitable for hosting as static files with no server-side logic, derived
entirely from the authored registry source. A committed, generated fallback
representation used for offline resolution SHALL be verified as reproducible
from the same authored source, and continuous integration SHALL fail if that
committed fallback disagrees with a fresh regeneration.

#### Scenario: CI validates registry metadata
- **WHEN** repository validation runs without external-network availability
- **THEN** it can validate all checked-in registry metadata and source mappings

#### Scenario: A committed fallback representation goes stale
- **WHEN** a registry source file or `registry.json` entry changes without the
  committed offline-fallback representation being regenerated
- **THEN** continuous integration fails and identifies that the fallback
  disagrees with a fresh regeneration

## ADDED Requirements

### Requirement: Registry resolution cost is proportional to requested items
Resolving and validating a registry source for an `adico add`, `adico list`,
or `adico view` request SHALL cost proportionally to the items actually
requested (including their transitive registry dependencies), not to the
total number of items or files present in the configured registry.

#### Scenario: Listing a large registry does not fetch every item's files
- **WHEN** a consumer runs `adico list` against a configured registry with
  many items
- **THEN** the CLI enumerates the catalog without fetching or checksumming
  every item's individual source files

#### Scenario: Adding one item does not validate the whole catalog
- **WHEN** a consumer runs `adico add <item>` against a registry containing
  many items unrelated to `<item>` and its transitive dependencies
- **THEN** the CLI resolves and validates only `<item>` and its transitive
  registry dependencies, not the unrelated items
