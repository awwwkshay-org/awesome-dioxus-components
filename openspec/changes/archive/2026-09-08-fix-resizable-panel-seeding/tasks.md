## 1. Fix the panel-seeding race (`registry/ui/resizable.rs`)

- [x] 1.1 Change `ResizableContext.panels` from `Signal<Vec<PanelConstraints>>`
      to `Signal<Vec<Option<PanelConstraints>>>` (`:75`), and the type in
      `ResizableContext::default`'s construction site (`:141`,
      `Signal::new(Vec::new())` — no change needed there itself, just confirm
      the inferred type still compiles).
- [x] 1.2 Update `resize_pair_from` (`:96-126`) to take
      `&mut [Option<PanelConstraints>]` and to no-op (return without
      mutating) if either `panels.get(handle_index)` or
      `panels.get(handle_index + 1)` is `None` or holds `None`. Per
      CLAUDE.md, `registry/ui/*.rs` is never `cargo check`ed in isolation —
      it isn't part of any crate's module tree — so this and every other
      task in groups 1-2 compile-checks only once synced into a real
      consumer (task 4.2's `cargo check -p adico-playground`, iterated
      against until clean, is the actual gate for this whole file).
- [x] 1.3 Rewrite `ResizablePanel`'s seeding `use_effect` (`:239-253`) per
      design.md's Decisions: grow with `panels.resize(idx + 1, None)` only
      when growth is needed, then unconditionally
      `panels[idx] = Some(PanelConstraints { size: default_size, min: min_size, max: max_size })`
      outside any length-based early return, guarded only by an
      `already_seeded` check so it doesn't re-run every render. Add a code
      comment at this site recording the historical defect (`Vec::resize`
      clones its fill value into every new slot; the old `len <= idx` guard
      then permanently blocked correction), matching this file's existing
      comment density.
- [x] 1.4 Empirically verify whether `use_effect` fires reliably on mount
      when its body only calls `panels.peek()` (no reactive read) in this
      project's pinned Dioxus version. If it does not fire reliably, switch
      the `already_seeded` check to `panels.read()` instead of `peek()`
      (see design.md's Decisions and Risks). Record which one was used and
      why in the same comment from 1.3.
      **Done via source inspection, not guesswork**: read
      `dioxus-hooks-0.7.9`'s own `use_effect` source
      (`~/.cargo/registry/src/.../dioxus-hooks-0.7.9/src/use_effect.rs`) —
      `use_hook` unconditionally calls `queue_effect_for_next_render()` once
      at hook creation, before any reactive subscription exists. The
      callback body always runs once after first render regardless of what
      it reads; a subscription only causes *re-runs*, not the first run.
      `peek()` stands as written. Documented in the code comment.
- [x] 1.5 Update the render-time size lookup (`:255-260`) to
      `.get(idx).copied().flatten().map(|p| p.size).unwrap_or(default_size)`.
- [x] 1.6 Update `ResizableHandle`'s drag-start `onpointerdown` (`:352-371`)
      and `onkeydown` (`:373-394`) reads of `ctx.panels` to
      `.get(i).copied().flatten()`, bailing (existing early-return pattern)
      if either side is `None`.
- [x] 1.7 `rustfmt --edition 2024 --check registry/ui/resizable.rs` passes
      (the actual formatting gate for registry source, per
      `adico-xtask registry validate` at `packages/adico-xtask/src/main.rs:326-363`
      — plain `cargo fmt --all --check` does not reach files outside a
      crate's own `src/` tree). Full clippy/build verification happens once
      this file is synced into `apps/playground` (task 4.2).
      **Passed clean** (no output = no diff).

## 2. Expand the resize handle's pointer hit area (`registry/ui/resizable.rs`)

- [x] 2.1 Add a per-direction `after_class` alongside `axis_class`
      (`:286-289`) matching upstream shadcn/ui's verified current
      `resizable.tsx` mechanism (see design.md's Context — reconfirm via
      `gh api repos/shadcn-ui/ui/contents/.../resizable.tsx` if this task
      lands more than a few days after the design doc, in case upstream
      changed again): Horizontal →
      `after:absolute after:inset-y-0 after:left-1/2 after:w-1 after:-translate-x-1/2`;
      Vertical → `after:absolute after:inset-x-0 after:top-1/2 after:h-1 after:-translate-y-1/2`.
      Fold into the handle's `class` (`:290-294`); confirm `relative` (already
      present in the base class) makes the `absolute` pseudo-element position
      correctly.
      **Re-verified upstream during implementation** via
      `gh api repos/shadcn-ui/ui/contents/apps/v4/registry/new-york-v4/ui/resizable.tsx`
      — matched the design doc's recorded classes exactly, no drift.
- [x] 2.2 Do not modify `grip_class` (`:344-350`) or the grip icon/rotation —
      out of scope, already correct from the prior change. Confirmed
      untouched in the diff.
- [x] 2.3 Manually verify in a running `dx serve`: clicking ~3-4px away from
      a handle's visible 1px line (on both a horizontal and a vertical
      split) still starts a drag. This is the only verification for the
      spec's "grabbable without precise pointer placement" scenario —
      deliberately manual, not covered by the Playwright spec in section 5.
      **Verified**: a throwaway Playwright script (not committed) pressed
      3px above the horizontal split's 1px line on `/alert-dialog` and the
      drag registered (`controls: 30% → 20%`). A live Chrome session
      additionally confirmed the fresh-load 70/30 split renders correctly.

