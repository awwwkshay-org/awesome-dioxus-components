## 1. Doc/spec-only corrections (D12)

- [x] 1.1 Rewrite `positioner.rs:16-29`'s module doc to describe the current, partial
      observer-bridge state (`MutationObserver` path live-verified;
      `IntersectionObserver`/`ResizeObserver`/`scroll` paths registered but not
      fire-tested) instead of "was deferred", and fix the archived-task path it
      cites. Verify: `cargo doc -p adico-primitives` builds with no broken intra-doc
      links; comment content matches the `adico-primitives` spec delta's 2026-09-07
      Correction verbatim in substance. **Done:** rewrote, fixed the archived path
      (`openspec/changes/archive/2026-09-05-build-adico-component-ecosystem/tasks.md`),
      switched the one new intra-doc link to a code span after `cargo doc` flagged it
      as unresolved (it's a private fn); doc build now carries the same 10
      warnings it had before this change, none newly introduced.
- [x] 1.2 Correct the evidence in `lib.rs:270-289` and `positioner.rs:19-27`: remove
      the false "provenance record does not exist in this repository's history"
      claim (it does — `1caf731` add, five modifications, `6851afc` delete per
      `git log --all --name-status -- provenance/records/adico-primitives-wave3-overlays.json`),
      while keeping the live-verification conclusion those same comments already
      support. Verify: `git log` command above still shows the file's history as
      cited in the corrected comment. **Done:** both corrected with a dated
      (2026-09-07) note; `positioner.rs`'s copy was folded into 1.1's rewrite.
- [x] 1.3 Reconcile `pointer.rs:12`, `gesture.rs:25`, and `alert_dialog.rs:24` (which
      still carry the un-retracted original defect claim) to match the corrected
      text from 1.2. Verify: `rg -n "does not exist in this repository" packages/adico-primitives/src` finds no remaining instance of the false claim. **Done:**
      added a dated note at each site; each file's own specific finding (not the
      blanket citation) is preserved unchanged, per design.md.
- [x] 1.4 Fix `virtual_list.rs:15`'s reference to `use_global_escape_listener` (a name
      that no longer exists) to name the current mechanism. Verify:
      `rg -n use_global_escape_listener packages/adico-primitives/src` returns no
      matches. **Done:** renamed to `use_escape_key` with a parenthetical noting the
      rename.
- [x] 1.5 Run `cargo fmt --all --check` and `cargo clippy --locked -p adico-primitives --all-targets -- -D warnings` — doc-only changes, so both must pass with zero diff beyond comments. **Done:** both pass clean.

## 2. Promote `segment` and `time` to public (D4/D5)

- [x] 2.1 Change `mod segment;` and `mod time;` to `pub mod segment;` /
      `pub mod time;` in `lib.rs:97,100`; change their internal items from
      `pub(crate)` to `pub` as needed for external reachability, with doc comments
      on each newly-public item. No logic changes. Verify:
      `cargo check --locked -p adico-primitives` passes, and a new doctest in
      `segment.rs` (or a `tests/` integration test) imports
      `adico_primitives::segment::{NumericSegment, MeridiemSegment, use_segment_field_provider}`
      from outside the crate's own `src/` and compiles. **Done:** all 6 `segment.rs`
      items and both `time.rs` `sleep` impls promoted; added
      `tests/test_segment.rs` (new, black-box, external-crate import) — passes.
- [x] 2.2 Verify `date_picker.rs`/`time_picker.rs`'s existing imports of `segment::`
      still compile unchanged, and every existing `crate::time::sleep` call site
      (`preview_card`, `typeahead`, `virtual_list`, `toast`, `gesture`, `menu`)
      still compiles unchanged. Verify: `cargo test --locked -p adico-primitives`
      passes with no new failures. **Done:** full suite green — 48 test binaries,
      zero failures (incl. the new `test_segment` binary), all doctests pass.

## 3. Mechanical merges (D3, D6-doc, D9, D10)

- [x] 3.1 (D3) Make `ToolbarSeparator` (`toolbar.rs:276`) render via
      `separator::Separator` internally, preserving its `horizontal: Option<bool>`
      default-from-context behavior and its "no `children`" signature. Verify: an
      existing or new test asserts identical HTML output before/after (same
      `role`/`aria_orientation`/`data-orientation` attributes) for both the explicit
      and inherited-orientation cases. **Done:** existing `test_toolbar.rs` already
      pinned this exact behavior — 5/5 pass unchanged.
