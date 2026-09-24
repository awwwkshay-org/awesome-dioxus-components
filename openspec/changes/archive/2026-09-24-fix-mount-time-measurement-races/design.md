## Context

See `proposal.md` — Why. The mechanics that matter:

- `registry/ui/carousel.rs`'s `onmounted` (lines ~192-210) spawns an async
  measurement of `get_client_rect()` and `get_scroll_size()` into
  `ctx.viewport_size` / `ctx.content_size`. `max_offset()` is
  `(content - viewport).max(0.0)`, and `can_scroll_next/prev` derive from it.
- Its `onscroll` handler *does* re-measure — but only while scrolling. When the
  mount measurement leaves `max_offset() == 0`, nothing can scroll, so that
  correction path is unreachable. The two form a deadlock.
- `registry/ui/time_picker.rs`'s dial measures `get_scroll_size()` once in
  `onmounted`; its file comment already records choosing `get_scroll_size` over
  `get_client_rect` precisely because the popover's entrance transform
  corrupts the latter — the same timing hazard, one step short of the fix.
- `packages/adico-primitives/src/scroll_area.rs` already solves exactly this,
  in `scroll_area_viewport_onresize`: read `ResizeData::get_content_box_size()`,
  then re-read `get_scroll_size()` from the stored handle.

## Goals / Non-Goals

**Goals**
- The failure mode becomes self-correcting rather than permanent.
- Follow the mechanism this repository already established, so there is one
  measurement pattern rather than two.

**Non-Goals**
- No API change, no new prop, no visual change.
- Not a general audit of every `onmounted` measurement in the registry. Two
  components have a demonstrated, test-backed failure; the rest do not, and
  changing them speculatively would be consumer-visible churn.

## Decisions

### D1 — Native `onresize`, not a timer or a JS bridge

Both components gain an `onresize` handler beside their existing `onmounted`.
`ResizeData` is backed by a real `ResizeObserver`, so the correction fires
exactly when the box actually settles.

*Alternative rejected — re-measuring after a delay* (`spawn` + sleep, or a
second measurement on the next tick). It trades a deterministic signal for a
guess about how long layout takes, and the merge change's own finding is that
this timing shifts with nesting depth — a delay tuned today breaks when someone
adds a layout layer, which is precisely how this bug surfaced.

*Alternative rejected — `document::eval` polling.* Explicitly documented in
this repo as the unreliable channel in this Dioxus runtime
(`registry/ui/sidebar.rs`, `message_scroller.rs`).

*Alternative rejected — fixing it in the app.* The defect is in registry
source that every consumer copies; an app-level workaround would leave every
other consumer broken, and `adico-web-structure` forbids app-specific
escape hatches regardless.

### D2 — Carousel: re-measure both axes, guarding against zero

`onresize` sets `viewport_size` from the observed content box and re-reads
`get_scroll_size()` for `content_size`, mirroring `scroll_area`'s handler.
Zero-size readings are ignored, matching `onscroll`'s existing
`if viewport > 0.0` guard — an element that is temporarily display-none
reports a zero box, and adopting that would reintroduce the stuck state the
fix exists to remove.

### D3 — TimePicker: size only, preserving the existing rationale

The dial's `onresize` updates `face_size` only. The file's existing comment is
explicit that *position* must never be cached because `Positioner` places the
popup after mount, and that `get_scroll_size` is used over `get_client_rect`
because of the entrance transform. Both constraints are preserved: the new
handler reads the content box for size, and still stores no position.

### D4 — The installed copies go through the real CLI

`apps/web/src/components/ui/{carousel,time_picker}.rs` are refreshed with
`adico add --replace`, not hand-edited — `adico-web-structure` #5 requires the
app's installed source be produced by the CLI. The resulting `adico.lock` churn
(every item in the plan gets the current manifest's digest) is expected, not a
defect: `manifestDigest` is `sha256` of the whole manifest, cloned per item.

## Risks / Trade-offs

- **`ResizeObserver` fires on every box change, including during drag** → the
  handler only writes size signals, which are idempotent; it does not touch
  scroll offset or drag state.
- **Consumer-visible source change** → additive, and it cannot degrade a
  correct measurement: `onresize` reports the element's real box, which is what
  the mount path was already trying to read.
- **SSR** → `onresize` simply never fires without a live DOM, exactly like the
  existing `onmounted`/`onscroll` handlers; initial render still succeeds on
  the signal defaults.

## Migration Plan

No consumer action. A consumer who re-runs `adico add carousel time-picker`
picks the fix up; one who does not keeps today's behavior, since they own their
copy. Revertable as one commit plus a registry rebuild.

Archiving order, which is the point of this change:
`fix-mount-time-measurement-races` → `2026-09-24-merge-apps-into-web` →
`redesign-web-visual-foundation` → `docs-component-examples` →
`docs-guide-pages`.
