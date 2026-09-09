## Why

A manual pass over the Adico Playground surfaced nine problems spanning
registry component source, the owned primitive layer, and playground demo
composition: a hard UI hang on the drag-and-drop page, layout misalignment in
the color picker, calendar, and analog clock, an avatar whose fallback text
doesn't scale with its size, and a missing semantic tone vocabulary — only
`Default`/`Destructive` exist today, so `Badge::Verified` and `Toast` hardcode
`emerald-600`/`amber-500` and ignore the consumer's theme palette. Fixing
these now keeps the registry's existing-component surface trustworthy before
more components are added on top of it.

## What Changes

- Fix a UI-hanging bug on the playground's Drag And Drop List page: the page
  calls `use_drag_and_drop_list_items()` from an ancestor scope of the
  `DragAndDropContext` provider, so `consume_context` cannot find it. Also fix
  two secondary bugs found at the same site: a conditionally-called hook in
  `DragAndDropListItems`, and `watch_document_drop` never calling `ctx.drop()`
  on an `"end"` message.
- Remove the Calendar demo's doubled popover frame: `CalendarView` (already
  `border bg-popover p-3`) is nested inside a `PopoverContent` that also
  carries `border bg-popover p-4`, producing two visible borders with a seam.
- Add the `ColorPickerContent` export that `adico-existing-components` already
  promises but that does not exist in the repo, and make `ColorArea` track the
  width of its siblings (`HueSlider`, `ColorPickerFields`) instead of a
  hardcoded `size-48` square, so the picker's parts render with consistent
  width and spacing.
- Fix the analog clock overflowing its container in `DateTimePicker`: the
  playground page imposes a `Digital`-derived fixed height on a view
  (`Analog`) whose own documentation says `fill_height` has no effect for it.
- Scale Avatar's fallback text with `AvatarSize`, not just the circle (the
  circle already scales correctly via `size-8`/`size-10`/`size-12`, but
  `AvatarFallback` carries no `text-*` class, so its text renders at ambient
  size in every circle). This introduces the first shared-state coupling
  between `Avatar` and `AvatarFallback`.
- Add a shared `Tone` vocabulary (`registry/lib/variants.rs`) covering
  `Default`, `Success`, `Warning`, `Error`, `Info`, backed by new
  `--success`/`--warning`/`--info` (+ `-foreground`) theme tokens in the
  adico-managed CSS marker region and in `ThemeBuilder` (`Tone::Error` still
  resolves to the pre-existing `--destructive` token — only the Rust name is
  new). Give `Toast`'s `info` type real border styling (previously
  unstyled) using this vocabulary.
- Give `Alert` the full tone set (`Success`, `Warning`, `Info`, alongside the
  existing `Default`/`Destructive`, kept as one enum — see design.md) built on
  the `Tone` vocabulary above.
