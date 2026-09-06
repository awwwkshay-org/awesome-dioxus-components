# Design: enrich-registry-components-and-playground-ux

## Context

See proposal.md — Why. Constraints that shape the approach:

- `apps/playground/src/components/ui/*.rs` is CLI-managed (checksummed in
  `adico.lock`) and is the only input the `playground-controls` generator
  reads. All component changes originate in `registry/ui/*.rs` or
  `packages/adico-primitives` and reach the playground via
  `adico add --all --replace` after a CLI rebuild (the registry is embedded
  in the binary via `include_bytes!`).
- `registry/registry.json` file checksums are hand-maintained;
  `registry build`/`validate` fail on mismatch. `moduleExports` is
  per-module, so new `pub` components in existing files need no manifest
  export changes — only checksums.
- `adico_primitives::menu`'s `MenuGroup`/`MenuGroupLabel`/`MenuSeparator`
  consume no context; `MenuCheckboxItem`/`MenuRadioGroup`/`MenuRadioItem`/
  `MenuSubmenuRoot`/`MenuSubmenuTrigger` require `MenuContext`, provided
  only by `Menu` (the dropdown root). Menubar and ContextMenu use
  independent context types.
- The carousel's original no-drag rationale cited a broken-on-web pointer
  pattern; that claim was retracted (`positioner.rs:17-28`) and the correct
  approach (per-element pointer events + transient overlay) is already
  proven in `registry/ui/resizable.rs:184-217`.
- Generated control panels call five control widgets with fixed signatures
  (`BoolControl { label, value: Signal<bool> }`, `TextControl`,
  `NumberControl { value: Signal<f64> }`, `OptionalBoolControl`,
  `SelectControl { value: Signal<T>, options: &'static [(&'static str, T)] }`).
- Icons come from `adico_primitives::icons` (a `dioxus_icons::lucide`
  re-export; `Eye`, `EyeOff`, `Check`, `Circle`, `ChevronRight`, and
  dashboard glyphs all exist). Playground depends on `adico-primitives`
  directly, so pages may use icons.

## Goals / Non-Goals

**Goals:**
- Registry additions are purely additive: existing consumer markup renders
  byte-identically unless a new prop/part is used.
- Playground demos become realistic compositions that exercise previously
  undemoed surface (`CardAction`, `DrawerDirection`, new menu parts).
- The playground shell dogfoods installed components without changing the
  generator or the generated-panel contract.

**Non-Goals:**
- No shared `MenuContext` across ContextMenu/Menubar (a primitives redesign;
  the delta spec explicitly scopes checkbox/radio/submenu to dropdown-menu).
- No extension of the `playground-controls` prop-type allowlist — pages
  hand-roll controls for `Option<u32>`/`ReadSignal<usize>` shapes, following
  the existing `sheet.rs`/`hover_card.rs` precedent. Extending the allowlist
  would add an `OptionalNumberControl` shape and churn every component with
  an `Option<u32>` prop for two demo bindings.
- No carousel autoplay, loop, momentum physics, or `setPointerCapture`
  interop (no repo precedent for web_sys downcasting; the overlay pattern
  achieves the same containment).

## Decisions

1. **Menu facades wrap primitives; shortcut is a bare span.**
   New `Dropdown/ContextMenu/Menubar` `Group`/`Label`/`Separator` parts wrap
   the context-free `adico_primitives::menu` parts with shadcn classes;
   `*Shortcut` is a styled `span` with no primitive (precedent:
   `CommandShortcut`). Checkbox/radio/submenu facades exist only in
   `dropdown_menu.rs`, wrapping the `MenuContext`-requiring primitives with
   stateless indicator styling driven by the primitive's own `data-state`
   attribute (`group-data-[state=checked]:opacity-100`), so the facade holds
   no duplicate state. Alternative considered: adding a shared menu context
   to Menubar/ContextMenu primitives — rejected as a high-risk redesign far
   beyond demo needs.

2. **Carousel drag = pointerdown on the track + transient full-screen
   overlay** (`fixed inset-0 z-[100]`) carrying `onpointermove`/`up`/
   `cancel`, exactly the `resizable.rs` pattern; never the global
   `pointer.rs` registry. During a drag: scroll imperatively with
   `ScrollBehavior::Instant` (fall back to `Smooth` if the variant doesn't
   exist in Dioxus 0.7.9), swap `snap-x snap-mandatory` → `snap-none` (snap
   fights instant positioning), and compute from the drag-start offset, not
   the live `scroll_offset` signal (which `onscroll` keeps writing during
   the drag). On release: |delta| > 20% of viewport pages ±1 via the
   existing `page()`; otherwise smooth-scroll back. The stale module doc
   citing the retracted rationale is rewritten.

