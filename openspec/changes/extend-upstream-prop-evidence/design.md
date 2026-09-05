## Context

See `proposal.md` - Why for the motivating evidence gap.

Relevant existing pieces:
- `packages/adico-xtask/src/catalog/schema.rs` already defines the shared
  `CatalogSnapshot`/`CatalogEntry`/`PartEntry`/`PropsSource` types used by
  all four axes. `PropsSource::InheritsFrom { reference }`'s own doc comment
  already flags the exact gap this change closes: *"Kept as a string rather
  than a resolved link because the target may be on an axis this snapshot
  doesn't itself carry (see design.md's Radix open question)"* — this
  change is that resolution.
- `packages/adico-xtask/src/catalog/shadcn.rs` regex-scans each exported
  function's prop type expression (`parse_type_expression`) for
  `React.ComponentProps<typeof X(.Part)?>` (→ `inherits_from` +
  composition) and a trailing `& { ... }` object literal (→ `explicit`
  augmentation props). It resolves `X` to an axis via
  `resolve_import_axis`, which reads the file's own `import { X as Alias }
  from "module"` statements and maps `radix-ui` → axis `"radix"`,
  `base-ui` → axis `"base-ui"`. It has no handling at all for
  `VariantProps<typeof buttonVariants>`, shadcn's actual `cva()`-based
  variant API — that type reference doesn't match `React.ComponentProps<...>`
  at all, so today it silently contributes nothing.
- `packages/adico-xtask/src/rust_introspect.rs` (`syn`-based) already
  extracts `#[derive(Props)]` struct fields and, separately, inline
  function-argument component parameters (used by `component_compat.rs`).
  `FileIntrospection` derives `Serialize`.
- `packages/adico-xtask/src/{primitive_usage,styling_usage}.rs` establish
  the `sync|check|diff` command structure this change's `prop_parity.rs`
  follows: `sync` writes one committed file per registry item under
  `statics/<kind>/<item>.json`; `check` regenerates in memory and diffs
  without writing, exiting non-zero on any difference; `diff` prints the
  delta; all three read only committed `statics/catalogs/*.json`, never
  fetching. `prop_parity.rs` does **not** follow their preserve-on-sync
  mechanism — see the "Reasons ... live in `prop_parity.rs`" decision
  below for the one deliberate departure.
- Measured overlap between shadcn's dangling `radix.*` references and the
  already-fetched `statics/catalogs/base-ui.json`, against
  `statics/catalogs/shadcn.json` at revision
  `7c9eaba1c0a6404c990c144a654792e3313c650d` (2026-09-04) and
  `statics/catalogs/base-ui.json` at 2026-08-31: of shadcn's 326 parts, 141
  are `props_source: inherits_from`. **109** of those specifically target
  `"radix.<component>.<part>"`; the other 32 target axes with no catalog of
  their own at all (`vaul` 8, `cmdk` 7, `@shadcn/react/message-scroller` 5,
  `@/registry/new-york-v4/ui/*` 11, `base-ui.*` 1 — already resolves
  directly). Of the 109 `radix.*` references, **57 resolve verbatim** (same
  component id, same part id). Of the 52 that don't, the pattern is mostly
  renames, not absences — enumerated, corrected, and verified one-by-one
  against `base-ui.json` in the Decisions section below.
  (An earlier pass of this document reported 141/74/67 for these same
  numbers; 141 was actually the all-axes `inherits_from` total, not the
  `radix.*` subset, and the earlier alias table both missed several
  verifiable renames and contained one entry — `radio-group.root` — that
  resolved to the wrong Base UI part. Both are corrected here.)

## Goals / Non-Goals

**Goals:**
- Make shadcn's `variant`/`size`-style API (its dominant prop-declaration
  pattern, used by the large majority of its 61 components) visible in
  `statics/catalogs/shadcn.json`.
- Make shadcn's Radix-inherited props resolvable against Base UI's already-
  complete, 100%-explicit catalog, without a second scraper.
