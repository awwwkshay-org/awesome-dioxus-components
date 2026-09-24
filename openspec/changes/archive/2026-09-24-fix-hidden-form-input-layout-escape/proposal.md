## Why

`Checkbox` renders a hidden native `<input>` so the control participates in
real form submission. That input is styled
`position: absolute; opacity: 0; transform: translateX(-100%)` — but with **no
width or height**, so it keeps its natural ~13×13px box, and with no
positioned ancestor, so its containing block is the *initial* containing
block.

An ancestor's `overflow` only clips a descendant whose containing block sits at
or below that ancestor. This input's does not, so no scroll container clips it:
it contributes to `documentElement.scrollHeight` from wherever the checkbox
happens to sit.

In any layout that deliberately scrolls inside a container rather than at the
document — which is exactly what `apps/web` does, because the playground's
percentage `flex-basis` splits need a definite ancestor height — a `Checkbox`
placed low on a long page silently makes the **document** scrollable.

Measured against a real `dx build` served statically, so this is not a
dev-server artifact:

| route | `documentElement.scrollHeight` | viewport | scrolls? |
| --- | --- | --- | --- |
| `/docs/components/checkbox` | 977 | 900 | **yes** |
| `/docs/components/switch` | 900 | 900 | no |
| `/docs/components/button` | 900 | 900 | no |

`Switch` renders the same kind of hidden input and does **not** leak, because
its style string already includes `width: 0; height: 0`. The repository
already contains the fix; `Checkbox` simply never received it.

`2026-09-24-polish-web-shell-and-landing` mitigated this in `apps/web` by
establishing a containing block on one wrapper. That fixed one app's landing
page and nothing else: `/docs/components/checkbox` still leaks, and so does
every consumer project that installs `checkbox`.

## What Changes

- **`Checkbox`'s hidden form-mirror input stops occupying layout**, matching
  what `Switch`'s equivalent already does.
- The app-level mitigation in `apps/web/src/pages/index.rs` is removed, since
  the cause is gone.

Non-goals: no API change, no visual change, no change to form-submission
behavior or to the checked-state sync. No other primitive is touched —
`Switch` is already correct, and a speculative audit of every `onmounted` or
`absolute` in the crate is not in scope.

## Capabilities

### Modified Capabilities

- `adico-primitives`: gains a requirement that a primitive's hidden
  form-participation input must not affect document layout or scroll extent.

## Impact

**Modified**
- `packages/adico-primitives/src/checkbox.rs` — the hidden input's styling.
- `apps/web/src/pages/index.rs` — drop the now-unnecessary mitigation.
- `registry/registry.json`, `packages/adico-cli/embedded/registry.json`,
  `apps/web/src/components/ui/*`, `apps/web/adico.lock` — only if the
  primitive change alters installed registry source (it should not; the
  hidden input lives in the primitive, not in `registry/ui/checkbox.rs`).

**Risk: this is a primitive every consumer depends on.** The input remains a
real, form-participating `<input type="checkbox">` with the same name, value,
and checked state; only its box size changes, and it was already invisible
(`opacity: 0`, `pointer-events: none`, `aria-hidden`, `tabindex="-1"`).

**Verification surface:** the measured leak must disappear on
`/docs/components/checkbox`; `Checkbox`'s own primitive tests and the
Playwright suites that exercise checkbox behavior must be unchanged.
