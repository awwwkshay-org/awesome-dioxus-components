## Why

adico switches appearance with a **class**: `adico_primitives::theme_mode`
adds and removes `dark` on `<html>`, and the generated theme region defines
its dark token values under a `.dark` selector.

But the generated region never emits `@custom-variant dark`. Without it,
Tailwind v4's *default* `dark` variant applies, and every literal `dark:`
utility compiles to `@media (prefers-color-scheme: dark)` — following the
**operating system** while every token-driven colour follows the app's own
toggle. For any user whose OS appearance differs from the one they chose, the
two contradict each other.

This is not hypothetical, and it is not confined to one app: the affected
utilities ship inside registry source that every consumer installs —
`dark:hover:bg-accent/50` (Button's ghost variant),
`dark:aria-invalid:ring-destructive/40`, `dark:hover:bg-white/10`,
`dark:text-emerald-500`, and the `dark:focus-visible:ring-*/40` set. A
consumer who installs `button`, picks Dark while their OS is Light, and hovers
a ghost button gets the wrong hover surface, with nothing in their own code to
explain why.

`2026-09-24-redesign-web-visual-foundation` fixed this for `apps/web` by
declaring the variant by hand above the marker region, and recorded in its
`FOLLOWUPS.md` that the durable fix belongs in the generator so every consumer
inherits it. This is that change.

## What Changes

- **The generated theme region emits `@custom-variant dark (&:is(.dark *))`**,
  so literal `dark:` utilities resolve against the same class the theme
  control writes.
- **`apps/web`'s hand-written declaration is removed**, since the generator now
  provides it. Its README section is updated to match.

Non-goals: no change to which tokens are generated or to their values; no
change to `ThemeMode`, `ModeToggle`, or `ThemeSwitcher`; no change to any
`registry/ui/*.rs`.

## Capabilities

### Modified Capabilities

- `adico-existing-components`: the requirement governing the adico-managed CSS
  theme region gains the rule that the region declares the `dark` variant
  against the theme class, so class-based switching governs literal `dark:`
  utilities rather than the OS preference.

## Impact

**Modified**
- `packages/adico-cli/src/css.rs` — `theme_region()` emits the variant.
- `apps/web/tailwind.css` — remove the now-duplicated hand-written
  declaration; the generated region supplies it.
- `apps/web/README.md` — the section documenting the hand-written line.

**Risk: this is a visible rendering change for existing consumer projects.**
Any project that refreshes its theme region will see literal `dark:` utilities
start following the theme class instead of the OS. That is the fix, but it is
a behavior change, not a no-op, and it is why this is its own change rather
than a quiet edit folded into app work.

Consumers are not forced into it: the region is only rewritten when they run a
command that reinstalls it. A project that never refreshes keeps today's
behavior.

**Verification surface:** the emitted region must contain the variant; the
compiled stylesheet must resolve `dark:` utilities against `:is(.dark *)`
rather than a media query; and `apps/web` must behave identically after its
hand-written line is removed, since the generator now supplies the same rule.
