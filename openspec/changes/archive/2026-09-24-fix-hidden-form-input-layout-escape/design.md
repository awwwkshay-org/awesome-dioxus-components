## Context

See `proposal.md` — Why. The two implementations, side by side:

```rust
// checkbox.rs:306-320 -- leaks
position: "absolute", pointer_events: "none", opacity: "0",
margin: "0", transform: "translateX(-100%)",

// switch.rs:134 -- does not leak
style: "transform: translateX(-100%); position: absolute; \
        pointer-events: none; opacity: 0; margin: 0; width: 0; height: 0;",
```

The only difference that matters is `width: 0; height: 0`. Verified
empirically: a zero-area absolutely-positioned box creates no scrollable
overflow in Chromium even when its static position sits past the viewport — on
`/docs/components/switch` one such input measured `bottom: 919` against a
900px viewport while `documentElement.scrollHeight` stayed exactly 900.

## Goals / Non-Goals

**Goals**
- One correct implementation of "hidden but form-participating", not two.
- A fix at the cause, so every consumer gets it rather than each app
  rediscovering the workaround.

**Non-Goals**
- No audit of unrelated `position: absolute` usages. `Switch` is already
  correct and nothing else in the crate matches this pattern
  (`grep 'position: "absolute"'` finds exactly one other site, which is this
  one).
- No change to `use_bubble_input_sync` or to how checked state is mirrored.

## Decisions

### D1 — Zero the box, matching `Switch`

`Checkbox`'s hidden input gains `width: 0; height: 0`.

*Alternative rejected — the `sr-only` / `clip-path: inset(50%)` pattern.* It is
the more conventional visually-hidden idiom and would also work, but it is a
*different* third approach in a crate that already has a working one. Two
implementations of the same idea is what caused this; adding a third to fix it
would be worse. If the crate ever standardises on `clip-path`, both sites
should move together.

*Alternative rejected — giving the input a positioned wrapper.* That makes the
input clipped by whichever ancestor scrolls, which fixes the symptom, but
leaves a transparent 13px box participating in layout inside that wrapper —
capable of affecting flex/grid sizing at the call site. Removing the box
entirely is the stronger property.

*Alternative rejected — leaving the app-level mitigation in place.* It fixed
one wrapper in one app. `/docs/components/checkbox` still leaked, and so did
every consumer.

### D2 — Remove the app-level mitigation in the same change

`apps/web/src/pages/index.rs`'s `relative` wrapper exists only to contain this
input. Leaving it would be a workaround with no remaining cause, and its
comment would become a false explanation for a future reader. The audit that
caught the original leak is re-run to confirm the primitive fix alone is
sufficient.

## Risks / Trade-offs

- **A zero-size form control is unusual** → it is `aria-hidden`,
  `tabindex="-1"`, `opacity: 0` and `pointer-events: none` already; it is
  never focused or clicked, only read by form serialisation. `Switch` has
  shipped this way.
- **Native validation UI anchors to the control's box** → these mirrors carry
  no validation constraints (no `required`, no `pattern`), so there is no
  bubble to anchor.
- **Consumer-visible primitive change** → behaviorally inert; only the box
  changes.

## Migration Plan

No consumer action. Projects that already installed `checkbox` pick the fix up
when they next update, and nothing breaks if they do not — the leak is a layout
nuisance, not a correctness failure, for consumers that scroll at the document.
