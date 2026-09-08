## Why

On some playground pages (e.g. `/alert-dialog`), the live-preview canvas and
the "Component controls" card inside `Demo` render at equal heights instead
of the coded 70/30 split, together filling only ~60% of the main area with
bare background below and controls content clipped — and the resize handle
between them cannot recover the dead space no matter how far it's dragged.
Root cause, confirmed by reading `registry/ui/resizable.rs`: `ResizablePanel`'s
seeding effect (`if panels.len() <= idx { panels.resize(idx + 1,
PanelConstraints { .. }) }`) both clones whichever panel seeds *first* into
every lower index (`Vec::resize`'s documented fill behavior) and then
permanently blocks any other panel from correcting it (the `len <= idx`
guard is false forever once the vec is long enough). Two sibling
`ResizablePanel`s race this effect on mount; which one wins is page-dependent
(an effect-scheduling race, not page structure — `/alert-dialog`, `/dialog`,
and `/sheet` are built identically), so some pages render a clobbered
`[{30,15,60},{30,15,60}]` table instead of the intended
`[{70,40,85},{30,15,60}]`. Because `clamp_delta` holds a dragged pair's sum
constant, a clobbered pair's dead space is permanent, not merely
default-wrong. Separately, the handle's pointer hit area is a literal 1
CSS-pixel line, making it hard to grab even when sizes are correct.

## What Changes

- Fix the seeding race in `registry/ui/resizable.rs`: `ResizableContext`'s
  shared table becomes `Vec<Option<PanelConstraints>>`; each `ResizablePanel`
  seeds only its own index with its own `default_size`/`min_size`/`max_size`,
  unconditionally, outside any length-based early return, so the outcome no
  longer depends on which sibling's effect runs first. Every other read site
  (render, drag-start, keyboard step, the mutual-clamp helper) is updated to
  treat an unseeded neighbour as a no-op rather than panicking or reading
  garbage.
- Retune `Demo`'s preview/controls bounds
  (`apps/playground/src/components/demo.rs`): controls panel min/max
  20%/40% (was 15%/60%), preview panel min/max 60%/80% (was 40%/85%) — both
  pairs still sum to 100 at each extreme. Defaults stay 70/30.
- Expand `ResizableHandle`'s pointer hit target in `registry/ui/resizable.rs`
  to match upstream shadcn/ui's `resizable.tsx` (verified against current
  upstream source, not memory): the 1px visual line gains a wider invisible
  hit area via a pseudo-element, so the handle is grabbable without
  pixel-hunting. The grip chip/icon/rotation already fixed in the archived
  `2026-09-08-add-playground-resizable-panels` change is untouched.
- Propagate: regenerate `registry/generated/items/resizable.json` via
  `registry build`; reinstall `resizable` into the playground through the
  real CLI path (`adico add resizable --replace`) so both the installed
  source and `apps/playground/adico.lock`'s checksum stay consistent, rather
  than hand-copying (this also repairs pre-existing file-vs-lock drift from
  an earlier hand-applied grip fix). Regenerate
  `statics/styling_usage/resizable.json` for the new handle classes.
- Add a Playwright regression spec asserting the fix by *dragging* the
  handle to its new bounds, not merely by reading the initial paint (an
  unseeded table can still paint a correct-looking default via a fallback
  while dragging stays dead).

**Explicitly out of scope:** `ResizablePanel`'s unconditional `overflow-auto`
base class clips anchored popovers taller than the preview canvas, defeating
`Demo`'s deliberately `overflow-visible` canvas. This is a real, separate
defect with a different symptom (popover clipping vs. panel-size collapse) —
noted here as a follow-up, not fixed in this change. `examples/basic-spa`,
`examples/basic-ssr`, and `tests/installation/m7-consumer`'s installed
`resizable` copies are already several KB behind the registry (predating the
prior change's grip fixes) and are left stale — no CI gate enforces their
freshness today, and refreshing all three is a materially larger, unrelated
diff.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `adico-playground-structure`: the existing "The preview/controls split and
  the nav/content split are user-resizable" requirement is strengthened to
  make explicit two guarantees this fix newly provides that the requirement's
  text left implicit: (1) each panel's seeded size/min/max come from that
  panel's own props regardless of sibling mount/effect order, and (2) a
  resize handle's pointer hit target is not limited to its 1px visual line.
  The requirement's bound values are also updated to the concrete numbers now
  in use (controls 20–40%, preview 60–80%).

## Impact

- `registry/ui/resizable.rs` (seeding fix, handle hit-area) and its generated
  metadata (`registry/generated/items/resizable.json`,
  `statics/styling_usage/resizable.json`).
- `apps/playground/src/components/demo.rs` (bounds), `apps/playground/src/components/ui/resizable.rs`
  and `apps/playground/adico.lock` (reinstalled via CLI, not hand-copied).
- New `tests/playwright/playground-resizable-split.spec.ts`.
- No change to any of the 70 playground pages, `registry/ui/sidebar.rs`,
  `adico-primitives`, the CLI's install-planning logic itself, or any other
  registry item.
- No database, WebAssembly-target, or CLI-installation-fixture validation
  surface applies — registry-component and playground-app UI/state work only.