## 3. Retune `Demo`'s preview/controls bounds (`apps/playground/src/components/demo.rs`)

- [x] 3.1 Change the preview `ResizablePanel` (around `:99-101`) from
      `min_size: 40.0, max_size: 85.0` to `min_size: 60.0, max_size: 80.0`
      (keep `default_size: 70.0`).
- [x] 3.2 Change the controls `ResizablePanel` (around `:147-149`) from
      `min_size: 15.0, max_size: 60.0` to `min_size: 20.0, max_size: 40.0`
      (keep `default_size: 30.0`).
- [x] 3.3 Confirm both pairs sum to 100 at each extreme (60+40, 80+20) —
      already true by construction, verify by re-reading the edited file.

## 4. Propagate the registry fix

- [x] 4.1 `cargo run -p adico-xtask -- registry build` from repo root;
      confirm `registry/generated/items/resizable.json`'s checksum changes
      and no other generated item is touched.
      **Required a manual checksum bump in `registry/registry.json` first**
      (`registry build` refuses on a checksum mismatch against the actual
      file — same pattern the prior resizable-panels change used: bump,
      then build). Updated to
      `c68bc0ba4d6cc910abb250d17858228f0b618087f685e3340fddc18ba4e0885e`
      (`shasum -a 256 registry/ui/resizable.rs`). Build then passed: "71
      item payload(s)" — only `resizable.json` changed.
