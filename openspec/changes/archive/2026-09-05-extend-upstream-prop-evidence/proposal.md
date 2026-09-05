## Why

A follow-on effort ("complete the component prop surface") wants to audit
every one of adico's 66 registry components against shadcn, Base UI,
dioxus-components, and dioxus-primitives and add every prop that's missing.
That audit needs a trustworthy per-component list of what each upstream axis
actually exposes, and today's `statics/catalogs/*.json` snapshots cannot
supply one for two of the four axes. All counts below are measured against
`statics/catalogs/shadcn.json` at revision `7c9eaba1c0a6404c990c144a654792e3313c650d`
(refreshed 2026-09-04) and `statics/catalogs/base-ui.json` at 2026-08-31; they
move again once task 5.1 regenerates the catalogs.

- `statics/catalogs/shadcn.json` captures only **63 explicit props across 326
  parts**. `packages/adico-xtask/src/catalog/shadcn.rs` has no `cva()`
  extraction, so shadcn's own `Button` entry records only `asChild` — its
  entire `variant`/`size` API, defined via `cva(..., { variants: { variant:
  {...}, size: {...} } })`, is invisible. This isn't only a `Button`
  problem: the same `ComponentProps<"tag"> & VariantProps<typeof
  fooVariants>` shape recurs across many of shadcn's 326 parts (e.g.
  `alert.root`'s real type is `React.ComponentProps<"div"> &
  VariantProps<typeof alertVariants>`) and today matches neither of
  `shadcn.rs`'s two existing regexes, so those parts sit among the 139
  `unavailable` parts rather than just missing two props on `Button`.
- Of shadcn's 326 parts, 141 are `props_source: inherits_from`. **109** of
  those specifically point at `"radix.<component>.<part>"`; the other 32
  point at other unregistered axes (`vaul` (8, the `Drawer` fork), `cmdk`
  (7, `Command`), `@shadcn/react/message-scroller` (5), 11
  `@/registry/new-york-v4/ui/*` shadcn-internal self-references, and one
  `base-ui.*` reference that already resolves directly). No axis besides
  `radix` is registered in `catalog::AXES`, so every one of the 141
  references is dangling. Measured: of the 109 `radix.*` references, **57**
  already resolve verbatim against `statics/catalogs/base-ui.json` (shadcn
  is mid-migration from Radix to Base UI, as `shadcn.rs`'s own module doc
  already notes); of the remaining 52, most are part-id/component-id
  renames (radix `content` → Base UI `popup`, radix `dropdown-menu` → Base
  UI `menu`, radix `hover-card` → Base UI `preview-card`, etc.), not
  genuine absences.
- `statics/catalogs/dioxus-components.json` marks 136 of 235 parts
  `unavailable`, even though most of those parts are inline
  function-argument components that `rust_introspect.rs` already knows how
  to read — this is a wiring gap in the props lookup, not a parsing
  limitation. The same gap affects the `dioxus-primitives` axis (7 of 161
  parts `unavailable`), since both axes route through the same
  `dioxus_shared::parts_from_introspection` helper.

Without this, "make sure we have all the shadcn props" cannot be verified —
it would rest on manually re-reading shadcn source component by component,
which is exactly the kind of undocumented, unenforced judgment call this
project's `statics/*` tooling exists to replace (see
`openspec/specs/upstream-catalog-tooling/spec.md`'s existing purpose
statement).

## What Changes

- Extend `packages/adico-xtask/src/catalog/shadcn.rs` to parse a local
  `const <alias> = cva(<base>, { variants: { <group>: { <key>: <value>,
  ... }, ... }, defaultVariants: { ... } })` definition referenced by a
  component's `VariantProps<typeof <alias>>` type, and emit each variant
  group as an explicit prop (name = group name, e.g. `variant`/`size`; type
  = the literal union of its keys; default = the matching
  `defaultVariants` entry). This is additive to the existing
  `React.ComponentProps<typeof X>` passthrough and `& { ... }` augmentation
  parsing already in that file — it does not replace them.
- Add a small, explicit, hand-maintained alias table (component-id and
  part-id renames observed between the two projects — not a heuristic, not
  a network fetch), verified entry-by-entry against the committed
  `base-ui.json`, used only when **resolving** an already-fetched
  `inherits_from: "radix.*"` reference. This is deliberately *not* a fifth
  network-fetched axis: `catalog fetch shadcn` continues to record what it
  observes (including raw `radix.*` references, unchanged, matching
  `shadcn.rs`'s existing "recorded as observed" module doc), and resolution
  happens downstream, in the new `prop-parity` command below, at read time.
  Measured result of the corrected table, against the fetched
  `statics/catalogs/shadcn.json` (revision `7c9eaba1c0a6404c990c144a654792e3313c650d`,
  refreshed 2026-09-05, post-`cva`-fix): **88 of 107** `radix.*`
  references resolve; the other 19 (`menubar.*`'s 10 sub-parts,
  `alert-dialog.action`/`cancel`, `toggle-group.item`, and the genuine
  absences `aspect-ratio`/`slot`/`label`/`popover.anchor`/
  `navigation-menu.indicator`) are recorded unresolved with a specific,
  named reason each — never a generic one, and never guessed. (Before the
  `cva` fix, the same revision's catalog had 109 `radix.*` references and
  90 resolved; the `cva` fix independently moved `tabs.list` and
  `toggle.root` from `inherits_from` to `explicit`, since both components
  also declare a `cva()` variant group — the same upstream revision can
  yield different catalog content across parser versions, which is why
  every count here is pinned to a catalog `refreshedAt` date, not only an
  upstream revision.) A reference on a non-`radix` axis with no catalog of
  its own (`vaul`, `cmdk`, `@shadcn/react/*`) or a shadcn-internal
  self-reference (`@/registry/*`) gets its own distinct reason too, rather
  than the misleading "no resolvable Base UI equivalent" message a generic
  fallback would produce for a question that was never about Base UI.
- A shadcn part frequently carries **both** a local `explicit` augmentation
  prop and a `composition` reference to the upstream primitive it also
  wraps (`Dialog.Content`'s `showCloseButton` on top of the `dialog.popup`
  primitive it wraps; 25 of shadcn's 61 real components have this shape).
  `prop-parity` resolves and merges both prop sources for such a part
  (local names take precedence over same-named inherited ones) rather than
  comparing only the local augmentation prop and silently missing every
  prop the wrapped primitive itself declares.
- Extend `packages/adico-xtask/src/catalog/dioxus_shared.rs`'s shared
  `parts_from_introspection` helper (used by both the `dioxus-components`
  and `dioxus-primitives` fetchers) so an inline function-argument
  component (already understood by `rust_introspect.rs`) is recorded
  `explicit`, closing most of `dioxus-components`' 136 `unavailable` parts
  and `dioxus-primitives`' 7.
- Add a new generator, `cargo xtask prop-parity sync|check|diff`
  (`packages/adico-xtask/src/prop_parity.rs`), following the
  `primitive_usage.rs`/`styling_usage.rs` `sync|check|diff` command
  structure (`sync` writes, `check` regenerates in memory and diffs without
  writing — using `write_if_changed` so two successive `sync` runs are
  idempotent, `diff` prints the delta; all three run offline against
  committed `statics/catalogs/*.json`, never fetching), with one
  deliberate departure: unlike `primitive_usage.rs`, whose `sync` preserves
  hand-edited JSON prose across regeneration, `prop-parity`'s output is
  **100% derived** — see design.md's reason-source decision. For each of
  the 66 `registry:ui` items and each applicable upstream axis, it maps the
  adico item to that axis's component id the same way
  `component_compat.rs` already does (plain kebab-case identity plus its
  existing `SHADCN_EXCEPTIONS`/`DIOXUS_COMPONENT_EXCEPTIONS` tables — not a
  new mapping table), joins adico's own declared props (via
  `rust_introspect::FileIntrospection`, already `Serialize`) against that
  axis's resolved upstream props, and classifies every upstream prop as
  `present`, `missing`, `intentional_difference`, or `adico_extension`
  (adico props with no upstream counterpart). The last two require a
  `reason`, sourced from a fixed `const` table in `prop_parity.rs` keyed by
  (item, part, prop) — never hand-edited into the generated JSON — mirroring
  `primitive_usage.json`'s `classification`/`reason`/`followUp` shape in
  spirit, but not its preserve-on-sync mechanism. Output: one committed
  file per item, `statics/prop_parity/<item>.json`.
- Prop-name normalization (camelCase → snake_case, `className` → `class`,
  React-only structural props — `render`, `style`, `className`,
  `nativeButton`, `inputRef`, `asChild` — and Dioxus-structural props —
  `children`, `attributes`, `Element` slots — classified
  `intentional_difference` rather than compared 1:1) is a fixed,
  hand-maintained table in `prop_parity.rs`, not a heuristic. See design.md.
- Wire `cargo xtask prop-parity check` into `.github/workflows/ci.yml` and
  the validation matrix in `docs/validation.md`, alongside the six existing
  `*-usage`/`*-compat` `check` commands.
- **Non-goal**: this change does not add, remove, or modify any prop on any
  `registry/ui/*.rs` or `adico-primitives` file. It produces evidence only.
  Acting on that evidence (adding missing props, fixing inconsistencies) is
  a separate, later change that consumes `statics/prop_parity/*.json`.
- **Non-goal**: no fifth network-fetched catalog axis for Radix. The
  measured 57/109 direct-resolution rate plus the observed rename pattern
  make a resolution table sufficient; see design.md's rejected-alternative
  note.

## Capabilities

### New Capabilities

- `adico-prop-parity`: a committed, CI-gated, offline-regenerable record
  (`statics/prop_parity/<item>.json`, one per registry item) that joins
  adico's own declared component props against each applicable upstream
  axis's resolved props and classifies every upstream prop as present,
  missing, an intentional difference, or absent from adico with a recorded
  reason — the evidence base a later prop-completion change is generated
  from.

### Modified Capabilities

- `upstream-catalog-tooling`: `catalog fetch shadcn` additionally extracts
  `cva()`-declared variant-group props (previously invisible); dangling
  `inherits_from: "radix.*"` references become resolvable against the
  `base-ui` axis through a documented alias table, rather than remaining
  permanently dangling; `catalog fetch dioxus-components` records
  significantly fewer parts as `unavailable`. The shared catalog schema
  (`CatalogSnapshot`/`PartEntry`/`PropsSource`) is unchanged.

## Impact

- `packages/adico-xtask/src/catalog/shadcn.rs`: `cva()` parsing, new unit
  tests using a `cva`-shaped fixture alongside the existing
  `DIALOG_FIXTURE` test.
- `packages/adico-xtask/src/catalog/dioxus_shared.rs`: inline-component
  extraction fix in the shared `parts_from_introspection` helper, covering
  both the `dioxus-components` and `dioxus-primitives` axes.
  `catalog/dioxus_components.rs` keeps its own equivalent fallback for its
  bespoke (non-shared) builder.
- `packages/adico-xtask/src/catalog/radix_aliases.rs`: the component-id/
  part-id alias table, the non-`radix`-axis and known-unresolved reason
  tables, and the resolution function.
- New `packages/adico-xtask/src/prop_parity.rs`: the `sync|check|diff`
  command, its normalization table, its item/part mapping (reusing
  `component_compat.rs`'s exception tables), and its reason tables.
- `packages/adico-xtask/src/main.rs`: dispatch wiring + usage string.
- `statics/catalogs/shadcn.json`, `statics/catalogs/dioxus-components.json`:
  regenerated content (requires network; run once during implementation,
  not part of this proposal).
- `statics/prop_parity/*.json` (new, 66 files, committed).
- `docs/development.md`, `docs/validation.md`: document the new command and
  add it to the validation matrix.
- `.github/workflows/ci.yml`: add the `prop-parity check` step.
- No change to `registry/`, `adico-primitives`, `adico-registry-core`,
  `adico-cli`, or any consumer-facing behavior.
