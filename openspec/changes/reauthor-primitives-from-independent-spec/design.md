## Context

See `proposal.md` - Why for the motivation and the license-obligation constraint that forces
this change to precede any deletion of the provenance apparatus.

Current state, verified:
- `packages/adico-primitives/src`: 71 files, 24,510 lines. 64 files / 22,051 lines recorded
  as imported across 9 `provenance/records/*.json` entries. 6,794 of those lines (31% of the
  imported set) are byte-identical to their import commit; 24 files have zero edits since
  import. 8 files (2,459 lines) are already originally authored in-repo:
  `positioner.rs`, `menu.rs`, `layer.rs`, `gesture.rs`, `scroll_lock.rs`, `direction.rs`,
  `theme_mode.rs`, `time.rs`.
- Only 17 of the 64 imported files have any `#[test]`; 23 of the 24 zero-churn files have
  none. Playwright coverage totals 49 tests across 14 specs for ~44 registry components.
- `select/` (9 files), `combobox/` (8 files), `virtual/` + `virtual_list.rs` (5 files), and
  `color_picker.rs` + `color_picker/color_naming.rs` are the only primitives still split
  across a directory rather than a single file.
- `openspec/changes/build-adico-component-ecosystem` (still active, unarchived) already
  established the target architecture in `design.md` §8a and has completed tasks 7.2-7.8c,
  producing the 8 owned files above. Tasks `7.8d` (select/combobox onto
  `positioner`/`layer`), `7.8e` (menu family onto `menu.rs`), and `7.9` (Base-UI-parity tier)
  are still open in that change and are treated here as prerequisites/inputs for two of this
  change's waves, not duplicated.
- `packages/adico-xtask/src/main.rs`'s `check_provenance()` (`:342-414`) and
  `collect_imported_paths()` (`:428-452`) already implement exactly the invariant this change
  needs as a progress gate: every upstream-attributed file must have a record, and every
  record's paths must still carry their revision string. No tool changes are required.

## Goals / Non-Goals

**Goals:**
- Every file currently recorded as imported is re-authored from an independent
  specification, with no upstream implementation file open during authoring.
- Every primitive ends up feature-comparable to both Base UI and dioxus-primitives, pinned to
  the revisions in `statics/catalogs/`.
- Every primitive is one file.
- The crate's public API (`adico_primitives::<name>::*`) is unaffected by directory
  flattening; only internal module layout changes.
- `cargo xtask provenance check`'s reported count reaches zero imported records/files by the
  end of this change.

**Non-Goals:**
- Deleting the provenance apparatus itself (records, schema, xtask command, SPDX headers,
  `third_party/`, the registry `provenance` field) — that is `remove-provenance-tracking`,
  gated on this change's completion.
- Changing `primitive-compat`/`component-compat`/`catalog fetch` tooling or the
  `statics/catalogs/*` / `statics/*_compatibility.json` data — only consumed, never modified.
- Achieving native/desktop or Android/iOS test coverage — no fixture exists for either today
  (`docs/validation.md`), and this change does not add one.
- Claiming "clean-room" status — see the framing note below.

## Decisions

**Framing: "re-authored from an independent specification," not "clean-room."** A true
clean-room process requires an unexposed second author who never saw the original; that is
not achievable here since the derived text has already been read. What is achievable, and
what this change requires per the spec's second requirement, is authoring from behavioral
contracts rather than from copied expression: the WAI-ARIA APG as primary authority, Base
UI's public anatomy/docs (already the repo's declared target, `design.md` §8a in the other
change) and dioxus-primitives' public API as feature checklists, and this repo's own tests as
the executable contract. Consulting a project's documented API surface to decide what to
build is ordinary interoperability; copying its implementation text is what creates a
derivative. Every rewritten file's target behavior is recorded so the claim is auditable, not
asserted.

**Synthesis, not mirroring: each rewritten primitive is a one-stop-shop union, not a copy of
either reference.** Where `primitive-compat diff` finds a gap against Base UI or
dioxus-primitives, the goal is not to match whichever reference happens to be "correct" — it's
to combine both references' capabilities into adico's own shape, using the ARIA APG as the
tie-breaker wherever the two references disagree or a straight union would be incoherent. This
is a standing instruction for every wave, not just background motivation for the change as a
whole. Internal-only files that no compat axis tracks as a component or util — `collection.rs`
is the concrete example, consumed by roughly a dozen components but absent from
`statics/primitive_compatibility.json` entirely — have no compat-diff gap to close; their spec
is the ARIA APG pattern plus their actual consumers' needs, and any task's "close the compat
gap" step is a no-op for them, not a blocker.