- **BREAKING (registry API — renamed/removed variants):** Split `Button`,
  `Badge`, and `TagOption`'s single variant enum into two orthogonal props,
  per explicit user direction given after the first implementation pass: a
  **shape** enum (`Primary`, `Secondary`, `Outline`, `Ghost`, `Link` — renamed
  from `Default`/`Secondary`/`Outline`/`Ghost`/`Link`) and a separate
  **`color: Tone`** prop. `Destructive` is removed from `ButtonVariant`/
  `BadgeVariant` (reachable instead via `color: Tone::Error` on any shape);
  `BadgeVariant::Verified` is removed outright (reachable via `variant:
  Primary, color: Tone::Success`). `color: Tone::Default` reproduces each
  shape's exact pre-existing look, so the visual only changes for a
  non-`Default` color. `TagOption` gains a new `variant: TagOptionVariant`
  (it had no shape enum before) and `color: Tone`, both affecting only its
  *selected*-state look — the resting look is unchanged. `Kbd` is explicitly
  excluded from this split (kept exactly as-is, no variant system at all,
  per the user's direction). See design.md's Decisions for the full
  before/after and why this supersedes the delegation model originally built
  for these three components.
- **BREAKING (DOM shape change for every `Input` consumer using
  `type="password"`):** Move the password-reveal affordance into `Input`
  itself, driven by `r#type == "password"`, reversing a documented decision
  that the toggle must be a consumer-side composition
  (`apps/playground/src/pages/input.rs:30-32`). Collapse the playground's
  two-field Input demo to one field.
- **BREAKING (default value change):** Flip `InputOTP`'s `mask` default to
  `true` and promote the playground's hand-rolled reveal-toggle button into a
  registry-owned part, so masking-by-default with an explicit reveal is the
  out-of-the-box behavior rather than an opt-in a consumer must build.

**Explicitly out of scope:** extending the shape+`color` prop split to any
component beyond `Button`, `Badge`, and `TagOption` — `Kbd` is explicitly
excluded per direct user direction (no variant system at all, left untouched);
`Alert` and `Toast` keep the single-enum tone model they already have from
this same change (not revisited); `Bubble`, `Message`, `Item`, `Marker`,
`Progress`, and other presentational items keep their current styling
entirely. Adding new `AvatarSize` steps (e.g. `Xs`/`Xl`) beyond the existing
`Sm`/`Default`/`Lg`. Any desktop/mobile-native validation (no fixture exists
in this repo for that surface). Changing the fixed-position/portal behavior of
popover-family primitives beyond what each individual group's fix requires.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `adico-existing-components`: adds a shared tone vocabulary and semantic
  tokens; gives Alert the full tone set; makes Avatar's size scale its whole
  presentation, not just the circle; makes Input reveal a password value from
  within the component; modifies the existing ColorPicker trigger-composition
  requirement so `ColorPickerContent` actually exists; modifies the existing
  DateTimePicker requirement so the analog view fits its popup surface;
  modifies the existing Input OTP masking requirement so masking defaults to
  on and a reveal toggle is a registry-owned part.
- `adico-playground-structure`: modifies the existing requirement that demo
  pages render only the demoed component's real composition, to cover the
  Input page's collapse to one field and (if confirmed during
  implementation) the Drag And Drop List page's corrected hook usage.

## Impact

- `registry/lib/variants.rs`: new `Tone` enum with `solid_class`/`soft_class`/
  `outline_class`/`ghost_class`/`link_class`.
- `registry/ui/{button,badge,tag_group,alert,toast,avatar,input,input_otp,
  color_picker,time_picker,date_time_picker,calendar,theme_builder}.rs`:
  variant/prop additions, layout fixes, token adoption. `button.rs`/`badge.rs`/
  `tag_group.rs` specifically: their shape enum is renamed/reduced
  (`Default`→`Primary`, `Destructive`/`Verified` removed) and a new `color:
  Tone` prop is added.
- `packages/adico-primitives/src/drag_and_drop_list.rs`: hook-scope,
  conditional-hook, and cleanup fixes (exact file TBD by group 2's
  reproduction step — may be confined to the playground page instead).
- `packages/adico-cli/src/css.rs`: new theme tokens in the managed marker
  region, plus the 23 checked-in `tailwind.css` files that carry it
  (`apps/playground`, `examples/basic-spa`, `examples/basic-ssr`, and 20
  `tests/installation/*` fixtures), refreshed through the real CLI, not by
  hand-editing.
- `apps/playground/src/pages/{drag_and_drop_list,calendar,color_picker,
  date_time_picker,avatar,input,input_otp}.rs`: demo composition fixes.
- `apps/playground/src/generated/controls/*.rs`: regenerated via
  `playground-controls sync` for every enum-variant addition — required
  before the build compiles again, since the generated exhaustiveness guards
  break on a new variant.
- `statics/{prop_parity,styling_usage,primitive_usage}/*.json`: regenerated
  for every touched registry item. `statics/styling_usage/badge.json` and
  `toast.json` need particular attention — badge's record documents a
  deliberate rationale for the emerald hardcode that this change reverses,
  and toast's record already inaccurately claims full token compliance.
- Installed copies in `examples/basic-{spa,ssr}` and
  `tests/installation/*-consumer` reinstalled through `adico add <item>
  --replace`, not hand-copied.
- No database, container, or mobile-native surface is touched.
- Breaking changes affect any external consumer's already-installed copy of:
  `Input` (DOM shape under `type="password"`); `InputOTP` (default `mask`
  value); and `Button`/`Badge`/`TagOption` (renamed/removed variant
  identifiers — `Default`→`Primary`, `Destructive` and `Badge::Verified`
  removed, reachable instead via the new `color` prop). This registry ships
  source, not a versioned dependency, so existing installed copies are
  unaffected until a consumer re-runs `adico add --replace`.
