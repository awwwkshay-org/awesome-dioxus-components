# M4 acceptance (task 5.6)

Status: partial — every dimension for every audited component has a
complete-with-evidence status, an explicit deferral with a reason, or a
named residual-scope note; nothing is silently omitted. M4 is not fully
closed: real audited-but-unfixed scope remains (below), consistent with
task 5.6's own permissive text ("complete required parity dimensions or
visible, justified in-progress status with a dependency-group task").

This records task 5.6, closing out the M4 sub-batches (5.1's audit, 5.2a/5.2b,
5.3a-5.3d, 5.4) against `docs/adico/m4-parity-audit.md`'s original 38-component,
10-dimension matrix. It does not re-derive that matrix; it records what moved
since.

## What M4 closed

- **api / visual (interaction-state classes)** — 5.2a closed the missing
  focus-visible-ring/disabled-state gap the audit found real for `accordion`,
  `tabs`, `pagination`, `menubar`, plus `color-picker`'s `AreaThumb` (found
  during the same pass, not in the original 13). The audit's other 8
  candidates (`collapsible`, `context-menu`, `hover-card`, `popover`,
  `tooltip`, `dialog`, `sheet`, `mode-toggle`) were re-verified per-component
  against upstream and confirmed matches, not gaps — upstream's own bare
  triggers carry the same zero default styling, or the item already composes
  a fully-treated sibling component (`Button`, `DropdownMenuTrigger`).
- **api / composition-structure (prop-shape, slots)** — 5.2b diffed 22 of the
  38 components against current upstream source (not just interaction-state
  classes) and found 8 genuine gaps, now fixed: `dialog`/`sheet` (default
  close button, `DialogClose`/`SheetClose`, `DialogFooter`), `card`
  (`CardAction` slot), `badge` (`Ghost`/`Link` variants), `toggle`
  (`ToggleVariant`), `toggle-group` (`ToggleItemVariant`/`Size`), `switch`
  (`SwitchSize`), `avatar` (`AvatarSize`). 13 of the 22 were confirmed fine
  with no gap. See below for the 16 not yet given this diff.
- **visual (transitions)** — 5.3a closed the missing open/close
  transition-animation gap for `context-menu`, `hover-card`, `menubar`,
  `popover`, `tooltip` (ported the missing Tailwind animation utilities from
  `tw-animate-css` into `css.rs`, since the underlying utility layer, not
  just the per-component classes, was missing). `accordion`'s equivalent
  transition was attempted twice (5.3a and again in this task) and reverted
  both times — see "Explicitly not closed" below.
- **visual / states (slider)** — 5.3b fixed `slider`'s invisible thumb
  (an `overflow-hidden` clipping bug, not a missing class).
- **visual (label/input spacing)** — 5.3b determined this is an example
  composition responsibility, not a component defect (matches upstream's own
  source exactly), and fixed the three affected consumer usages.
- **Registry-source formatting gate** — 5.3c found and closed a real,
  previously-silent gap: `registry/ui/*.rs` isn't a Cargo workspace member,
  so `cargo fmt --all` never touched it, letting non-canonical formatting
  drift in unnoticed until a consumer installed it. Added a permanent
  `cargo xtask registry validate` gate against this class of drift.
- **Semantic-token contract** — 5.3d refreshed `css.rs`'s installed CSS
  contract to current upstream token names and radius scale (`--chart-1..5`,
  `--sidebar` rename, expanded radius scale), with an explicit, evidenced
  decision to stay HSL rather than migrate to upstream's now-OKLCH color
  model (a materially bigger, more disruptive change than this milestone's
  mandate).
- **keyboard / focus / ARIA / layering** — 5.4 found most of this dimension
  was already closed by the separately-completed, separately-archived
  `reauthor-primitives-from-independent-spec` (every primitive re-authored
  against WAI-ARIA APG patterns). Re-verifying that change's own "known gaps"
  rather than trusting the record found its top-priority gap misdiagnosed:
  `positioner.rs`'s measurement was blamed, but the real defect was a stale
  `visibility: hidden` CSS property Dioxus's style-attribute patching never
  cleared once set. Fixed — `popover`/`hover-card`/`tooltip`/`select`/
  `combobox` (every `Positioner`-anchored component) are no longer stuck
  invisible after opening. This was named "the single largest gap" in the
  prior change's own acceptance record.

## Explicitly not closed, with reasons (not silently dropped)

- **`accordion`'s open/close transition.** Tried twice (5.3a: `height: auto`
  keyframe, reverted — `AccordionContent` never sets a measured
  content-height custom property, so the keyframe sticks at `height: 0`;
  this task: a CSS grid `0fr`/`1fr` technique, reverted — it only animates
  the close direction, since `AccordionContentPrimitive` mounts already in
  its final open `data-open="true"` state with no intermediate frame for the
  browser to transition from). A symmetric fix needs primitive-level
  two-phase mount timing on the shared `use_animated_open` hook, used by 5
  other already-verified components — too broad a blast radius to risk in
  this milestone. Left as a named, understood gap.