- [x] 3.2 (D3) Make `MenuSeparator` (`menu.rs:767`) and `CommandSeparator`
      (`command.rs:337`) delegate to `separator::Separator` instead of hardcoding
      the same three-line markup. Verify: existing menu/command tests asserting
      `role="separator"` still pass unchanged. **Done:** both delegate now (gaining
      `data-orientation="horizontal"`, a superset addition, no attribute dropped);
      full menu/command suite green, no test pinned the prior omission.
- [x] 3.3 (D6) Add a one-to-two-line comment at `lib.rs:107`, `toast.rs:200`, and
      `portal.rs:48` explaining why each id/counter mechanism is separate and not
      merged. No behavior or ordering change. Verify: `cargo test --locked -p adico-primitives` unaffected (zero test diff expected). **Done:** comment added at
      all three sites; zero test diff.
- [x] 3.4 (D9) Replace `selectable.rs:309-329`'s inlined
      `dx*dx + dy*dy > 25.0` touch-drift check with a call to
      `gesture::moved_past_threshold`, preserving the `pointer_type() == "touch"`
      guard and the exact `25.0` tolerance. Verify: `gesture.rs`'s existing tests
      (`:172-184`) already cover this tolerance; add/confirm a
      `selectable`-side test exercises the same touch-drift boundary. **Done:**
      delegated; `25.0` tolerance and touch guard unchanged.
- [x] 3.5 (D10) Reimplement `selection::selected_text` (`:87`) as
      `selected_texts(..).join(", ")`, keeping `selectable::selected_texts`'s
      `Vec<String>`-returning signature untouched. Verify: existing tests for both
      functions pass unchanged; `cargo test --locked -p adico-primitives` green.
      **Done:** extracted a new `selection::selected_texts` (`Vec<String>`) that
      both `selection::selected_text` and `selectable::SelectableContext::selected_texts`
      now delegate to — the one shared body, two unchanged public signatures.

## 4. `use_controlled` consolidation (D7 — corrected during implementation)

**Correction (implementation time):** the plan as proposed assumed all four
sites were hand-rolled copies of `lib.rs::use_controlled` and could route
through it directly. On inspection, three of the four (`Accordion`,
`use_single_selectable_value`, `MenuRadioGroup`) use a **different, deliberate**
pattern: their `value` prop is `Option<ReadSignal<Option<T>>>` (optionality at
the *outer* level, not just inside the signal), specifically because
`use_controlled`'s single-level `Option` can't distinguish "controlled but
currently nothing selected" from "not controlled at all" — a controlled prop
reading `None` in `use_controlled` always falls back to `default`, which would
be a real behavior regression for a controlled-and-cleared `Select`/`Accordion`/
`MenuRadioGroup`. This is not a new observation: `AccordionProps::value`'s and
`MenuRadioGroupProps::value`'s own doc comments already cross-referenced each
other as sharing "an identical convention" before this task started — the
duplication was real, just not duplication *of* `use_controlled`.

Only `AccordionMulti` (`values: ReadSignal<Option<Vec<String>>>`, no
outer-Option needed since `Vec` already has a native "nothing selected"
representation, `vec![]`) is a genuine, safe `use_controlled` duplicate.

Revised scope: add one new shared primitive (`use_optionally_controlled`,
alongside `use_controlled` in `lib.rs`) for the three-way tri-state pattern,
migrate `AccordionMulti` onto the existing `use_controlled` directly, and
migrate the other three onto the new primitive — preserving each site's exact
behavior, including the controlled-and-cleared case, with a new regression
test proving it.

- [x] 4.1 Add `pub fn use_optionally_controlled<T>` to `lib.rs`, next to
      `use_controlled`, implementing the tri-state
      `Option<ReadSignal<Option<T>>>` pattern with a `Callback<Option<T>>`
      setter (not `Callback<T>` — `Accordion` needs to set `None` directly).
      Verify: `cargo check -p adico-primitives` passes; doc comment records
      why this is separate from `use_controlled` (no behavior regression) per
      design.md's corrected D7.
- [x] 4.2 Route `accordion::Accordion` (`:189-207`) through
      `use_optionally_controlled`, and `accordion::AccordionMulti`
      (`:277-289`) through the existing `use_controlled` directly. Verify:
      existing `test_accordion.rs` suite passes unchanged, **plus three new
      tests** added first (test-before-rewrite): an uncontrolled trigger
      click opens its item; an uncontrolled already-open trigger click
      collapses it (the `None`-setter path); and — the regression test for
      this correction's whole premise — a controlled accordion whose value
      signal reads `None` renders nothing open and does **not** fall back to
      `default_value`. All 10 tests in the suite (7 existing + 3 new) pass.