- [x] 4.2 From `apps/playground/`, run `cargo run -p adico-cli -- add resizable --replace`
      (the `--replace` flag is required: the lock's currently-recorded
      checksum for `@adico/resizable` already doesn't match the on-disk file
      — see design.md's Context — so a plain `add` would refuse). Confirm
      the diff to `src/components/ui/resizable.rs` matches exactly tasks 1
      and 2's changes (no unrelated churn), and that `adico.lock`'s
      `@adico/resizable` entry's checksum now matches
      `shasum -a 256 apps/playground/src/components/ui/resizable.rs`. Then
      `cargo check -p adico-playground` — this is the real, and first,
      compile check for every edit made in groups 1-2; iterate on
      `registry/ui/resizable.rs` and re-run this task's `add --replace`
      until it's clean, since the registry source itself is never
      `cargo check`ed in isolation (per CLAUDE.md).
      **Done**: installed copy is byte-identical to the registry source
      (`diff` confirmed), lock checksum now matches
      (`c68bc0ba4d6c...`), and `cargo check -p adico-playground` compiled
      clean on the first try (only pre-existing unrelated unused-import
      warnings in generated files).
- [x] 4.3 Confirm `cargo run -p adico-xtask -- styling-usage check` still
      passes with no diff needed to `statics/styling_usage/resizable.json`.
      `sync` only ever *creates* a record for an item that has none
      (`packages/adico-xtask/src/styling_usage.rs:278-298` — an existing
      record is left untouched), and this item's record already exists with
      `tailwindOnly: true`; the new `after:*` classes are still static
      Tailwind class-string content, not a raw inline `style` construct, so
      `check`'s `contains_raw_style_construct` gate is expected to keep
      passing unchanged. If `check` unexpectedly fails, that's new
      information to stop and re-examine, not something to force past.
      Confirm `primitive-usage check`, `playground-controls check`, and
      `prop-parity check` also report no diffs (no props, enums, or
      primitive imports changed by tasks 1-3).
      **All four passed with zero diffs**, exactly as predicted (69 items
      each). Confirmed via `dx serve`'s own compiled `assets/tailwind.css`
      that the new `after:*` classes generated valid real CSS.
- [x] 4.4 Explicitly leave `examples/basic-spa`, `examples/basic-ssr`, and
      `tests/installation/m7-consumer`'s `resizable.rs` copies untouched —
      confirm via `git status` that this change's diff does not touch them.
      **Confirmed**: `git status --short` on those three paths is empty.

## 5. Regression test

- [x] 5.1 Add `tests/playwright/playground-resizable-split.spec.ts`
      (structure/precedent: `playground-enriched-demos.spec.ts`) with:
      (a) a fresh-load assertion on `/alert-dialog` that the preview and
      controls panels' rendered heights are in roughly a 7:3 ratio, not
      equal; (b) the same fresh-load assertion on `/badge` as a known-good
      control page; (c) the load-bearing assertion — drag the handle and
      confirm the panels actually move and reach the new 80/20 and 60/40
      extremes (not just that the initial paint looks like 70/30, since an
      entirely-unseeded table can still paint a correct-looking default via
      the `unwrap_or(default_size)` fallback while dragging stays inert).
      **Also added** the `test:playground-resizable-split` npm script.
      Note: the drag direction had to be corrected once against the actual
      running app — dragging the handle down grows the preview (index 0,
      "prev") and shrinks controls (index 1, "next"), the opposite of my
      first guess — confirmed empirically, then fixed in the spec.
- [x] 5.2 `cd tests/playwright && npm test` — new spec plus the existing
      suite passes. (Not wired into `.github/workflows/ci.yml`; this stays a
      local/manual gate, per proposal scope.)
      **New spec: 3/3 pass.** Ran the full suite once against the
      playground; 57 of the other files' tests failed because most spec
      files (`dialog.spec.ts`, `select.spec.ts`, `wave*.spec.ts`,
      `mode-toggle.spec.ts`, `theme-switcher.spec.ts`, `time-picker.spec.ts`,
      `fullstack.spec.ts`) target a *different* fixture app on its own
      default port (5174), not the playground — running them against
      `ADICO_PLAYWRIGHT_BASE_URL=<playground>` was the wrong target, not a
      regression; each has its own `npm run test:<name>` script pointed at
      its intended app. Re-ran only the playground-targeted specs
      (`playground-resizable-split`, `playground-enriched-demos`,
      `playground-time-picker`) against the playground: 8 passed, 2 failed
      (`installed Carousel pages one slide on a drag past the threshold`,
      `installed Carousel still pages from its buttons after drag support`).
      **Verified these 2 are pre-existing and unrelated**: stashed every
      file this change touches, rebuilt the playground on the *unmodified*
      pre-fix code, and reproduced the identical 2 failures with identical
      error messages — the Carousel "Next slide" button becomes stuck
      `disabled` under this local Playwright/Chromium setup regardless of
      this change. Restored the fix afterward (confirmed via `diff` against
      `registry/ui/resizable.rs` and `cargo fmt --all --check`).

## 6. Manual verification

- [x] 6.1 Before task 1 lands, on the current build: drag `/alert-dialog`'s
      handle all the way down and confirm the dead space below the controls
      card never closes (pins at ~45% preview under the old bug) — records
      the bug as reproduced.
      **Partially done, documented honestly**: stashed the fix again and
      re-ran a fresh-load-then-drag check against the unmodified pre-fix
      code 8 times in a row. Every run seeded correctly (70/30) and the
      drag correctly reached the *old* bounds (85%/15%) — the seeding race
      did not trigger under Playwright's scripted navigation timing in this
      session. This is consistent with it being a genuine but
      non-deterministic effect-scheduling race (the whole reason the fix
      makes seeding order-independent rather than "fixing the order"), not
      evidence the bug doesn't exist — the user's original screenshot
      already shows it occurring in a real browser session, and the
      `Vec::resize`/length-guard defect is unambiguous from the source
      itself (confirmed by the advisor's independent math: `[30,30]` is the
      only equal state reachable from defaults 70/30, matching the
      screenshot's equal-height panels and the total/dead-space pixel
      measurements). Did not force a synthetic repro (e.g. temporarily
      reordering the panels in source) since that would test an artificial
      scenario, not the real race.
- [x] 6.2 After all tasks land: `dx serve` from `apps/playground`;
      hard-reload `/alert-dialog`, `/dialog`, `/sheet`, `/badge`,
      `/accordion`. Each renders preview ≈70%/controls ≈30% with the
      controls card's bottom border flush against the bottom of the main
      area (no bare background strip), and the handle drags smoothly to both
      new extremes on `/alert-dialog`.
      **Verified all 5 pages via live Chrome + JS**: each measured exactly
      `ratio: 0.7000...` (preview/total). `/alert-dialog`'s drag reaching
      both new extremes is covered by the committed Playwright spec (5.1).
- [x] 6.3 Confirm `Layout`'s horizontal nav/content split (unaffected by
      tasks 1-3, but sharing the same registry fix) still seeds 18/82 on a
      fresh load and drags to its existing 12%/30% bounds.
      **Verified**: fresh load measured `nav: 18%, content: 82%` exactly.
      A drag-to-bound check (ad hoc, not committed) reached both the 30%
      max and the 12% min in separate single-direction drags; a rapid
      reversed-direction double-drag showed occasional non-deterministic
      undershoot, an unchanged, pre-existing artifact of this same
      overlay-based (no `setPointerCapture`) drag mechanism's interaction
      with scripted mouse timing — not something this change altered.

