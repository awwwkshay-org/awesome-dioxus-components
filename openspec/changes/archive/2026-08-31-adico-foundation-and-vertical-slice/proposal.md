## Why

`build-adico-component-ecosystem` has grown into an eleven-milestone, ~130-task change that
mixes fully-complete foundational work with still-open, actively-evolving scope (M4 parity
hardening, M6 shared primitive expansion). That makes the change hard to review and hard to
keep `openspec validate --strict` meaningful: a reviewer checking "is the CLI installation
contract done" has to wade through unrelated in-progress primitive work to find out.

M0 (repository/architecture foundation), M1 (upstream inventory and primitive ownership), M2
(registry and CLI vertical slice), and M3 (existing component migration) are 100% complete —
62/62 tasks done, each already recording its own verification evidence. This change extracts
exactly that finished scope into its own archivable unit, so the synced main specs reflect
what's actually shipped and `build-adico-component-ecosystem` shrinks to only the work still
in flight (M4–M10).

## What Changes

- Move the M0–M3 task groups (currently `## 1.`–`## 4.` in
  `build-adico-component-ecosystem/tasks.md`, task numbers `1.1`–`4.9`) into this change's own
  `tasks.md`, verbatim, preserving their exact numbering and evidence text — several files
  under `docs/adico/` and `UPSTREAMS.md` cite these task numbers by number (e.g. "task 4.6",
  "task 4.9") and must continue to resolve correctly.
- Move the five capability delta specs whose requirements were established entirely by
  M0–M3 work into this change: `adico-workspace-and-provenance`, `adico-registry`,
  `adico-project-configuration`, `adico-cli-installation`, `adico-component-validation`.
- Sync those five capabilities into `openspec/specs/` and archive this change immediately,
  so they become the durable record of finished, load-bearing behavior.
- `build-adico-component-ecosystem` is edited in a follow-up pass (not part of this change)
  to remove the now-duplicated M0–M3 task groups and the five extracted spec deltas, leaving
  only M4–M10 and the two capabilities still actively produced by that in-progress work
  (`adico-primitives`, M6-sourced; `adico-example-fixtures`'s existing MODIFIED delta, which
  belongs to a different, already-archived change and is untouched either way).

**Explicitly not in this change:** M4 (parity hardening) and M6 (shared primitive expansion)
task groups, and the `adico-primitives` capability — those requirements were produced by
M6 work (design.md §8a, the 2026-08-30 shadcn props parity audit), not M0–M3, and stay in
`build-adico-component-ecosystem` since that work is still open.

## Capabilities

### New Capabilities
- `adico-workspace-and-provenance`: workspace product boundaries, owned/independently
  releasable primitives, provenance/attribution obligations for reused source, and
  platform-specific-behavior isolation (M0/M1).
- `adico-registry`: the registry item schema, transitive dependency resolution, curated
  organization registries, compatibility declarations, and reproducible build output (M2).
- `adico-project-configuration`: `components.json` shape, project detection, and
  marker-region-scoped module editing (M2).
- `adico-cli-installation`: `adico list`/`view`/`init`/`add` behavior, Cargo edit safety,
  non-destructive overwrite handling, and reviewable installation results (M2, plus M3's
  CLI hardening in tasks 4.8h–4.8j).
- `adico-component-validation`: consumer-style fixture/installer coverage, registry-switching
  coverage, progressive example growth, playground theme customization, the
  hardening-before-new-migration rule, layered interactive-component coverage, and honest
  platform-result reporting (M2/M3).

### Modified Capabilities
(none)

## Impact

- **OpenSpec:** five capabilities newly synced into `openspec/specs/`; this change archived
  immediately after validation. `build-adico-component-ecosystem` is edited afterward (as a
  separate, explicit follow-up, not bundled into this change) to drop the now-redundant
  content.
- **No code changes.** This is a planning-artifact reorganization only — nothing under
  `packages/`, `registry/`, `apps/`, or `examples/` changes as a result of this change.
- **Traceability preserved:** original task numbers (`1.1`–`4.9`) are kept exactly as written
  so existing `docs/adico/*.md` and `UPSTREAMS.md` references remain valid.