- **RTL and responsive validation.** No test infrastructure exists for
  either (no RTL-flip harness, no viewport/breakpoint fixture system).
  Decided explicitly (see `tasks.md`'s 5.3 entry) that building either well
  is disproportionate to M4's hardening mandate, and RTL support itself is
  M6 primitive-layer scope (`direction.rs`), not something the registry/UI
  layer has anything to test against yet. Recorded as a **permanent**
  `unmeasurable (no test harness exists)` status for the M3 migrated set,
  not a temporary gap this milestone owes a fix for.
- **Desktop rendering/interaction validation.** Compilation is exercised
  (primitive-layer `feature = "native"` gates exist and compile), but no
  desktop fixture exists to test rendering/interaction against
  (`examples/desktop` was removed, an already-recorded decision). Recorded
  as `unmeasurable (no fixture exists)`, matching this repo's existing
  platform-honesty convention (see `adico-component-validation`'s "Platform
  results are reported honestly" requirement).
- **The other symptom-class of `document::eval` defects.** Confirmed
  independently three times now (this session's own `use_focus_trap`/
  `use_scroll_lock` verification, the prior change's `use_outside_dismiss`
  finding, and `pointer.rs`'s global drag tracker) that some evals never fire
  their callback at all — a different failure mode than `positioner.rs`'s
  (which fired correctly but left a stale style). Not attempted here,
  consistent with the prior team's own explicit deferral to a dedicated
  follow-up change; a blind fix given three independent confirmations of a
  real structural defect risks being wrong in a new way.

## Genuinely open, unaudited or unaddressed (residual scope for future sub-batches)

- **16 of the 38 components have no prop-shape/composition diff yet**:
  `calendar`, `date-picker`, `mode-toggle`, `sidebar` (a 726-line/24-export
  upstream file — a substantial standalone effort), `toast` (upstream
  deleted `toast.tsx` entirely at the pinned revision, moving to a separate
  `sonner` package — no like-for-like source to diff against, needs its own
  decision), plus the `select`/`alert-dialog`/`combobox` group (spot-checked
  at export-list level only, not fully diffed) and the
  `dropdown-menu`/`context-menu`/`menubar`/`popover`/`hover-card`/`tooltip`
  family (leaned on the primitive re-authoring's own findings, not
  independently re-diffed against upstream TSX for this dimension).
- **`variants`/`states` were never independently audited** for any of the 38
  components beyond the specific defects 5.2a/5.3b happened to find
  (`m4-parity-audit.md` marked this dimension `U` uniformly and named it
  explicit residual scope for 5.3; still true).
- **Light-mode re-verification.** Every live-rendering check this session
  and the original audit performed was in dark mode (the example's default).
  No component has had its light-mode rendering independently confirmed.
- **`docs` (task 5.5): entirely untouched, systemic 38/38 gap**, exactly as
  the original audit found it. Every registry item still carries only a
  one-line `documentation.compositionNote`; there is no per-component usage
  example, accessibility/keyboard note, or dedicated doc file anywhere in
  this repo, and `apps/docs` remains a stub. This is the largest single
  piece of named-but-unstarted M4 scope. Given its size (38 components,
  each needing real composition examples and accessibility/keyboard notes,
  not generated boilerplate), it is recorded here as its own dependency-group
  task rather than attempted as a checkbox in this pass.

## Verification

```sh
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask
cargo test --locked --workspace
cargo run -p adico-xtask -- registry validate
cargo run -p adico-xtask -- provenance check
cargo run -p adico-xtask -- primitive-compat check
cargo run -p adico-xtask -- component-compat check
cargo run -p adico-xtask -- primitive-usage check
cargo run -p adico-xtask -- styling-usage check
openspec validate build-adico-component-ecosystem --strict
```

All of the above pass as of this record. `cargo clippy`/`cargo test
--workspace` (the full workspace, feature-unified) are included deliberately,
not just the package-scoped baseline: this session found that scoped
commands alone had let two latent test bugs go undetected on `main` since
before this milestone (fixed as part of closing out this record, unrelated
to any M4 component finding).

Not run/verified in this pass, named rather than silently skipped: light-mode
rendering (see above), RTL/responsive/desktop harnesses (do not exist, see
above), `tests/playwright`'s wave2-5/mode-toggle/theme-switcher/fullstack
suites (not re-run since the primitive re-authoring change's own closing
sweep already named this gap; still true).

## Acceptance statement

Every dimension for every one of the 38 M3-migrated, shadcn-equivalent
components carries one of: cited evidence of a match, a cited real gap that
was found and fixed with evidence, or an explicit, reasoned deferral —
consistent with `docs/validation.md`'s honesty requirement that no report
claim a skipped surface as a pass. M4 is not declared fully complete: 16
components' composition/API parity, all 38 components' `variants`/`states`
and light-mode rendering, and the entire `docs` dimension remain named,
unstarted or partially-started residual scope, tracked here rather than
force-completed. Whoever resumes this work should treat this document, not
`m4-parity-audit.md` alone, as the current source of truth for what M4
actually closed.