- Reduce `dioxus-components.json`'s (136/235) and `dioxus-primitives.json`'s
  (7/161) `unavailable` rates by fixing the actual wiring gap (inline
  components), not by inventing new extraction logic.
- Produce one committed, CI-gated, per-item prop-parity record that a later
  change can turn directly into a task list, with false positives
  eliminated by an explicit normalization table.
- Keep every new/changed command offline-safe per
  `upstream-catalog-tooling`'s existing "Catalog fetch is the sole
  network-touching command" requirement — only `catalog fetch shadcn` and
  `catalog fetch dioxus-components` touch the network, and only because
  they already did.

**Non-Goals:**
- Do not add a `radix` axis to `catalog::AXES`, and do not write a Radix
  scraper. See Decisions.
- Do not change any prop on any registry or primitive source file. This
  change produces evidence; acting on it is out of scope (see proposal.md).
- Do not attempt automatic prop-name translation as a general heuristic
  (e.g. inferring `readOnly` → `read_only` structurally, then guessing).
  The normalization table is explicit and hand-maintained, matching this
  project's established preference (`playground_controls.rs`'s prop-shape
  allowlist, `component_compat.rs`'s `SHADCN_EXCEPTIONS` table) for a fixed
  allowlist over a heuristic wherever the classification result feeds a
  human-facing report.
