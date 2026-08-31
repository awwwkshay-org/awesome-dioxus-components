## Context

This change is a pure extraction of already-completed work; see `proposal.md` - Why for the
motivation. The technical decisions behind M0–M3 (workspace layout, upstream primitive
ownership/provenance, registry schema and resolution, CLI installation flow, existing-
component migration strategy) were made and recorded in
`openspec/changes/build-adico-component-ecosystem/design.md` at the time each milestone was
implemented. Rather than re-deriving them, this change references those sections directly:
§1–§5 (roughly) cover workspace/provenance/registry/CLI decisions; the M3 migration-wave
rationale is further recorded in `docs/adico/m3-migration-queue.md` and the per-wave
`docs/adico/m3-wave*-migration.md` reports.

## Goals / Non-Goals

**Goals:**
- Sync the five capabilities whose requirements were fully established by M0–M3 into
  `openspec/specs/` as the durable record of finished behavior.
- Preserve exact task numbering and evidence text from the original `tasks.md`, since
  `docs/adico/*.md` and `UPSTREAMS.md` cite these task numbers directly.

**Non-Goals:**
- Re-deriving or second-guessing the M0–M3 design decisions themselves — this change moves
  planning artifacts, it does not revisit implementation choices.
- Touching `adico-primitives` or `adico-example-fixtures` — both are produced/modified by
  work outside M0–M3 scope (see proposal.md's Capabilities section) and stay in
  `build-adico-component-ecosystem`.
- Any code change. This change's `impact` is entirely to `openspec/` planning artifacts.

## Decisions

**Capability attribution was done per-requirement, not per-file.** Each of the seven original
capability deltas in `build-adico-component-ecosystem/specs/` was read in full and checked
against the M0–M3 task text (including its "Done <date>: ..." evidence notes) to confirm every
requirement in the five extracted capabilities was actually produced by M0–M3 work, not merely
still being exercised by later milestones. `adico-component-validation`'s requirements (fixture
coverage, progressive examples, playground theming, hardening-before-migration, layered
behavior coverage, honest reporting) read as ongoing obligations established during M0–M2 that
M4+ continues to *satisfy*, not requirements that M4+ *introduced* — so the whole capability
moves, and `build-adico-component-ecosystem` no longer needs its own delta for it unless a
future milestone actually changes one of these requirements' text.

**`adico-primitives` and `adico-example-fixtures` were deliberately left out**, even though
extracting "everything M0–M3 touched" might seem simpler. `adico-primitives`' four
requirements explicitly cite `design.md §8a` and the 2026-08-30 shadcn props parity audit —
both M6-era artifacts — so none of it was actually produced by M0–M3. `adico-example-fixtures`
is not an ADDED delta at all here; it is a MODIFIED delta against an already-synced capability
from a different, already-archived change, referencing the 2026-08-30 `examples/desktop`
removal — unrelated to this extraction and left untouched in
`build-adico-component-ecosystem`.

## Risks / Trade-offs

- **Task-number stability is load-bearing.** `UPSTREAMS.md` and multiple `docs/adico/*.md`
  files cite "task 4.6", "task 4.9", etc. by number. Mitigation: this change's `tasks.md` is a
  byte-for-byte copy of the original M0–M3 groups, not a renumbering — verified before archive
  that no external reference depends on the *group heading* numbers (`## 1.`–`## 4.`) changing,
  only the task numbers within them (`1.1`–`4.9`), which are preserved unchanged either way.
- **`build-adico-component-ecosystem` must still validate `--strict` after its own follow-up
  edit** removes the now-duplicated content. That edit is intentionally a separate step from
  this change (not bundled here) so this change's own archive/sync can be verified in
  isolation first.