**All tests live in `packages/adico-primitives/tests/`, not inline in `src/*.rs`.** Every
`#[test]` for this crate — including tests of currently-private state (for example
`collection.rs`'s `register_item`/`unregister_item`/`CollectionItemState`) — is authored as a
black-box test in `packages/adico-primitives/tests/test_<file-stem>.rs` (one file per rewritten
source module, filename pattern `test_*.rs`; the existing `tests/public_api.rs` is renamed
`tests/test_public_api.rs` to match). Since `tests/*.rs` compiles as a separate crate and can
only see `pub` items, this requires widening whatever internal API a test needs to reach to
`pub` — a deliberate, repo-wide override of ordinary Rust white-box-testing practice, adopted
because the user directed it explicitly, and accepted as a trade-off (see Risks/Trade-offs)
against keeping every rewritten file's test scaffolding out of its own source file.

**Sequencing: dependency depth first, then how clearly derivative a file is.** Rationale:
files with dependents (the shared internals) must be re-specified before their consumers are
rewritten against them, and the largest-line/zero-churn files carry the most derivation
exposure per line, so proving the recipe on cheap files first (Wave A) de-risks the expensive
ones.

- **Wave A** — near-free wins, already mostly rewritten by prior work: `collection.rs`,
  `js/focus-trap.js`, `typeahead.rs`. Purpose: prove the five-step recipe (below) and the
  provenance burn-down mechanics cheaply before scaling.
- **Wave B** — menu family: `dropdown_menu.rs`, `menubar.rs`, `context_menu.rs`, re-authored
  onto the existing `menu.rs`/`positioner.rs`/`layer.rs`. This is the same migration as tasks
  `7.8d`/`7.8e` in `build-adico-component-ecosystem`; this change's tasks reference those
  rather than re-planning the same work.
- **Wave C** — shared internals other primitives depend on: `portal.rs`, `pointer.rs`,
  `move_interaction.rs`, `listbox.rs`, `selection.rs`, `selectable.rs`.
- **Wave D** — large zero-churn leaves (highest per-line exposure): `tag_group.rs`,
  `toast.rs`, `toolbar.rs`, `scroll_area.rs`, `tabs.rs`, `radio_group.rs`, `toggle_group.rs`,
  plus flattening `virtual/*` → `virtual_list.rs`.
- **Wave E** — remaining files grouped by their existing provenance record (wave2-state,
  wave2-simple, wave2-roving-focus, wave2-risk, wave3-overlays, wave4-collection,
  wave5-extras, dialog-select), including the `select.rs`/`combobox.rs`/`color_picker.rs`
  flattenings. Grouping by record keeps each wave independently verifiable and lets its
  record die as a unit.
- **Wave F** — `lib.rs`, last, deliberately: it is the crate facade (489 lines, 25 commits,
  `+486/-3` since import) and its header covers upstream's module-declaration file; it clears
  once every module it declares has been rewritten/flattened in the earlier waves. Also
  removes its `#![allow(dead_code)]` / `#![allow(clippy::collapsible_if)]` blocks, whose
  `reason` strings cite "the source-preserving initial fork" — budget for a possible warning
  surface under `-D warnings` as a separate task, not a line item of the `lib.rs` rewrite.
- **Wave G** — `packages/adico-cli/src/css.rs` (parallel, independent of the primitives
  waves): re-derived from Tailwind v4 `@theme`/`@keyframes` documentation.
- **Closing task** — after all waves, run `primitive-compat diff` across all axes and
  implement any remaining gap (this is task `7.9`'s scope in the other change; referenced,
  not duplicated).

**Per-file recipe (five steps, applies to every task in Waves A-G):**
1. Derive the behavior spec: ARIA APG pattern + the primitive's row in
   `statics/primitive_compatibility.json` for the Base UI/dioxus-primitives gap.
2. Write or extend tests from that spec, before rewriting the implementation, in
   `packages/adico-primitives/tests/test_<file-stem>.rs` — never as an inline `#[cfg(test)]`
   module in `src/*.rs`. Widen any item a test needs to reach that is currently private
   (a struct, field, or method) to `pub`.
3. Author one flattened file implementing the spec, without the upstream file open.
4. Close the parity gaps the compat diff named.
5. Atomically, in the same commit: drop the file's upstream header lines and remove its entry
   from the owning provenance record's `localPaths`; delete the record file if that empties
   it (required — `provenance/schema.json` sets `minItems: 1` on `localPaths`, and
   `packages/adico-xtask/src/main.rs:375-377` hard-errors on an empty array).

**Provenance-check-as-burn-down-gate.** No tool changes needed. `check_provenance()` already
errors in both useful directions: an attributed file missing from a record (`:402-408`), and
a recorded path whose file no longer contains the revision string (`:389-394`) — the latter
is what forces step 5 to be atomic, since dropping a header without updating its record fails
the build immediately. The gate's own printed line
(`"provenance check passed: {N} imported record(s), {M} source unit(s)"`) is the burn-down
number. Target: `0 imported record(s), 0 source unit(s)`, with only the all-zero
`example-dioxus-components.json` schema fixture remaining untouched (it is skipped by
`check_provenance()` at `:364-366` and does not represent real imported code).

**Registry follow-through is part of the same task, not a separate pass.** `registry/ui/*.rs`
(44 files) are thin `pub use adico_primitives::*` facades with zero upstream attribution of
their own, and are never `cargo check`ed in isolation (`CLAUDE.md`). Any rewrite that changes
a primitive's public API to close a parity gap must update the facade,
`registry/registry.json`, and regenerated `registry/generated/**` in the same task
(`cargo xtask registry build` then `registry validate`), and must be proven by a real
consumer fixture install — never by checking `registry/*.rs` alone.

## Risks / Trade-offs

- **Effort is large** (~22k lines, comparable to prior M0-M6 milestones) → sequenced into
  independently-verifiable waves so the change is reviewable and pausable rather than one
  monolithic diff.
- **Regression risk is concentrated in untested zero-churn files** (23 of 24 have no unit
  tests) → mitigated by making test authorship step 2 of the recipe, before step 3's rewrite,
  for every task.
- **Public-API drift silently breaking `registry/ui/*.rs` facades** (never checked in
  isolation) → mitigated by running `cargo xtask primitive-compat check` after every wave as
  an acceptance gate; any diff must be an intentional, task-recorded API change.
- **Paraphrase instead of genuine re-authorship is not mechanically detectable** — a diff
  against upstream is not a meaningful test, since structural similarity is expected for the
  same ARIA pattern. Mitigation is procedural only: spec written first, upstream file closed
  during authoring, and each task records the specification it was written against. State
  this limitation plainly rather than claiming certainty the process cannot provide.
- **`lib.rs`'s allow-block removal could surface many warnings** under `-D warnings` once
  "the source-preserving initial fork" justification no longer applies → scheduled as its own
  Wave F task with its own budget, not folded into the `lib.rs` rewrite estimate.
- **Flattened `select.rs`/`combobox.rs` may land very large** → the escape hatch is
  extracting a genuinely shared cross-primitive primitive (as `typeahead.rs` was already
  extracted from `select/text_search.rs`), never a private sub-module of the component.
- **Externalizing all tests widens the crate's public surface.** Making previously-private
  state (item registration, internal item structs, helper predicates) `pub` so
  `tests/test_*.rs` can reach it means more of each primitive's implementation detail becomes
  technically importable by consumers, not just its intended API. Mitigated by noting in the
  doc comment of anything widened solely for testability that it is an implementation detail,
  not part of the primitive's intended public surface — accepted anyway as the standing
  convention this change adopts.
- **Parity scope could grow without bound** against two live upstream projects → fixed by
  measuring against the pinned revisions in `statics/catalogs/`, refreshed only by explicit,
  separate `catalog fetch` runs, not implicitly during this change.

## Migration Plan

No runtime migration: this is source-level rewriting behind an unchanged public API, verified
per wave by the full baseline validation suite (`cargo fmt`, `cargo check --workspace`,
`cargo clippy -D warnings`, `cargo test`, `openspec validate --strict`), the wasm32 target
check, Playwright, `provenance check`, `primitive-compat check`, and a real consumer fixture
install. No rollback beyond normal git revert is needed since nothing outside this crate and
its registry facades changes shape. `remove-provenance-tracking` is the follow-up change that
performs the actual apparatus removal once this change is archived and the gate reads zero.