- [x] 4.3 Route `selectable::use_single_selectable_value` (`:193-216`) through
      `use_optionally_controlled`, keeping its own `RcPartialEqValue`
      type-erasure/panic-on-mismatch layer and public signature unchanged.
      Verify: existing `select`/`combobox` tests (both compose this function)
      pass unchanged.
- [x] 4.4 Route `menu::MenuRadioGroup` (`:623-643`) through
      `use_optionally_controlled`, adapting `on_value_change: Callback<T>` to
      the shared primitive's `Callback<Option<T>>` (the adapter only ever
      receives `Some`, since this component's own setter never clears).
      Verify: existing `menu` tests covering `MenuRadioGroup` pass unchanged.
- [x] 4.5 Run `cargo test --locked -p adico-primitives` and confirm zero test
      failures and zero test-count regression across the four sites.

**Done:** all four sites migrated; 474 tests passing crate-wide (up from 465
at the start of this group, net +9: 1 `test_segment` + 8 new/counted
`test_accordion` assertions — see 4.2), zero failures, `cargo fmt --all
--check` and `cargo clippy -p adico-primitives --all-targets -- -D warnings`
both clean.

## 5. Escape consolidation and bug fix (D8)

**Correction (implementation time):** task 5.1/5.2's originally-planned "open
a Tooltip/Menu while a Dialog is open above it" regression test turned out to
require a *genuine ancestor/descendant* fixture, not independent siblings —
see design.md's D12 for the empirical finding (sibling components each
calling `use_layer` register onto separate, unshared `LayerStack`s; only a
true ancestor relationship shares one). The Menu version of that fixture was
additionally confounded by `Menu`'s own unrelated "close when nothing is
roving-focused" `use_effect`, which a synthetic test harness with no real DOM
focus always triggers, masking whether a close came from the Escape fix or
from that effect. Resolved by adding an isolated, consumer-independent test
(`test_lib.rs`) that exercises `use_escape_key`'s topmost-gating directly via
two true ancestor/descendant components and synthetic events (no DOM
dispatch, no effect flush) — the definitive proof the mechanism works,
per the model this repo already uses in `test_layer.rs` for the same class of
hook. The full-component tests were kept as positive/no-regression guards
(tooltip: both directions, confirmed unconfounded; menu: topmost-still-closes
only, since the non-topmost direction is the one the Menu effect confounds).

- [x] 5.1 Add a regression test (new or extended) that opens a `Tooltip` while a
      `Dialog` is open above it and asserts an Escape keypress dispatched at the
      tooltip's root does *not* close the tooltip when the dialog is topmost —
      capturing today's bug before fixing it. **Done, corrected structure:**
      `test_tooltip.rs`'s `TwoTooltipsInsideAnOpenDialog` (Dialog ancestor,
      two Tooltips as true descendants) plus
      `an_escape_at_a_non_topmost_tooltip_does_not_close_it` /
      `an_escape_at_the_topmost_tooltip_still_closes_it`, both green and
      unconfounded (verified independently by `test_lib.rs`, see above).
