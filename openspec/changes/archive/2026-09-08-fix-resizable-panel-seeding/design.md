## Context

See `proposal.md` - Why. Confirmed directly against source before writing this:

- `registry/ui/resizable.rs:71-77` — `ResizableContext { direction, container_size: Signal<f64>, panels: Signal<Vec<PanelConstraints>>, drag: Signal<Option<DragState>> }`, provided once per `ResizablePanelGroup` via `use_context_provider` (`:138-143`).
- `registry/ui/resizable.rs:239-253` — the seeding effect:
  ```rust
  use_effect(move || {
      let mut panels = ctx.panels;
      panels.with_mut(|panels| {
          if panels.len() <= idx {
              panels.resize(idx + 1, PanelConstraints { size: default_size, min: min_size, max: max_size });
          }
      });
  });
  ```
  `Vec::resize(new_len, value)` clones `value` into every newly created slot
  (stdlib-documented behavior, not a Dioxus quirk). Combined with the
  `panels.len() <= idx` guard — which is permanently false once any later
  index has already grown the vec — whichever `ResizablePanel`'s effect
  resolves *first* determines every lower index's constraints forever.
- All eight touch points of `ctx.panels` are enumerated and line-referenced
  in the fix's task list; the render-size lookup (`:255-260`), both drag
  paths (`:203-213` pointer, `:352-371` drag-start, `:373-394` keyboard), and
  `resize_pair_from` (`:96-126`) all currently assume every index they access
  already holds a real `PanelConstraints`.
- `registry/ui/resizable.rs:286-294` — `ResizableHandle`'s hit area today is
  exactly its visual line: `axis_class` is `"w-px cursor-col-resize"`
  (Horizontal) or `"h-px w-full cursor-row-resize"` (Vertical), with no
  separate hit-target element.
- Verified upstream shadcn/ui's current `resizable.tsx`
  (`gh api repos/shadcn-ui/ui/contents/.../resizable.tsx`, 2026-09-08) rather
  than from memory, per this project's own established convention (the
  archived `2026-09-08-add-playground-resizable-panels` change's design.md
  did the same for the grip fix). Upstream's `ResizableHandle` base class:
  ```
  relative flex w-px items-center justify-center bg-border
  after:absolute after:inset-y-0 after:left-1/2 after:w-1 after:-translate-x-1/2
  focus-visible:ring-1 ... aria-[orientation=horizontal]:h-px aria-[orientation=horizontal]:w-full
  aria-[orientation=horizontal]:after:left-0 aria-[orientation=horizontal]:after:h-1
  aria-[orientation=horizontal]:after:w-full aria-[orientation=horizontal]:after:translate-x-0
  aria-[orientation=horizontal]:after:-translate-y-1/2
  ```
  The visible line stays 1px; a `::after` pseudo-element widens the actual
  pointer hit target to `w-1`/`h-1` (4px) centered on the line via
  `-translate-x-1/2`/`-translate-y-1/2`.
- `apps/playground/src/components/demo.rs:99-101,147-149` — today's bounds:
  preview `default 70.0, min 40.0, max 85.0`; controls
  `default 30.0, min 15.0, max 60.0`.
- `apps/playground/adico.lock:485-491` records `@adico/resizable`'s file
  checksum as `b3b1bc7e…`, but the file actually on disk at
  `apps/playground/src/components/ui/resizable.rs` hashes to `936a823a…` —
  pre-existing drift from the prior change's grip fix landing without a
  matching `adico add --replace`. `adico-cli`'s `add.rs:203-211` treats the
  on-disk checksum as an install precondition, so a plain `adico add
  resizable` would refuse to overwrite; `--replace` is the documented escape
  hatch for exactly this.

## Goals / Non-Goals

**Goals:**
- Seeding becomes order-independent: each `ResizablePanel` index ends up with
  its own `default_size`/`min_size`/`max_size` no matter which sibling's
  effect resolves first.
- No panel-count assumption beyond what already exists (still exactly two
  panels per group in every current call site) — the fix generalizes to any
  count without change.
- The resize handle is grabbable within a few pixels of its visible line, on
  both axes.
- Zero change to `ResizablePanelGroup`'s or `ResizableHandle`'s public prop
  surface — this is an internal-state and internal-class fix only.

**Non-Goals:**
- No change to `clamp_delta`'s mutual-clamp math, drag-overlay mechanism, or
  keyboard step size — those are unaffected by the seeding bug.
- No persistence of resized sizes across reload or navigation — unchanged
  from the already-specified behavior.
