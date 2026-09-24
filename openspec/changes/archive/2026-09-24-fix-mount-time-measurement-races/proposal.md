## Why

`registry/ui/carousel.rs` and `registry/ui/time_picker.rs` measure their own
geometry exactly once, in an `onmounted` handler, and never correct that
measurement afterwards. When the element's final layout is not yet settled at
mount, the stale reading is permanent.

For `Carousel` the result is a component that is simply broken: the track
genuinely overflows (`scrollWidth: 1680` against `clientWidth: 336`, five
slides) while both paging buttons report `disabled: true`, and even an arrow
key — which bypasses the disabled attribute and calls `scroll_next()` directly
— does not advance it. `max_offset()` is computed from the stale sizes, so it
is `0`, so nothing can scroll; and because nothing can scroll, the `onscroll`
handler that would have corrected the measurement never fires. It is a
deadlock, not a transient.

For `TimePicker`'s clock dial the face size can be measured mid-entrance
animation, so pointer positions resolve against the wrong centre and the hand
does not track the pointer.

This is the same class of defect as the already-archived
`2026-09-08-fix-resizable-panel-seeding`: a registry component seeding itself
from a one-shot mount-time measurement. It was found by
`2026-09-24-merge-apps-into-web`, whose task 9.8 root-caused it and
deliberately left it open, because fixing it means changing `registry/ui/*.rs`
— outside that change's declared scope. That change cannot be archived while it
stands, and three further changes are stacked behind it.

## What Changes

- **`Carousel` re-measures on resize.** Its scroll container gains an
  `onresize` handler alongside the existing `onmounted`, updating viewport and
  content size whenever the element's box changes. Paging state derives from
  current geometry rather than from whatever was true at mount.
- **`TimePicker`'s clock dial re-measures on resize**, so a face measured
  during the popup's entrance animation is corrected once it settles.
- Both use the repository's established mechanism: native `onresize` /
  `ResizeData`, backed by a real `ResizeObserver` — the same one
  `packages/adico-primitives/src/scroll_area.rs` and `message_scroller.rs`
  already use, and explicitly not a `document::eval` JS bridge, which this
  Dioxus runtime documents as unreliable.

Non-goals: no visual change, no API change, no new prop. No other registry
component is touched, and no consumer has to do anything.

## Capabilities

### Modified Capabilities

- `adico-existing-components`: the Carousel paging requirement and the
  TimePicker clock-dial requirement each gain the rule that the component
  derives its behavior from current geometry and recovers from a measurement
  taken before layout settled, rather than depending on a single mount-time
  reading.

## Impact

**Modified**
- `registry/ui/carousel.rs` — `onresize` on the scroll container.
- `registry/ui/time_picker.rs` — `onresize` on the dial face.
- `registry/registry.json` and `packages/adico-cli/embedded/registry.json` —
  regenerated checksums/content (`cargo xtask registry build`).
- `apps/web/src/components/ui/{carousel,time_picker}.rs` and
  `apps/web/adico.lock` — the installed copies, refreshed through the real CLI.

**Risk: this is consumer-visible source.** These files are copied into every
consumer project. The change is additive (one extra handler per component) and
cannot make a *correct* measurement worse: `onresize` fires with the element's
real box, which is what the mount-time path was trying to obtain.

**Lockfile note.** Refreshing the installed copies rewrites `manifestDigest`
for every item in the install plan to the current manifest's digest. That is
correct and expected — the field is `sha256` of the whole registry manifest
(`adico-registry-core/src/lib.rs:700`), recording which manifest version an
item came from, not a per-item hash. An earlier change misread this as a CLI
defect; see `redesign-web-visual-foundation/FOLLOWUPS.md` item 2 for the
retraction.

**Verification surface:** the three tests this unblocks —
`playground-enriched-demos.spec.ts`'s two Carousel tests and
`playground-time-picker.spec.ts`'s dial-tracking test — plus the full
registry/provenance/staleness gates, since registry source changed.