- [x] 5.2 Add the same shape of regression test for a root `Menu` opened underneath
      an open `Dialog`. **Done, with a documented limitation:** the
      non-topmost direction is confounded in this harness by `Menu`'s own
      pre-existing focus-sync effect (see the correction above and
      `test_menu.rs`'s doc comment on
      `an_escape_at_the_topmost_root_menu_still_closes_it`, the one direction
      kept as a full-component test); the mechanism itself (including for
      `Menu`'s exact code path) is proven directly by `test_lib.rs`.
- [x] 5.3 Replace `menu.rs:845` + `:880`'s hand-rolled Escape condition with
      `use_escape_key`, and wire `menu.rs:243` (root `Menu`'s own keydown) and
      `tooltip.rs:197` through `use_escape_key` as well (registering `use_layer`/
      `use_layer_member` in `tooltip.rs`, which does not call it today). Verify: the
      two tests from 5.1/5.2 now pass (bug fixed); confirm `use_escape_key`'s
      `prevent_default`/`stop_propagation` calls do not change `menu`'s existing
      submenu-Escape test outcomes — re-run the full `menu` test suite and diff
      failures, if any, against the pre-change baseline. **Done:** also found
      and removed `MenuSubmenuRoot`'s now-redundant standalone `use_layer(open)`
      call (a genuine second `provide_context(LayerOwner(..))` on one scope,
      left over from the hand-rolled version) since `use_escape_key` registers
      its own; full `menu` suite green, zero diff from baseline.
- [x] 5.4 `cargo test --locked -p adico-primitives -- menu tooltip` green, plus the
      full crate suite. **Done:** zero failures crate-wide; `cargo fmt --all
      --check` and `cargo clippy -p adico-primitives --all-targets -- -D
      warnings` both clean.

## 6. Shared `hover_intent` primitive (D1)

> **Correction (implementation time, 2026-09-07):** re-reading the three call
> sites fresh (`menu.rs:59-140`, `preview_card.rs:26-64`,
> `navigation_menu.rs:53-94`) before designing the primitive surfaced two
> changes from the original plan, both recorded in design.md's D1 section:
> (1) `MenuContext`/`PreviewCardCtx`/`NavigationMenuCtx` are each
> `#[derive(Clone, Copy)]` structs called from event handlers outside render —
> the primitive must hand back something `Copy`-storable as a plain field on
> those structs, not an `impl FnMut(..)` closure (the `use_escape_key` shape).
> (2) None of the three ever applied a zero-delay request synchronously
> (`spawn` is unconditional in all three; only the sleep inside it is
> conditional), so `hover_intent` preserves that, and the `skip_delay_when`
> predicate is dropped in favor of each caller computing its own per-request
> delay (unchanged in substance) and passing it as a plain argument. **6.4 from
> the original plan (migrating `preview_card.rs`'s own `request_open`) is
> folded into task 7.1** instead of standing alone here — `PreviewCardCtx`
> itself is deleted by task 7.2's facade rewrite, so migrating it first here
> and then deleting it in Group 7 would be throwaway work; `hover_card.rs`'s
> hover-open path is what actually needs to end up on `hover_intent`, and that
> only exists once Group 7 adds `delay_ms`/`close_delay_ms` to `HoverCard`.

- [x] 6.1 **Correction:** the original plan's premise for this task —
      "port `navigation_menu`'s existing... test coverage" — doesn't hold:
      searched `packages/adico-primitives/tests/` (no `test_navigation_menu.rs`
      exists at all) and `navigation_menu.rs`'s own inline `mod tests`
      (`:509-554`, itself a pre-existing, out-of-scope violation of this
      repo's test-placement convention — not touched here); neither test
      exercises `request_open`'s delay-selection logic in any way. No
      "switching between open items" coverage exists anywhere to port. Before
      writing `hover_intent.rs`, write a **new** black-box test (in the
      to-be-created `tests/test_hover_intent.rs`, this repo's normal
      placement) that pins the primitive's generation-counter supersede
      behavior deterministically: two `.request()` calls dispatched from a
      real event handler in immediate succession (both `delay_ms: 0`, so no
      real timer wait is involved — confirmed by a throwaway experiment that
      a zero-await `spawn()`ed task resolves within a single
      `render_immediate_to_vec()` call, the same mechanism this change's
      Group 5 tests already rely on for effect flushing) — the second
      request's value must win, proving the first was superseded by
      generation rather than by real elapsed time. Verify: this test fails
      against a stub primitive that applies every request unconditionally
      (proves it actually exercises supersession) before `hover_intent.rs` is
      implemented.
- [x] 6.2 Implement `packages/adico-primitives/src/hover_intent.rs`:
      `use_hover_intent<T: Copy + PartialEq + 'static>(setter: Callback<T>) ->
      HoverIntent<T>`, where `HoverIntent<T>` is `#[derive(Clone, Copy)]` (a
      `Signal<u64>` generation counter + the setter) with `fn request(&self,
      value: T, delay_ms: u64)` carrying the generation-bump/spawn/
      conditional-sleep/supersede-check body every site had duplicated. Add
      `pub mod hover_intent;` to `lib.rs`. Verify: the test from 6.1 now
      passes against the real primitive; add one more covering a non-`bool`
      `T` (e.g. `Option<usize>`) round-tripping correctly. **Not covered**,
      consistent with this crate's own established precedent
      (`test_toast.rs`'s documented "Auto-dismiss timing has no coverage
      here" gap, after a real attempt at a `#[tokio::test(start_paused =
      true)]` harness for the same class of spawn+timer behavior didn't work
      through this crate's `VirtualDom` test harness): a real, non-zero
      `delay_ms` actually waiting out its real duration before applying. That
      gap is carried forward, not silently dropped — see this change's manual
      playground checks (7.4) for a real-time substitute.
- [x] 6.3 Migrate `menu.rs`'s `MenuContext::request_hover_open` (`:119-140`) onto
      `hover_intent` (`T = bool`) — store a `HoverIntent<bool>` field on
      `MenuContext`, replacing the hand-rolled generation/spawn body with a call
      to `.request(open, delay_ms)` where `delay_ms` is still selected by the
      existing `hover_open_delay_ms`/`hover_close_delay_ms` branch. Verify:
      existing submenu hover-open/close tests pass unchanged.
- [x] 6.4 Migrate `navigation_menu.rs`'s `NavigationMenuCtx::request_open`
      (`:69-93`) onto `hover_intent` (`T = Option<usize>`) — the existing
      `switching_between_open_items` computation stays in `navigation_menu.rs`
      exactly as today (it reads `self.open_index`, ambient state the shared
      primitive has no access to), extracted into a private, pure
      `resolve_open_request_delay` function so it is directly unit-testable
      without a real timer, and its result passed as the resolved `delay_ms`
      argument to `.request(index, delay_ms)`. Verify: new unit tests (in
      `navigation_menu.rs`'s existing, pre-existing-and-out-of-scope-to-relocate
      inline `mod tests`, per Group 6's correction note) cover: switching
      between two already-open items resolves to zero delay regardless of a
      large configured `delay_ms`/`close_delay_ms`; opening when nothing else
      is open uses the configured open delay; closing uses the configured
      close delay.
- [x] 6.5 `cargo test --locked -p adico-primitives -- menu navigation_menu hover_intent` green, plus the full crate suite.

## 7. `preview_card` → `hover_card` facade merge (D2)

> **Post-hoc addendum (2026-09-07, from an advisor review after 7.1–7.5 were
> first marked done):** the review flagged that nothing actually exercised
> `PreviewCard`'s prop-forwarding into `HoverCard` — every existing test only
> asserted static markup, so a broken/deleted `delay_ms: props.delay_ms` line
> in the facade (both fields have defaults, so the compiler would accept it
> silently) would have shipped undetected, degrading `PreviewCard` to instant
> open/close. Verified this was NOT already broken (re-read `preview_card.rs`
> line-by-line, then deliberately broke the forwarding with a throwaway patch
> and confirmed the code as shipped does NOT match that broken shape). Added
> two new dispatch-based tests closing the gap: `test_hover_card.rs`'s
> `hovering_an_uncontrolled_trigger_opens_the_card_immediately` (proves a real
> `delay_ms: 0` still resolves within one dispatch-and-render cycle after the
> `hover_intent` migration) and `test_preview_card.rs`'s
> `preview_card_stays_closed_after_a_hover_because_its_delay_is_forwarded_and_not_zero`
> (proves the facade's non-zero default actually reaches `HoverCard`, by
> exploiting the fact that only a genuinely non-zero delay reaches
> `crate::time::sleep`, which panics outside a Tokio runtime in this crate's
> bare `#[test]`s — see that test's own doc comment for the full reasoning).
> Deliberately broke the forwarding again with the same throwaway patch to
> confirm the new test fails, then restored the correct source and reran the
> full suite green. Also fixed two stale doc references the same review
> flagged: `preview_card.rs`'s module doc named a `use_open_close_delay`
> function that never existed (the real name was
> `PreviewCardCtx::request_open`); and confirmed (not just assumed)
> `hover_card.rs`'s `force_mount: false` doc claim about
> `registry/ui/hover_card.rs` still holds by compiling
> `adico-example-basic-spa` (a real consumer, patched to this session's local
> `adico-primitives`) against the new `HoverCardContentProps`/`HoverCardProps`
> API — clean, confirming `registry validate`'s payload-manifest check isn't
> the only thing standing behind that claim.

- [x] 7.1 Add `delay_ms`/`close_delay_ms` props to `HoverCardProps` (default `0`,
      preserving today's instant open/close) and a `role: Option<&'static str>` prop
      to `HoverCardContentProps` (default `Some("tooltip")`, preserving today's
      behavior). Migrate `HoverCard`'s own open/close request path onto
      `hover_intent` (`T = bool`, the same shape as `menu.rs`'s from 6.3) — this is
      where `preview_card.rs`'s `PreviewCardCtx::request_open` (`:43-63`) logic
      actually lands (see Group 6's correction note): `hover_card` becomes the one
      place both delay-driven open/close behaviors converge. Also made
      `HoverCardCtx` `Copy` (previously `Clone`-only, with no `Copy` field to
      justify it before this task; every field, including the new `hover:
      HoverIntent<bool>`, already is) — required so `HoverCardTrigger`'s/
      `HoverCardContent`'s several `move` closures over `ctx` can each still
      compile without a per-closure `.clone()`. Verify: existing `HoverCard`
      tests (4/4 in `tests/test_hover_card.rs`) pass unchanged with the new props
      at their defaults; live-verified in a running `dx serve` (see Verification
      note below) that hover still opens/closes instantly and `role="tooltip"`
      is still present, since a spawn-deferred (even if same-microtask) apply
      path replaces what used to be a direct, non-deferred `set_open.call(..)` —
      a discovery specific to this task, not present in D1's original three
      sites, discussed with the advisor and deliberately kept (see design.md D2):
      the deferral is unobservable before the next paint, and losing it would
      make zero-delay requests structurally uncancelable, contradicting the
      spec's own supersede requirement.
- [x] 7.2 Rewrite `preview_card.rs` as a facade over the now-superset `hover_card`
      components, supplying `delay_ms: 600`, `close_delay_ms: 300`, content `side:
      Bottom`, and content `role: None` (no tooltip role) as its own defaults.
      Verify: `preview_card.rs`'s two existing tests
      (`closed_content_is_not_rendered_without_force_mount`,
      `open_root_marks_data_state_open`) moved to the new
      `tests/test_preview_card.rs` (this file's own pre-existing inline `mod
      tests` predated this repo's test-placement convention, same as
      `navigation_menu.rs`'s), asserting through the facade
      (`PreviewCard`/`PreviewCardContent`), not `HoverCard` directly — still
      pass, plus a new `content_does_not_carry_a_tooltip_role` test. 3/3 green.
- [x] 7.3 Re-record `hover_card.rs`'s existing `force_mount`-non-functional doc note
      on the merged implementation (carried forward unchanged, not fixed — see
      design.md Decisions D2): `hover_card.rs` already carries the note
      unmodified (not touched by this task); `preview_card.rs`'s own module doc
      now cross-references it explicitly, since the facade no longer has its
      own separate implementation to attach a duplicate note to. Verify: the
      note is present and accurate in both files; no new `force_mount` test is
      added (out of scope).
- [x] 7.4 Manual/playground check, actually run (not merely planned) via a local
      `dx serve` + Chrome automation session against `apps/playground`'s
      `/hover-card` route: hovering the `@dioxus` trigger opens `HoverCardContent`
      with no visible delay (screenshot before/after `hover`) and closes the same
      way on mouse-out; `document.querySelector('[role="tooltip"]')` finds the
      open content. **Correction:** the plan's premise that a "preview-card demo"
      exists in `apps/playground` to inspect doesn't hold — `preview_card` has no
      `registry/ui/` styled component or playground route at all (primitive-only,
      confirmed via `find`/route search), unlike `hover_card`, which has both.
      Its 600ms/300ms defaults are verified only by `tests/test_preview_card.rs`
      checking config forwarding through the facade (task 7.2), not by a live
      timed observation — the same class of gap `test_toast.rs` already
      documents for this crate's other real-timer behavior. `dx serve` stopped
      and the Chrome tab closed after the check.
- [x] 7.5 `cargo test --locked -p adico-primitives -- hover_card preview_card` green
      (`test_hover_card.rs`: 4/4; `test_preview_card.rs`: 3/3).

## 8. `menubar` → `Positioner` migration (D11 — gated)

> **2026-09-08 update:** re-verified `document.hidden` fresh in this
> session's own Chrome automation tab before doing anything else — still
> `true` (confirmed a third time, across two different sessions/days), so the
> live scroll-follow check remains structurally impossible for me to run
> myself, exactly as 8.4 originally concluded. Asked the user how to proceed
> (archive with D11 still gated, migrate-and-merge without verification, or
> do the migration now and have the user run the live check themselves); the
> user chose the third option. 8.1 below is the result.

- [x] 8.1 Migrated on a separate, isolated **git worktree** (not just a
      branch on the main working tree, since ~45 other tasks' work sat
      uncommitted there): `git worktree add .claude/worktrees/
      d11-menubar-positioner -b d11-menubar-positioner-migration <HEAD>`,
      isolating this from the rest of `deduplicate-primitives` exactly as
      planned. In `packages/adico-primitives/src/menubar.rs`:
      `MenubarMenuContext` gains a `trigger_id: Signal<String>` (generated via
      `use_unique_id()` in `MenubarMenu`, applied to `MenubarTrigger`'s own
      `button` element — neither existed before, since nothing needed an
      anchor id); `MenubarContentProps` gains `side`/`align` props (defaults
      `ContentSide::Bottom`/`ContentAlign::Start`, matching
      `MenuContentProps`'s identical defaults for the same top-level-trigger
      case); `MenubarContent`'s body now wraps its content in
      `positioner::Positioner` (`anchor_id: menu_ctx.trigger_id`) instead of a
      bare `div` with no placement logic (previously left entirely to the
      consumer's own CSS — confirmed `registry/ui/menubar.rs`'s styled facade
      sets no `absolute`/`fixed` utility classes either). Confirmed `Positioner`
      renders inline (a `position: fixed`-styled `div`, not a DOM portal —
      read its own `rsx!` body to verify) before relying on this: `MenubarMenu`'s
      existing wrapping `onkeydown` (covering both trigger and content) needed
      *no* changes at all, since keydown events dispatched inside the now-
      `Positioner`-wrapped content still bubble up through the normal DOM tree
      to that same handler — the migration is placement-only, not a keyboard-
      handling rewrite, matching the task's own stated scope. Verify: existing
      `menubar` behavior untouched — all 4 `tests/test_menubar.rs` tests pass
      unchanged; full `cargo test -p adico-primitives` suite green (all
      targets, 93 doctests including `menubar`'s own doc example); `cargo fmt
      --all --check` clean; `cargo clippy -p adico-primitives --all-targets --
      -D warnings` clean; `cargo check -p adico-example-basic-spa` (a real
      consumer of `registry/ui/menubar.rs`, patched to this worktree's local
      `adico-primitives`) compiles clean; `cargo xtask registry validate`
      passes (71 items) — none of these require the live scroll-follow check
      itself, so they confirm the migration didn't regress anything *other*
      than the one thing only a human can verify.
- [x] 8.2 **Run, via Claude in Chrome, at the user's explicit direction
      ("Use claude in chrome") — and it passed.** The user first showed a
      screenshot of the *unmigrated* menubar (their own separately-running
      `dx serve` on port 3000) with the `File` dropdown floating detached
      from its trigger, overlapping the `Edit`/`View` buttons — live,
      independent confirmation of the exact bug this migration fixes. Served
      the worktree's playground on port 3001, connected to the user's real
      Chrome browser (via `list_connected_browsers`/`select_browser`, after
      asking the user which browser per that tool's own instructions), and
      opened `/menubar` there. Re-confirmed `document.hidden: true` even on
      this newly-connected, user-selected browser — establishing this is a
      property of how the automation extension manages tabs, not of which
      underlying Chrome instance is connected. Visually confirmed the
      migrated version anchors the `File` dropdown correctly below its
      trigger (screenshot), then measured precisely via injected JS: `role`,
      `data-state="open"`, `data-side="bottom"`, `data-align="start"` all
      correct; `style="position: fixed; left: 929.59px; top: 386.65px"` —
      exactly `triggerBottom (382.65px) + offset (4px)`, confirming
      `ContentAlign::Start` and the `offset: 4.0` configured in 8.1. Made the
      page scrollable (temporary spacer div), opened the menu, scrolled
      100px, and re-measured: trigger `top` `335.05px → 235.05px`, content
      `top` `367.05px → 267.05px` — an exact 100px delta for both, the same
      32.0px offset preserved before and after. Escape-dismissal also
      re-verified working unchanged. Before treating this as a valid
      reproduction despite `document.hidden: true`, read
      `use_reposition_bridge`'s own injected JS (`positioner.rs`) and
      confirmed its scroll-tracking path is a plain
      `document.addEventListener('scroll', ..., true)` listener, not gated on
      the `IntersectionObserver`/`ResizeObserver` throttling the prior
      session's Correction assumed applied here too — that assumption was
      specific to those two observers and didn't hold for the scroll listener
      itself. Presented this nuance and the exact measurement to the user
      before proceeding (not silently treated as sufficient); the user
      reviewed it and confirmed "Good enough — merge and close the
      exception." Cleaned up: removed the temporary spacer div, closed the
      browser tab, stopped only the worktree's `dx serve` (left the user's
      own port-3000 instance untouched).
- [x] 8.3 8.2 passed — merged. Copied the migrated `menubar.rs` from
      `.claude/worktrees/d11-menubar-positioner` into this change's own
      working tree. Updated the `adico-primitives` spec's menubar Correction
      (2026-09-08) to record the exception as closed, with the measurement's
      exact before/after numbers as evidence, plus the corrected
      `document.hidden` assumption. Updated `proposal.md`'s Affected
      code/Public API/Verification bullets and `design.md`'s D11 section to
      match. Re-ran task 9's full validation sweep with `menubar.rs` merged
      into the rest of this change's work: `cargo fmt --all --check` clean;
      `cargo check -p adico-primitives` clean; `cargo clippy -p
      adico-primitives --all-targets -- -D warnings` clean; full `cargo test
      -p adico-cli -p adico-primitives -p adico-registry-core -p
      adico-test-utils -p adico-xtask` — 61/61 targets green, zero failures.
- [x] 8.4 Not applicable — 8.2 ran and passed, so the "8.2 never runs, D11
      ships as proposed-not-implemented" contingency this task described
      never triggered.

## 9. Full validation sweep

> **Re-run 2026-09-08 after 8.3 merged `menubar.rs`:** `cargo fmt --all
> --check` clean; `cargo check -p adico-primitives` clean; `cargo clippy -p
> adico-primitives --all-targets -- -D warnings` clean; full `cargo test -p
> adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p
> adico-xtask` — 61/61 targets green, zero failures; `cargo check -p
> adico-primitives --target wasm32-unknown-unknown --features web` clean;
> `cargo check -p adico-primitives --features native` clean;
> `primitive-compat sync`'s new diff (`MenubarContentProps` gains `side`/
> `align`, appearing twice — once per compat axis, matching the file's own
> two-axis structure) reviewed and traces exactly to 8.1's change, nothing
> unexpected; `registry validate` (71 items), `primitive-usage check` (69
> items), `provenance check` (1 record, 1 source unit) all still pass. The
> checks below are the original (pre-D11-merge) run; this note is the
> confirmation that merging D11 didn't invalidate any of them.

- [x] 9.1 `cargo fmt --all --check` — clean.
- [x] 9.2 `cargo check --locked --workspace` — exit 0 (only pre-existing,
      unrelated `apps/playground/src/generated/controls/mod.rs` glob-reexport
      warnings, not touched by this change).
- [x] 9.3 `cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings` — exit 0, no warnings.
- [x] 9.4 `cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask` — all green across every target (lib, every `tests/*.rs` integration binary, all 93 doctests).
- [x] 9.5 `cargo check -p adico-primitives --target wasm32-unknown-unknown --features web` — clean.
- [x] 9.6 `cargo check -p adico-primitives --features native` — clean.
- [x] 9.7 `cargo run -p adico-xtask -- primitive-compat sync`, then reviewed the diff
      by hand: new `hover_intent` module in the module list; `menu`/`tooltip` gain
      `use_escape_key`, `menu`/`navigation_menu`/`hover_card` gain
      `use_hover_intent`, `menu` loses `use_layer` (D8's redundant call removed);
      `accordion` gains `use_controlled`+`use_optionally_controlled` (D7);
      `HoverCardContentProps` gains `role`, `HoverCardProps` gains
      `delay_ms`/`close_delay_ms` (D2/7.1); `preview_card`'s `adico_hooks_used`
      goes to empty (now a pure facade, D2/7.2); `adico_only_extras` 25→26 (new
      `hover_intent` module). Every change traces to a specific task above — no
      unexplained diff. Accepted, not blindly.
- [x] 9.8 `cargo run -p adico-xtask -- registry validate` — "registry validation passed: 71 item payload(s) in @adico".
- [x] 9.9 `cargo run -p adico-xtask -- primitive-usage check` — "primitive-usage check passed: 69 item(s)".
- [x] 9.10 `cargo run -p adico-xtask -- provenance check` — "provenance check passed: 1 imported record(s), 1 source unit(s)".
- [x] 9.11 `openspec validate deduplicate-primitives --strict` — "Change 'deduplicate-primitives' is valid".
- [x] 9.12 Reported below (this session's final summary). Skipped, and why:
      D11's live-browser scroll-follow measurement (task 8.2) — re-verified
      `document.hidden` is still `true` in this session's own Chrome automation
      tab, confirming the constraint is structural to this environment, not
      merely unavailable "this window"; `menubar.rs` was not migrated as a
      result (see Group 8). Also not run, and out of scope per this change's
      own "Explicitly out of scope" list (unchanged from the original plan):
      Playwright/browser a11y suites, database/mobile/container checks (no
      such surface exists in this crate), and any registry-side consolidation.
