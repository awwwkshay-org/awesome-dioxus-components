# Follow-up discovered by this change

## `BubbleInput`'s hidden native input escapes every ancestor's overflow clip

**Where:** `packages/adico-primitives/src/checkbox.rs`, `BubbleInput`
(checkbox.rs:295-325). `switch.rs` renders the same shape.

**What:** the hidden native input that carries form state is rendered with

```rust
position: "absolute",
pointer_events: "none",
opacity: "0",
transform: "translateX(-100%)",
```

and **no positioned ancestor**. With no `position: relative|absolute|fixed`
between it and the root, its containing block is the *initial* containing
block — the `<html>` box. An ancestor's `overflow` only clips a descendant
whose containing block is at or below that ancestor, so `main`'s
`overflow-y: auto` does not clip this input. It contributes to
`documentElement.scrollHeight` instead.

**Why it matters here specifically.** This app deliberately never scrolls the
document: `main.rs` is `h-dvh … overflow-hidden` and `main` is the scrollport,
because the playground's percentage `flex-basis` splits need a definite
ancestor height (see `2026-09-24-merge-apps-into-web` task 9.8). A stray
absolutely-positioned descendant that escapes that clip makes the *document*
scrollable, which is exactly the state the layout is built to prevent.

**Evidence** (measured against a real `dx build` served statically, so this is
not a dev-server artifact):

| route | `documentElement.scrollHeight` | viewport | scrolls? |
| --- | --- | --- | --- |
| `/docs/components/checkbox` | 977 | 900 | **yes** |
| `/docs/components/switch` | 900 | 900 | no |
| `/docs/components/button` | 900 | 900 | no |

Whether it manifests depends on where the control lands: the escaped input
sits at the control's static position, so it only pushes past the viewport
when the control is far enough down a long page. That is why `switch`'s page
does not currently leak and `checkbox`'s does — the difference is content
height, not correctness.

**Pre-existing, not introduced here.** `/docs/components/checkbox` already
leaked before this change (those example pages came from
`2026-09-24-docs-component-examples`). Adding a `Checkbox` to the landing page
surfaced the same defect on `/`, which is how it was found.

**Mitigated, not fixed, in this change.** `apps/web/src/pages/index.rs`'s
showcase row establishes a containing block (`relative`) so the input is
clipped by `main` again. That is ordinary composition around an existing API,
which `adico-web-structure` explicitly prefers over patching a registry
component for one section's convenience — but it only fixes this app's own
usage. Every other consumer, and `/docs/components/checkbox`, still leaks.

**Real fix:** make `BubbleInput` not depend on an accident of ancestry. Either
give it a positioned wrapper of its own, or use the standard visually-hidden
pattern (`clip-path`/1px box) instead of `position: absolute` with a
transform. This is `packages/adico-primitives` source, consumer-visible
through every installed `checkbox` and `switch`, so it wants its own change
with its own delta against `adico-primitives`.
