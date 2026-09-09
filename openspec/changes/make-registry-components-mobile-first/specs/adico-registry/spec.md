## ADDED Requirements

### Requirement: Registry file checksums can be computed mechanically
`adico-xtask` SHALL provide a `registry checksums --write` subcommand that
computes the SHA-256 checksum of every file an item in `registry.json`
references and writes the result into that item's `files[].checksum` field.
This subcommand SHALL NOT replace `registry validate`'s role as the checksum
gate — `registry validate` continues to fail when a checked-in checksum does
not match its file's actual content; the write subcommand only removes the
need to compute and transcribe that value by hand.

#### Scenario: A registry source file changes
- **WHEN** a contributor edits a file referenced by an item in `registry.json`
  and runs `registry checksums --write`
- **THEN** that item's `files[].checksum` entry is updated to the new file's
  SHA-256 digest, and unrelated items' checksums are left unchanged

#### Scenario: Checksums are written but the registry is not rebuilt
- **WHEN** `registry checksums --write` has updated `registry.json`'s
  checksums
- **THEN** `registry validate` still independently verifies every checksum
  against its file's current content, so a stale or hand-edited checksum is
  still caught