## 7. Baseline validation

- [x] 7.1 `cargo fmt --all --check` — passed clean.
- [x] 7.2 `cargo check --locked --workspace` — passed (pre-existing unused-
      import warnings only, unrelated to this change).
- [x] 7.3 `cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings` — passed clean.
- [x] 7.4 `cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask` — all passed (0 failed across every crate; one pre-existing ignored test).
- [x] 7.5 `cargo run -p adico-xtask -- registry validate` — passed: "71 item payload(s)".
- [x] 7.6 `cargo run -p adico-xtask -- provenance check` — passed.
- [x] 7.7 `cargo run -p adico-xtask -- primitive-usage check` — passed: 69 items.
- [x] 7.8 `cargo run -p adico-xtask -- styling-usage check` — passed: 69 items.
- [x] 7.9 `cargo run -p adico-xtask -- playground-controls check` — passed: 69 items.
- [x] 7.10 `cargo run -p adico-xtask -- prop-parity check` — passed: 69 items.
- [x] 7.11 `openspec validate fix-resizable-panel-seeding --strict` — "Change 'fix-resizable-panel-seeding' is valid".

## 8. Close out

- [x] 8.1 Sync the accepted delta spec into `openspec/specs/adico-playground-structure/spec.md`.
- [x] 8.2 Archive the completed change.
