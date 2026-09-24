## Context

See `proposal.md` — Why. The mechanics:

- `packages/adico-cli/src/css.rs`'s `theme_region()` emits, in order: an
  `@theme` block of colour and radius tokens, `:root { … }` light values,
  `.dark { … }` dark values, the animation utilities, and the scrollbar CSS.
- `adico_primitives::theme_mode` (theme_mode.rs:165-176) applies appearance by
  adding/removing the class `dark` on `document.documentElement`.
- Nothing emits `@custom-variant dark`, so Tailwind v4's default applies and
  `dark:` compiles to `@media (prefers-color-scheme: dark)`.

## Goals / Non-Goals

**Goals**
- One signal governs appearance. Tokens and `dark:` utilities agree.
- Consumers get it without knowing the trap exists.

**Non-Goals**
- No change to token names or values.
- Not a migration tool. Existing projects pick this up when they next refresh
  their theme region; nothing reaches into a consumer's file otherwise.

## Decisions

### D1 — Emit the variant from `theme_region()`, at the top of the region

The declaration is placed first in the region, before `@theme`. Tailwind needs
the variant registered before the utilities that use it are generated, and the
top of the region is also where a reader looking for "how does dark work here"
will land, immediately above the `.dark` block it pairs with.

*Alternative rejected — leaving it to each consumer.* That is the status quo,
and it requires every consumer to independently discover a trap whose symptom
(hover colours wrong for *some* users) points nowhere near its cause.

*Alternative rejected — emitting it outside the marker region.* The CLI only
owns the region; writing outside it would mean editing consumer-authored
bytes, which `plan_theme_install` deliberately never does.

### D2 — `&:is(.dark *)`, matching the `.dark` block it pairs with

The selector mirrors what the region already generates for tokens. It matches
descendants of `.dark`, which is what the class-on-`<html>` mechanism
produces.

*Note:* this does not match an element that *is* `.dark` itself, only its
descendants — the same semantics the `.dark { --… }` token block has, since
tokens cascade to descendants. The two therefore agree by construction.

### D3 — Remove `apps/web`'s hand-written line in the same change

Leaving it would mean the app declares the variant twice: once by hand above
the marker and once inside the regenerated region. Harmless to render, but it
would make the app look like the exception that still needs the workaround,
and its comment would describe a problem that no longer exists. The README
section that documents the line is updated for the same reason.

## Risks / Trade-offs

- **Visible rendering change for existing consumers on refresh** → the point of
  the change, recorded in the proposal's Impact. The affected utilities are
  few and all hover/focus/invalid-state surfaces, so the change is real but
  contained.
- **A consumer who *wanted* OS-driven `dark:`** → they would have been getting
  it inconsistently with their own tokens, which is not a coherent state to
  preserve. A consumer who genuinely wants OS-driven appearance selects
  "System" in the theme control, which resolves to the class.
- **Two sources of the same declaration during the transition** → resolved by
  D3 for this repo; for a consumer, the generated one simply wins or
  duplicates harmlessly until they delete theirs.

## Migration Plan

No forced action. Consumers inherit the variant the next time the theme region
is written. `apps/web` is updated in this change so the repository's own app
demonstrates the generated output rather than a hand-patched version of it.
