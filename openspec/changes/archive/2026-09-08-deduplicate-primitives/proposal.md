## Why

`packages/adico-primitives/src` (66 files, 28,023 lines) has accumulated multiple
independent implementations of the same behavior, in direct violation of two
already-approved requirements:

- `openspec/specs/adico-primitives-authorship/spec.md:87-91` — behavior used by more
  than one primitive SHALL live in its own top-level primitive file, not be inlined
  into every consumer ("Behavior is genuinely shared across multiple primitives").
- `openspec/specs/adico-primitives/spec.md:29-33` — shared behavior reachable only
  through a private module or `pub(crate)` function does not satisfy the crate's
  "shared behavior is a public, documented primitive surface" requirement even when
  the behavior itself is correct, and is tracked as a promotion gap.

An audit of every module found the same hover-intent delay logic hand-written in
three separate files (each carrying a comment admitting it is "the same technique …
not shared code"), two near-identical hover-disclosure modules, two shared modules
kept `pub(crate)` despite 2 and 6+ in-crate consumers, four hand-rolled copies of the
crate's own `use_controlled`, and two overlays whose Escape handling silently skips
the shared dismissal-layer check — a real correctness bug, not only duplication.
Separately, `menubar`'s recorded exception from the shared `Positioner` cites a
capability gap that has since partially closed, and the spec text needs to catch up
to what is actually true today rather than being fixed by assertion.

Now, not later: every new registry component composes these primitives, so every day
this duplication survives is another day new consumers can copy the wrong instance,
and the crate's own authorship rules are already unenforced on this exact class of
violation.

## What Changes

- Add a new shared `hover_intent` primitive (generation-counter-debounced open/close
  delay) and migrate `menu`, `preview_card`, and `navigation_menu` onto it, generic
  over payload type and an opt-in `skip_delay_when` predicate so `navigation_menu`'s
  own behavior is preserved exactly.
- Merge `preview_card` into `hover_card` as the shared implementation, with
  `preview_card` becoming a thin facade supplying its own per-component defaults
  (`delay_ms`, `close_delay_ms`, `side`, `role`) rather than a bare re-export — no
  prop, default, or test may be dropped from either module.
- Promote `mod segment` and `mod time` from `pub(crate)` to `pub`, satisfying the
  existing spec requirement that segment input be a public, reusable primitive.
- Consolidate `ToolbarSeparator`, `MenuSeparator`, and `CommandSeparator` onto
  `separator::Separator`'s markup.
- Route `AccordionMulti` through the crate's existing `use_controlled`, and add a
  new sibling primitive, `use_optionally_controlled`, for the tri-state
  "controlled value is itself optional" pattern independently hand-written in
  `Accordion` (single), `selectable::use_single_selectable_value`, and
  `menu::MenuRadioGroup` — `use_controlled`'s single-level `Option` cannot
  distinguish "controlled, currently nothing selected" from "not controlled at
  all" without a real behavior change, so this is a second, correctly-scoped
  shared primitive rather than a forced consolidation onto the first (see
  `design.md`'s corrected D7 for the full analysis).
- Fix Escape-key handling in `menu.rs` (root `Menu`, and its submenu keydown path)
  and `tooltip.rs` to go through the shared `use_escape_key` hook, which corrects a
  real bug: today a root `Menu` or a `Tooltip` responds to Escape even when it is not
  the topmost layer (e.g. nested under an open `Dialog`).
- Route `selectable.rs`'s inlined touch-drift check through `gesture::moved_past_threshold`
  (built for this, only partially adopted) and `selection::selected_text` through
  `selectable::selected_texts` (identical bodies today).
- Re-evaluate the `menubar` exception to the shared `Positioner` requirement.
  **Closed (2026-09-08):** `MenubarContent` now composes `Positioner`, gated on a
  live scroll-follow verification (see `design.md`'s D11 and `specs/
  adico-primitives/spec.md`'s 2026-09-08 Correction) that was run and passed —
  scrolling 100px moved both the trigger and the positioned content by exactly
  100px, preserving the same offset before and after.
- Document (not merge) the three independent unique-id counters (`lib.rs`, `toast.rs`,
  `portal.rs`) — they are not duplicate implementations of one behavior, so no code
  changes there beyond an explanatory comment at each site.
- Correct stale doc/spec text: `positioner.rs`'s module doc still describes continuous
  repositioning as "deferred" against an archived task path that no longer resolves;
  `lib.rs`/`positioner.rs` retract an old defect claim on the grounds that a cited
  provenance record "does not exist in this repository's history" — it does (added,
  modified five times, and later deleted, all in git history) — so the retraction's
  *reasoning* is corrected while its *conclusion* (live-verified behavior) is kept.

## Capabilities

### New Capabilities

(none — no new user-facing capability is introduced; this reshapes existing
primitive internals and their documented contracts)

### Modified Capabilities

- `adico-primitives`: adds a requirement that hover-intent delay behavior is a
  shared, public primitive rather than duplicated per-consumer; modifies the
  segmented-field-input requirement to reflect that `segment` (and the target-aware
  `time` resolution helper) are public modules; corrects the `menubar`/`Positioner`
  exception's stated justification to match the capability's actual current state
  (dated Correction, 2026-09-07), without asserting the exception is closed.
- `adico-primitives-authorship`: clarifies that a primitive absorbing another's
  behavior may become a facade that supplies its own per-component defaults (not
  only a bare `pub use` re-export), provided the merged module's public API is still
  the union of both primitives' prior behavior.

## Impact

- **Affected code**: `packages/adico-primitives/src/` — new `hover_intent.rs`; edits
  to `lib.rs`, `hover_card.rs`, `preview_card.rs`, `menu.rs`, `navigation_menu.rs`,
  `tooltip.rs`, `toolbar.rs`, `command.rs`, `separator.rs`, `selectable.rs`,
  `selection.rs`, `accordion.rs`, `segment.rs`, `time.rs`, `gesture.rs`,
  `positioner.rs`, `menubar.rs`; doc-only edits to `toast.rs`, `portal.rs`,
  `pointer.rs`, `alert_dialog.rs`, `virtual_list.rs`.
- **Public API**: `segment` and `time` become newly `pub`; `HoverCard`/`HoverCardContent`
  gain new props (`delay_ms`, `close_delay_ms`, a content `role` override) with
  defaults chosen to preserve today's instant-open behavior; `MenubarContentProps`
  gains `side`/`align` props (defaults matching `MenuContentProps`'s own, preserving
  today's below-trigger placement); a new public `hover_intent` module is added; a
  new crate-root `use_optionally_controlled` function is added alongside
  `use_controlled` (see What Changes). These are additive — no existing public
  signature is removed or narrowed.
- **Behavior change (bug fix), scoped precisely:** `menu.rs:243` and
  `tooltip.rs:197` previously had *no* dismissal-layer check at all — routing
  them through `use_escape_key` adds one where none existed, which is a
  strict improvement with no narrower case to regress. The concrete
  observable win: a `Tooltip` or root `Menu` genuinely nested as a descendant
  of another open overlay (for example opened from inside an already-open
  `Dialog`'s content) no longer closes on an Escape meant for that outer
  overlay. This does **not** yet extend to the more common case of two
  independent, sibling-level overlays with no shared ancestor (e.g. a
  page-level `Tooltip` and a separately-triggered page-level `Dialog`) —
  `adico_primitives::layer`'s shared stack is only actually shared through
  Dioxus's ancestor-scoped context lookup, and two siblings with no
  stack-providing common ancestor each register onto their own private
  stack today, so each remains trivially "topmost" regardless of this fix
  (see `design.md`'s D12 for the empirical repro). Closing that gap is a
  separate, larger change to `layer.rs` itself, out of scope here.
- **Downstream tooling**: `cargo xtask primitive-compat sync/check` output changes
  (new public module, new hook coverage) and must be reviewed, not blindly accepted;
  `cargo xtask primitive-usage check`, `registry validate`, and `provenance check`
  must still pass since no registry item's classification changes.
- **Verification note**: the `menubar`→`Positioner` scroll-follow check was run and
  passed (see `specs/adico-primitives/spec.md`'s 2026-09-08 Correction for the exact
  before/after measurement). It ran in the same automation browser used throughout
  this change, with `document.hidden` confirmed `true` immediately before and after —
  `use_reposition_bridge`'s scroll-tracking path is a plain `document`-level `scroll`
  listener, not gated on the `IntersectionObserver`/`ResizeObserver` callbacks Chrome
  throttles under that condition, so the earlier assumption that a foregrounded tab
  was required for this specific measurement did not hold. The user reviewed the
  measurement and confirmed it as sufficient.
- **Explicitly out of scope** (tracked, not addressed here): registry-side
  duplication, including `registry/ui/time_picker.rs` inlining
  `time_picker::angle_to_value`'s body instead of calling it; the four parallel
  item-registry implementations (`collection`, `selection`, `tag_group`, and the
  `text_values` maps in `menu.rs`/`command.rs`); `DateElement`/`TimeElement`
  duplication; `use_select_root`/`use_combobox_root`; `progress`/`meter`; and
  `menubar`/`context_menu`/`navigation_menu` content-item rendering reuse of `menu`'s
  (an explicitly open question in `lib.rs:16-18`, not resolved by this change).