- No fix for `ResizablePanel`'s `overflow-auto` clipping anchored popovers
  (see proposal's Explicitly out of scope).
- No refresh of the stale `examples/basic-spa`/`examples/basic-ssr`/
  `tests/installation/m7-consumer` copies (see proposal).

## Decisions

**Represent unseeded panels as `None`, not a sentinel `PanelConstraints`.**
Changing `panels: Signal<Vec<PanelConstraints>>` to
`Signal<Vec<Option<PanelConstraints>>>` makes "not yet seeded" a distinct,
unrepresentable-as-valid-data state, so no accidental sentinel value (e.g.
`size: -1.0`) can leak into a real layout calculation if a read site forgets
to check it. The seeding effect becomes:
```rust
use_effect(move || {
    let mut panels = ctx.panels;
    let already_seeded = panels.peek().get(idx).is_some_and(Option::is_some);
    if !already_seeded {
        panels.with_mut(|panels| {
            if panels.len() <= idx {
                panels.resize(idx + 1, None);
            }
            panels[idx] = Some(PanelConstraints { size: default_size, min: min_size, max: max_size });
        });
    }
});
```
The length check still exists (a vec must be grown before indexing into it),
but it now only gates *growth*, never the *write* — every panel writes its
own slot unconditionally once grown. `resize(idx + 1, None)` fills any newly
created lower slots with `None`, never with another panel's real data, so a
later-mounting lower-index panel still seeds correctly on its own turn.

Every other `ctx.panels` read (render size, both drag paths, the mutual-clamp
helper) changes from `panels.get(i).copied()` to
`panels.get(i).copied().flatten()`, and a `None` on either side of a handle
is treated as "this pair isn't resizable yet" (drag/keyboard handlers
return early; `resize_pair_from` already takes `Option::get`-style access
patterns and is updated to bail the same way). In steady state (both panels
mounted) this is unreachable — every current call site renders exactly two
sibling panels that both mount before any user interaction is possible — so
it only matters as short-lived defensive bail-out, not user-visible
behavior.

**Verify `use_effect` fires for a `peek()`-only body before relying on it,
falling back to `read()` otherwise.** `peek()` is preferred because it
doesn't subscribe this effect to `ctx.panels`, which would otherwise
re-schedule it on every drag write; but if this Dioxus version (`=0.7.9`)
requires a *reactive read* inside `use_effect` to guarantee it runs at all on
mount, `peek()`-only would silently never seed. The implementation task
verifies this empirically (a manual test: does the seeded value ever appear
without any signal read?) before committing to `peek()`; if it doesn't fire
reliably, switch the `already_seeded` check to `panels.read()` — the
`already_seeded` guard already prevents this from re-entering on its own
write, so switching to `read()` costs one extra subscription, not
correctness.

**Alternative considered and rejected: keep `Vec<PanelConstraints>` and just
change `resize` to `resize_with` with per-slot defaults.** This alone doesn't
fix the bug: `resize_with(idx + 1, || default_placeholder)` still leaves the
`panels.len() <= idx` guard in place, so a lower-index panel whose effect
resolves *second* still never overwrites whatever placeholder occupies its
slot. The guard, not just `resize`'s fill behavior, is the other half of the
defect — fixing only one half still leaves a page-dependent bug, just with a
different (and equally wrong) clobbered value. This confirms the `Option`
approach's write must live outside the length check, not just change what
fills new slots.

**Expand the handle's hit area with a `::after` pseudo-element, following
upstream's exact mechanism, not this repo's own per-direction match-arm
pattern used elsewhere in the file.** `axis_class`/`grip_class` already use a
Rust `match direction { .. }` to pick Tailwind strings (no CSS attribute
selectors), which stays consistent for those. For the hit area specifically,
mirroring upstream's `after:*` utilities (translated into this file's
existing per-direction `match`, not upstream's `aria-[orientation=...]`
selector syntax, to stay consistent with the rest of this file) is simplest:
add an `after_class` alongside `axis_class`, keyed the same way:
- Horizontal (`w-px` vertical line): `after:absolute after:inset-y-0 after:left-1/2 after:w-1 after:-translate-x-1/2`
- Vertical (`h-px` horizontal line): `after:absolute after:inset-x-0 after:top-1/2 after:h-1 after:-translate-y-1/2`

(The vertical-case values are upstream's horizontal-`aria-orientation` rules
transposed onto this file's own enum naming — confirmed equivalent: a full
cross-axis pseudo-element via `inset-x-0`/`inset-y-0` rather than upstream's
`left-0 w-full`/`top-0 h-full`, same rendered result, fewer utility classes.)
No new DOM element, no change to focus/keyboard handling, no change to the
grip chip.

**Bounds retune is a plain numeric edit at the call site, not a spec-owned
constant.** `default_size`/`min_size`/`max_size` are `ResizablePanel` props
supplied per call site (`registry/ui/resizable.rs:229-233`); `Demo` already
owns its own values (`demo.rs:99-101,147-149`), so this is a two-line edit
with no registry-source change. Chosen pairing keeps both extremes summing
to 100 (preview 60↔80, controls 40↔20), the same convention set by the
archived resizable-panels change for both existing splits.

**Reinstall the playground's copy via `adico add resizable --replace`
instead of hand-copying the file.** `apps/playground/adico.lock` records a
per-file checksum for `@adico/resizable`; a hand copy would update the file
but not the lock, adding a *second*, self-inflicted drift on top of the
pre-existing one this task also happens to repair. Going through the real
CLI path keeps the lock an accurate record of what actually produced the
installed file, matching this project's architecture rule that consumer
fixtures and apps only ever receive registry source through the real install
path.

## Risks / Trade-offs

- **[`peek()` might not fire the seeding effect at all in this Dioxus
  version]** → Verify empirically before finalizing; fall back to `read()`
  if needed (see Decisions). Caught by the Playwright drag assertion either
  way, since a fully-unseeded table makes every drag a no-op.
- **[Widening the hit area changes nothing visually but could shift where
  adjacent content's own pointer events are captured]** → The `::after`
  pseudo-element is `absolute`, positioned relative to the handle's own
  `relative` container (already present at `:291`), and does not add layout
  box size — it only participates in hit-testing, matching upstream's
  identical, already-shipped mechanism. No layout shift.
- **[`adico add resizable --replace` also pulls in no unrelated changes,
  since the registry source for this task is exactly the seeding + hit-area
  diff]** → Confirmed by diffing the reinstalled file against
  `registry/ui/resizable.rs` post-fix; if `adico.lock`'s stale checksum
  causes `--replace` to also want to touch unrelated files, that would
  indicate the lock is wrong in a way this task doesn't anticipate — stop
  and re-examine rather than force past an unexpected diff.