3. **OTP mask is a root-level `ReadSignal<bool>` threaded through
   `OtpFieldCtx`** to `OtpFieldInput`, which renders
   `type: if masked { "password" } else { "text" }`. A facade-level
   attribute passthrough can't work — `type` is input-specific, not a
   GlobalAttribute, and the primitive sets it before `..attributes`.
   Purely presentational: the `Vec<Option<char>>` value model is untouched.

4. **Textarea counter wraps conditionally.** Only when `max_length` is
   `Some` does the component wrap in `div.relative.w-full` with an absolute
   bottom-right counter span (+`pb-6` on the textarea in that branch);
   `None` renders today's bare `<textarea>` byte-identically. The count is
   an internal signal updated in a wrapped `oninput` (forwarding the
   consumer handler), and a controlled `value` overrides the internal count
   so the counter can't drift. Trade-off: consumers who pass `max_length`
   AND style via `class` gain a wrapper div above the textarea — documented
   in the component doc comment.

5. **Pan starts only on the preview background.** The canvas cell gets
   `onpointerdown` to begin a pan; the inner component wrapper gets
   `onpointerdown: stop_propagation` so any drag beginning on the demoed
   component (slider thumbs, carousel tracks) never pans. No thresholds, no
   modifier keys, no target-vs-currentTarget discrimination (Dioxus doesn't
   expose it). Offset is a `Signal<(f64,f64)>` applied as
   `transform: translate(x px, y px)` on the already-centered wrapper, so
   `(0,0)` is the centered default and Center just resets the signal. Moves
   use the same transient-overlay pattern as decision 2. The canvas swaps
   `overflow-auto` → `overflow-hidden` (panning supersedes scrolling).

6. **Controls dogfooding freezes the five public signatures.** `BoolControl`
   rebuilds on installed `Switch` (its `Option<bool>`-in/`bool`-out API maps
   cleanly; `Checkbox` is tri-state), `TextControl`/`NumberControl` on
   `Input` (number via `r#type: "number"` + attribute-extension
   min/max/step), `SelectControl`/`OptionalBoolControl` on `NativeSelect`
   with index-valued options. The generator is untouched;
   `playground-controls check` plus `cargo check --workspace` prove the
   contract held. If `NativeSelect`'s `w-fit` wrapper can't stretch to the
   control column, that's a genuine registry defect fixed in
   `registry/ui/native_select.rs` as its own cited edit (spec-sanctioned
   path), not worked around.

7. **Ordering pipeline.** Registry/primitives edits land first, then one
   regeneration roll (checksums → `registry build`/`validate` → CLI rebuild
   → `adico add --all --replace` → `playground-controls sync` → statics
   syncs), then playground pages/shell. Running `playground-controls sync`
   before the reinstall would read stale installed sources — the task list
   encodes this order.

## Risks / Trade-offs

- [Pan pans when dragging a portaled popup rendered outside the wrapper] →
  acceptable; popups are transient and the component wrapper still blocks
  the common case. Documented in demo.rs.
- [Textarea wrapper changes DOM for consumers already passing `max_length`]
  → conditional wrapper keeps the no-max DOM identical; doc comment calls
  out the wrapped shape.
- [Carousel drag jitter vs scroll-snap] → `snap-none` during drag; math from
  drag-start offset; smooth restore on sub-threshold release.
- [Missed checksum update] → self-detecting: `registry build` fails loudly
  with ChecksumMismatch.
- [New facade props change generated panels] → regenerated
  `generated/controls/*` are committed; compile-time exhaustiveness guards
  turn staleness into `cargo check` failures.
- [SSR/wasm] → all new behavior is client-side pointer/render logic inside
  existing components; the OTP mask prop is exercised by an SSR unit test in
  adico-primitives, and wasm32 checks stay in the validation matrix.

## Migration Plan

Additive only; no consumer migration. Playground is reinstalled via
`adico add --all --replace` (lockfile refresh reviewed in-diff). Other
fixtures/examples keep compiling — they reference only pre-existing parts.
Rollback = revert the change commits; no data or config involved.