- Do not resolve every one of shadcn's 67 non-trivially-resolving `radix.*`
  references. Some (shadcn's `command`, `label`) have no Base UI equivalent
  at all and correctly stay unresolved.

## Decisions

### shadcn `cva()` extraction is a second, independent pattern in `shadcn.rs`, not a replacement for the existing one

shadcn's real `button.tsx` shape (and the majority of its components) is:

```tsx
const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 ...",
  {
    variants: {
      variant: { default: "...", destructive: "...", outline: "...", secondary: "...", ghost: "...", link: "..." },
      size: { default: "...", sm: "...", lg: "...", icon: "..." },
    },
    defaultVariants: { variant: "default", size: "default" },
  }
)

function Button({ className, variant, size, asChild = false, ...props }: React.ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & { asChild?: boolean }) { ... }
```

Today, `parse_type_expression`'s `typeof\s+(\w+)(?:\.(\w+))?` regex matches
`typeof buttonVariants`, `resolve_import_axis(source, "buttonVariants")`
correctly returns `None` (it's a local const, not an import), so
`composition` stays empty — and the trailing `& { asChild?: boolean }`
augmentation is correctly captured. This is why today's output is exactly
`asChild` and nothing else: the existing code already handles this file
shape *except* for the `cva()` call itself.

Add a second regex pass, independent of `parse_type_expression`: when a
signature's type expression contains `VariantProps<typeof (\w+)>`, look for
a `const <same alias> = cva(` definition earlier in the same source file,
and parse its `variants: { <group>: { <key>: ..., ... }, ... }` object
(each top-level key inside `variants` is a group name; each key inside a
group is a variant value) and its `defaultVariants: { ... }` object.  Emit
one `Prop` per group:

```json
{"name": "variant", "type": "\"default\" | \"destructive\" | \"outline\" | \"secondary\" | \"ghost\" | \"link\"", "default": "default"}
{"name": "size", "type": "\"default\" | \"sm\" | \"lg\" | \"icon\"", "default": "default"}
```

merged into the same part's `props` list alongside any `& { ... }`
augmentation props (so `Button`'s `root` part ends up with `variant`,
`size`, and `asChild` — all three of its real props). No change to
`PropsSource`'s shape: this is still `Explicit { props }`.

### Radix references resolve downstream, in `prop-parity`, against the already-fetched `base-ui` axis — not via a new scraper, and not by rewriting `shadcn.json` at fetch time

Rejected: a `radix.rs` fetcher registered as a fifth `AXES` entry. The
measured 57/109 direct-match rate against the *existing* `base-ui.json`,
plus the fact that `shadcn.rs`'s own module doc already documents shadcn as
"mid-migration from Radix to Base UI" (i.e. Radix is being designed out of
shadcn, not a stable long-term target), means a second scraper would
duplicate most of what Base UI's catalog already has, for a
shrinking fraction of components, at the cost of a new network dependency
this project would then own indefinitely (`upstream-catalog-tooling`'s own
"Catalog fetch is the sole network-touching command" requirement already
treats each fetcher as a maintenance liability worth minimizing).

Rejected: rewriting `shadcn.rs` to resolve and normalize the `radix.*`
reference into `base-ui.*` at fetch time, so `statics/catalogs/shadcn.json`
itself stores the resolved reference. This would require `catalog fetch
shadcn` to read another axis's committed file mid-fetch, coupling one
axis's fetcher to another axis's on-disk state and reintroducing exactly
the "did you fetch base-ui first" ordering hazard that the read-only
snapshot model was designed to avoid. It would also contradict
`shadcn.rs`'s own existing module doc, which is explicit that composition
axis is *"recorded as observed, not normalized"* — that's a deliberate
existing property worth keeping, not a bug.

Decision: resolution happens only when `prop-parity` (and, in the future,
`component-compat` if it wants this too) reads an `inherits_from:
"radix.<component>.<part>"` reference. A small, explicit, hand-maintained
alias table — not a heuristic — translates component ids and part ids
observed to differ between the two projects, based on this repo's actual
measured data. **An alias entry is added only after its target is verified
against the committed `base-ui.json`** — resolving to a *plausible but
semantically wrong* part (comparing adico's props against the wrong
upstream part list) is worse than leaving the reference unresolved, because
`prop-parity`'s output then reports authoritative-looking wrong evidence
instead of an honest gap. Every entry below carries a pinned unit test
asserting its expected target, not just its existence:

| kind | radix | base-ui |
|---|---|---|
| component | `dropdown-menu` | `menu` |
| component | `hover-card` | `preview-card` |
| component | `radio-group` | `radio` |
| part | `overlay` | `backdrop` |
| part | `content` (dialog/popover/select/tooltip/menu/preview-card/context-menu only) | `popup` |
| part | `content` (accordion/collapsible/tabs only) | `panel` |
| part | `collapsible-content` | `panel` |
| part | `collapsible-trigger` | `trigger` |
| part | `trigger` (tabs only) | `tab` |
| part | `scroll-area-scrollbar` | `scrollbar` |
| part | `scroll-up-button`/`scroll-down-button` (select only) | `scroll-up-arrow`/`scroll-down-arrow` |
| part | `sub` (context-menu/dropdown-menu→menu only) | `submenu-root` |
| part | `sub-content` (context-menu/dropdown-menu→menu only) | `popup` |
| part | `root` (radio-group→radio only) | `group` |
| part | `item` (radio-group→radio only) | `root` |

The last two entries are the fix for a real defect found while verifying
this table: without both, the generic component-alias-then-pass-through-
part-id path resolves `radix.radio-group.root` (Radix's *group container*)
straight onto Base UI's `radio.root` (the *individual radio button*,
`value*`/`nativeButton`/`disabled`/`readOnly`/`required`) instead of
`radio.group` (`name`/`defaultValue`/`value`/`onValueChange`/`form`/
`disabled`/`readOnly`/`required`) — the correct target. `radio-group.item`
has the mirrored problem in reverse. Both are now pinned tests in
`radix_aliases.rs`.

The shipped generic entry `scroll-bar → scrollbar` is dropped: no reference
in the data uses the id `scroll-bar` (the real id is
`scroll-area-scrollbar`), so it was dead and is replaced by the
component-scoped entry above.

Applied as: first translate the component id (if it has a component-level
alias), then translate the part id (checking the component-scoped part
aliases before the generic ones).

**Left unresolved, each with a specific, named reason — not a generic
one:**
- `menubar`'s 10 sub-parts (`checkbox-item`, `content`, `group`, `menu`,
  `portal`, `radio-group`, `radio-item`, `separator`, `sub`,
  `sub-content`, `trigger`): Base UI's `menubar` component carries only a
  `root` part (which *does* resolve directly and legitimately — its
  `loopFocus`/`modal`/`disabled`/`orientation` props are a genuine match
  for Radix's `Menubar.Root`); its actual menu items come from the
  separate `menu` component. Mapping menubar's sub-parts onto menu's parts
  is a composition judgment, not a mechanical rename, and is deliberately
  left unmade rather than guessed.
- `alert-dialog.action`/`alert-dialog.cancel`: Base UI's `AlertDialog`
  exposes only a single `close` part; it doesn't distinguish an
  affirmative action from a cancel the way Radix does.
- `toggle-group.item`: Base UI's `toggle-group` component carries only a
  `root` part; the individual toggle button is the separate `toggle.root`
  component, not a sub-part of `toggle-group` — the same kind of
  composition judgment as `menubar`, left unmade.
- Genuine absences: `aspect-ratio`, `slot`, `label` (no Base UI component
  at all), `popover.anchor`, `navigation-menu.indicator` (no such part on
  an otherwise-present component).

**Measured result of this corrected table against the fetched catalog
(revision `7c9eaba1c0a6404c990c144a654792e3313c650d`, refreshed
2026-09-05, post-`cva`-fix): 88 of 107 `radix.*` references resolve; 19
are unresolved**, each with the specific reason above, never the generic
"no resolvable Base UI equivalent" message. (Pre-`cva`-fix, the same
upstream revision's catalog had 109 `radix.*` references and 90 resolved
against this same table — `tabs.list` and `toggle.root` independently
moved from `inherits_from` to `explicit` once the `cva` extraction also
recognized their variant groups, so the denominator shrank by 2 with the
19-item residual unchanged. The same upstream revision can yield different
catalog *content* across parser versions, which is why counts here are
pinned to a `refreshedAt` date, not only a revision sha.)

**Non-`radix` axes get their own reason, checked before any Base UI
lookup is attempted.** The `inherits_from` references that don't target
`radix` (30 post-`cva`-fix; 32 before it, for the same reason as above --
`attachment`'s `action` part and `sidebar`'s menu-button both gained a
`cva()`-detected variant group) would otherwise fall through to
`resolve_inherits_from`'s generic Base UI lookup and produce a misleading
message — e.g. "no resolvable Base UI equivalent for component
`@/registry/new-york-v4/ui/button`", which isn't a Base UI question at
all. Two axis families are checked first: `vaul`/`cmdk`/`@shadcn/react/*`
(20 refs: shadcn's `Drawer`/`Command` forks and its message-scroller
extra, none of which have a catalog axis in this project) get
"third-party dependency, no catalog axis"; `@/registry/*` (9 refs post-fix:
shadcn's own components referencing each other, e.g. `field`'s `label`
part reusing `label`'s `root`) get "shadcn-internal self-reference, not
resolved in this change" — resolving those *is* possible with no new data
(both sides are already in `shadcn.json`), but is deliberately deferred
rather than folded into this change; see the Risks section.

### A part with both local augmentation and a composition reference joins both, not just the local one

Discovered while spot-checking the first real `prop-parity sync` run:
`schema.rs`'s `PartEntry` deliberately keeps `composition` and
`props_source` as independent fields (`upstream-catalog-tooling`'s
"Upstream composition and adico's own composition are tracked separately"
requirement) — so a part can be `props_source: explicit` (a local
augmentation prop like `showCloseButton`) while its `composition` field
still names the upstream primitive it also wraps (`radix.dialog.content`).
This is not rare: **25 of shadcn's 61 real components** have this shape
(`Dialog.Content`, `Sheet.Content`, every Radix `*.Item`'s
`inset`/`variant` augmentation, `Avatar.Root`'s `size`, and others).
`prop_parity.rs`'s first pass compared only the local `explicit` list
against adico, silently missing every prop the wrapped primitive itself
declares (e.g. `Dialog.Content`'s real `initialFocus`/`finalFocus` never
appeared as `missing` at all, because they live on the *inherited* side).

Decision: when a part's `props_source` is `Explicit` and its `composition`
is non-empty, `prop-parity` also resolves the composition reference
(reconstructed as `"<axis>.<component>.<part-or-root>"`, routed through
the same `resolve_props_source`/`resolve_inherits_from` machinery as a
real `inherits_from` reference) and merges its resolved props into the
comparison set, local names taking precedence over same-named inherited
ones. A composition reference that itself doesn't resolve degrades to
just the local explicit props (not to `unresolved` for the whole part) --
the local augmentation is still real, known data even when the wrapped
primitive's own props aren't determinable.

### Prop-name normalization table, in `prop_parity.rs`

A fixed `BTreeMap`/match covering:
- **Casing**: upstream `camelCase` → adico `snake_case` for comparison
  purposes only (the upstream name is still displayed verbatim in the
  report).
- **Renames**: `className` → `class`.
- **React-only structural props**, classified `intentional_difference`
  with a fixed reason ("Dioxus has no render-prop/style-object
  equivalent"): `render`, `style`, `nativeButton`, `inputRef`, `asChild`
  (`packages/adico-primitives/src/field.rs:6` already documents Dioxus has
  no merge-props mechanism).
- **Dioxus-structural props**, matched against adico's `children`/
  `attributes: Vec<Attribute>` rather than reported as 1:1 missing props:
  an upstream `children` prop is `present` if adico's component has a
  `children: Element` field; an upstream prop whose only purpose is
  attribute/event passthrough (Base UI's per-part global HTML attribute
  spread) is `present` if adico forwards `#[props(extends = ...)]`
  `attributes`.
- Everything not in the table is compared by name after casing
  normalization; a genuine miss is `missing`, not silently dropped.

This table is intentionally small and reviewed by hand as it grows — same
posture as `component_compat.rs`'s `SHADCN_EXCEPTIONS`/
`DIOXUS_COMPONENT_EXCEPTIONS` tables, which this project already trusts for
judgment calls that shouldn't be inferred.

### Item and part mapping reuses `component_compat.rs`, not a second table

`prop-parity` needs to map an adico registry item (e.g. `switch`) to each
axis's matching entry, and each adico *part* to each axis's matching part,
before it can join props. Both already have an established mechanism in
this repo, and `prop-parity` reuses them rather than growing parallel ones:

- **Item mapping**: plain kebab-case identity between the adico registry
  item's `name` and the catalog entry's `id`, exactly as
  `component_compat.rs:181-216`'s `build_catalog_axis` already does against
  `load_registry_items`. The hand-maintained
  `SHADCN_EXCEPTIONS`/`DIOXUS_COMPONENT_EXCEPTIONS` tables
  (`component_compat.rs:152-179`) are reused as-is for the handful of
  known status/note overrides (e.g. `separator`'s `not_applicable`); no
  new exception table is introduced.
- **Part mapping**: `catalog/case.rs:34`'s `part_id_for(prefix_source,
  name)` — the same function every fetcher already uses to derive a part
  id from a PascalCase component name (`DialogRoot` → `root`) — is applied
  to adico's own `rust_introspect`-derived component names too, so both
  sides of the join are produced by one rule rather than two independently
  maintained ones that could drift apart.
- An axis with no counterpart for an item (e.g. `dioxus-components` has no
  `Combobox`) is recorded as `matchedComponent: null` with an empty
  `parts` array — a real, correct answer ("nothing comparable exists on
  this axis"), not an omitted key.

### Reasons for `intentional_difference`/`adico_extension` live in `prop_parity.rs`, not in the generated JSON — `prop-parity`'s one deliberate departure from the `primitive_usage.rs` idiom

`adico-prop-parity`'s requirements need to hold simultaneously: every
`intentional_difference`/`adico_extension` classification carries a
`reason`, and `prop-parity check` fails on a hand-edited record. Those are
only compatible if `sync` can *derive* the reason on every run — and
nothing can derive *why* adico has `radius` (an adico extension with no
upstream counterpart at all) or *why* `render` is an intentional
difference from first principles at comparison time.

**Decision: every reason is a string literal in a fixed `const` table in
`prop_parity.rs`, keyed by `(item, part, prop)` for item-specific reasons
(e.g. `("button", "root", "radius")` → "adico extension: not part of
shadcn's or Base UI's Button API, added for consistent visual rounding
across the registry") and by `prop` alone for the structural-props table
that already exists for the normalization rule above (`render`, `style`,
`nativeButton`, `inputRef`, `asChild`). `sync` looks up the reason at
generation time; it never reads reason text back out of a previously
written `statics/prop_parity/*.json` file.** This keeps the generated JSON
100% derived from source, so `check`'s byte-comparison (regenerate in
memory, diff against committed output, fail on any difference — including
a hand-edit) works exactly as `specs/adico-prop-parity/spec.md` already
describes, and matches this repo's existing house rule —
`upstream-catalog-tooling`'s "Hand-maintained judgment data survives fetch
and sync" requirement already establishes that judgment lives in
adico-xtask source and is joined in at sync time, not stored in a fetched
or generated file — plus `component_compat.rs`'s `SHADCN_EXCEPTIONS`
precedent for exactly this shape of hand-maintained table.

This is a **deliberate departure** from `primitive_usage.rs`/
`styling_usage.rs`, worth naming explicitly because an earlier pass of
this document said `prop_parity.rs` follows their `sync|check|diff` idiom
"exactly." It does, for the write mechanics (write once, diff-on-check),
but not for preservation: `primitive_usage::sync`
(`primitive_usage.rs:218-238`) re-derives only `primitive_modules` and
*preserves* whatever hand-written `reason`/`followUp` prose already exists
on disk via `load_record(&path).ok()`; a malformed or missing record is
silently treated the same and gets a fresh default. `prop-parity` does
not have this preserve-branch at all — an implementer copying
`primitive_usage.rs`'s structure must leave it out, or `sync`'s
"idempotent, fully regenerated" behavior and `check`'s "fails on
hand-edit" requirement silently stop being true together.

### `prop-parity` output shape

One file per registry item, `statics/prop_parity/<item>.json`:

```json
{
  "adicoItem": "switch",
  "axes": {
    "base-ui": {
      "matchedComponent": "switch",
      "parts": [
        {
          "id": "root",
          "props": [
            {"name": "checked", "status": "present"},
            {"name": "value", "status": "missing"},
            {"name": "readOnly", "status": "missing"},
            {"name": "className", "status": "present", "note": "adico's `class`"},
            {"name": "render", "status": "intentional_difference", "reason": "Dioxus has no render-prop/style-object equivalent"}
          ]
        }
      ]
    },
    "dioxus-components": { "matchedComponent": null, "parts": [] }
  }
}
```

`status` is one of `present | missing | intentional_difference |
adico_extension`; the last two require `reason`. `matchedComponent: null`
+ empty `parts` records "this axis has nothing comparable for this item"
rather than omitting the axis key.

## Addendum: `attributes: Vec<Attribute>` extends coverage (found while gathering evidence for Change B)

Spot-checking real `prop-parity sync` output while drafting Change B's
evidence-driven task list surfaced a real false-positive class: 51 `missing`
entries across 7 items (`button`, `input`, `item`, `pagination`, `sidebar`,
`textarea`, `toggle`) were native DOM event names (`onchange`, `onmounted`,
`onkeydown`, ...) that the `dioxus-components`/`dioxus-primitives` axes
enumerate individually, but that adico covers generically via the
established `#[props(extends = GlobalAttributes)]` convention's
`attributes: Vec<Attribute>` field (confirmed present on 6 of the 7 by hand
— `toggle` genuinely has no such field, so its 3 entries were real gaps).
`prop_parity.rs`'s classifier only ever compared by exact canonical name, so
it had no way to know `attributes` covers these.

Fixed: `classify_upstream_prop` now also treats an upstream name matching
the native-event shape `^on[a-z]+$` (all-lowercase, no separating
underscore — distinct from adico's own `on_snake_case` semantic-callback
convention, so this can't misfire on a real adico prop) as `present` when
adico declares a field named `attributes` with a type containing
`Attribute`. This dropped the false-positive count from 51 to 11 (the
genuine `toggle` gaps plus a few others that turned out to lack the
`attributes` field), confirmed by re-running `prop-parity sync` and
diffing the `missing` counts before and after. Four new unit tests cover
the addition, including that adico's own `on_checked_change`-style
callbacks are never mistaken for a native event.

## Risks / Trade-offs

- [The `cva()` regex parser is fragile against a genuinely arbitrary TSX
  shape — nested template literals, a `cva` call split across a helper
  function, a component that doesn't destructure `variant`/`size` at all]
  -> Accepted, matching this file's existing posture: `parse_type_expression`
  already only handles the patterns actually observed in shadcn's real
  source, and unmatched shapes fall through to `PropsSource::Unavailable`
  rather than crashing or emitting wrong data — the same fallback this
  change's `cva` pass uses.
- [`parse_cva_variant_groups(source, alias)` only searches the file it's
  given a signature from, but shadcn sometimes imports a `cva()`-built
  variants object across files instead of declaring it locally (e.g.
  `toggle-group` importing `toggleVariants` from `badge`/`toggle`) — a
  distinct failure mode from a same-file helper-function split, and common
  enough to name on its own rather than folding into the risk above] ->
  Accepted, same fallback: an unmatched cross-file import falls through to
  `PropsSource::Unavailable`, visible in the before/after `explicit`
  count from task 5.1 rather than silently miscounted. This is the most
  likely reason a post-regeneration count comes in lower than the
  file-local worked example predicts.
- [The alias table is hand-maintained and can go stale if shadcn continues
  migrating off Radix and part names shift again] -> Low risk: as shadcn
  finishes its Base UI migration, its `import`s increasingly resolve
  directly to axis `"base-ui"` already (`resolve_import_axis` already
  handles that case with no alias needed), shrinking the table's relevance
  over time rather than growing it. `prop-parity check` is CI-gated, so a
  newly-dangling reference is visible, not silent. Every entry is also
  pinned by its own unit test (see the Radix decision above), so a future
  edit that makes an entry plausible-but-wrong — the class of defect this
  document's own alias table had before verification — fails immediately
  rather than shipping.
- [The 11 `@/registry/new-york-v4/ui/*` shadcn-internal self-references are
  left unresolved even though both sides already exist in the same
  `shadcn.json` snapshot, so resolving them needs no new data] -> Deferred,
  not accepted as a permanent gap: recorded here as a concrete follow-up
  for a later change, since it's a same-axis join rather than a
  cross-axis one and doesn't fit this change's "resolve against
  already-fetched base-ui" framing without a second, different resolution
  path.
- [`prop-parity`'s normalization table under-classifies a genuinely new
  React-only pattern as `missing` when it should be
  `intentional_difference`] -> Mitigated: `check` is a required review gate
  before any change based on the output ships (see proposal.md's
  non-goal — this change produces evidence, a later change acts on it and
  will surface a wrong classification as an implausible task).

## Open Questions

None — the cva-extraction pattern, the Radix-resolution-via-alias-table
decision (replacing the "Radix open question" `schema.rs` referenced), the
reason-source decision, the item/part mapping decision, and the
`prop-parity` output shape are the load-bearing unknowns and are resolved
above.
