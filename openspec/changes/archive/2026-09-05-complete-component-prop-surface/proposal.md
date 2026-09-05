## Why

`extend-upstream-prop-evidence` (Change A, complete) produced
`statics/prop_parity/<item>.json` — a generated, offline-verifiable,
per-item record of every upstream prop (Base UI, dioxus-primitives,
dioxus-components, shadcn) each of adico's 66 `registry:ui` items has, is
missing, or intentionally differs from. That evidence, read directly (not
estimated), shows:

- **41 of 66 items have at least one genuine missing upstream prop; 228
  such gaps total** (best-matching axis per item — `combobox` 29,
  `slider` 26, `select` 16, `calendar` 14, `pagination` 12,
  `navigation-menu` 11, down to single-prop gaps on `accordion`/`button`/
  `label`/`toggle`/etc.). The other 25 items (`alert`, `badge`, `card`,
  `data-table`, `skeleton`, `spinner`, `table`, and 19 more) have no
  evidence-backed gap at all on any axis.
- Most gaps are **native form-integration props** (`value`, `form`,
  `required`, `read_only`/`readOnly`, `name`, `id`) and
  **completion/status callbacks** (`onOpenChangeComplete`,
  `onValueCommitted`), not missing variants — e.g. `Switch` is missing
  `value`, `unchecked_value`, `form`, `required`, `read_only`, `id`;
  `Progress` is missing `min`, `format`/`get_aria_value_text`, `locale`.
- adico ships **no `radius` prop anywhere** (`grep`-verified: zero
  components declare a rounding-control prop; `theme_builder.rs`'s own
  `radius: String` field is an unrelated CSS-custom-property *value*
  editor, not a per-component prop), while **50 of 66** registry files
  hard-code a `rounded-*` Tailwind utility that a `radius` prop would need
  to own instead.
- adico ships **no `loading` prop anywhere**, even on the seven
  actionable components (`Button`, `InputGroupButton`, `PaginationLink`,
  `AlertDialogAction`, `ToolbarButton`, `SidebarMenuButton`, `DataTable`)
  where shadcn's own convention is to compose `<Button disabled><Spinner
  /></Button>` by hand at each call site — adico already ships a `Spinner`
  registry item with nothing composing it into these controls.
- Only **8 of 66** items declare a `variant` prop and only **5 of 66**
  declare `size`, even though shadcn declares a `variant`/`size` pair on
  the majority of its components (Change A's `cva()` extraction now makes
  this comparison possible for the first time).
- Real, verified internal inconsistencies exist independent of any
  upstream comparison: four spellings of "disabled" (`bool`,
  `Option<bool>`, `ReadSignal<bool>`, native-only via `attributes`), two
  controlled-value shapes (`Option<ReadSignal<Option<T>>>` vs
  `ReadSignal<Option<T>>`), three handler-naming conventions (`onclick` /
  `on_click` / `on_select`), and `Slider`/`RangeSlider` having **no
  dedicated `class` field at all** — confirmed in `registry/ui/slider.rs`'s
  own code comment, which documents the current hand-rolled
  `with_class()` workaround this forces on every call site. Two registry
  facades silently narrow what their primitive already exposes:
  `Checkbox` drops the primitive's form `value` and `attributes`;
  `ToastProvider` narrows `ReadSignal<...>` primitive props to plain
  values, so a consumer cannot reactively change toast duration or
  capacity — the same defect class already recorded for `Menubar`'s
  dropped `disabled` in `docs/adico/component-hardening-audit.md`.

`adico-existing-components`'s own "Current registry scope reaches complete
applicable parity" requirement is scoped to only the 21 first-wave items
and predates the 66-item registry this repo now ships; it needs to widen
to reflect the whole current registry and to name Change A's `prop-parity`
evidence as its verification mechanism, alongside a new set of prop
*convention* requirements this codebase has never had a name for before.

## What Changes

- **New registry item `variants`** (`registry:lib`, alongside `cn`) at
  `registry/lib/variants.rs`: a shared `Radius` enum
  (`None`/`Sm`/`Default`/`Lg`/`Xl`/`Full`) with the established `fn
  class(self) -> &'static str` inherent-impl convention 14 registry files
  already use. **No shared `Size` enum** — the five existing per-component
  size enums (`ButtonSize`, `AvatarSize`, `NativeSelectSize`,
  `SwitchSize`, `ToggleSize`) stay; see design.md's rejected-alternative
  note.
- **Prop convention normalization**, each a named, hand-verified decision
  in design.md: a single `disabled` representation per component
  category; `class: Option<String>` on every styled component (closing
  `Slider`/`RangeSlider`/`Toast`/`AreaThumb`/`HueSlider`, preferring the
  primitive-owning fix over a registry wrapper where the facade is a bare
  re-export); `#[props(extends = GlobalAttributes)]` + `attributes:
  Vec<Attribute>` on every styled component rendering a real element (25
  of 66 already have it; extending toward the rest); restoring
  `Checkbox`'s dropped `value`/`attributes` and `ToastProvider`'s dropped
  `ReadSignal` reactivity; one handler-naming rule per component category.
  **BREAKING**: the controlled trio standardizes on
  `ReadSignal<Option<T>>`. Verified against current source (not the
  originally assumed 5): only the *singular*-select functions —
  `Select`'s and `Combobox`'s `value` parameter and `TagGroup`'s `value`
  parameter — declare the double-Option `Option<ReadSignal<Option<T>>>`
  shape; their `SelectMulti`/`ComboboxMulti`/`TagGroupMulti` counterparts
  already use the standard single-Option `ReadSignal<Option<Vec<T>>>` for
  their own `values` parameter. So 3 items (`select.rs`, `combobox.rs`,
  `tag_group.rs`), each touching one function, not five. A public
  prop-type change for those three items' consumers, landing through the
  normal `adico add --replace` upgrade path like any other registry source
  change, not a runtime migration.
- **`radius: Radius`** added to every component with a visible bounded
  surface, with the existing `rounded-*` utility stripped from *both* base
  class strings and any per-variant/size `fn class(self)` impl (`cn()` is
  a joiner, not tailwind-merge, so a coexisting hard-coded `rounded-*`
  would not be overridden) — enforced by a new unit test in the
  `styling-usage` generator, not a one-off grep.
- **`loading: bool` + `loading_text: Option<String>`**, composed over the
  existing `Spinner` registry item (declared in `registryDependencies`,
  never duplicated), on the seven actionable components named above,
  setting `aria-busy` and native `disabled` while loading. Recorded as
  `adico_extension` with a reason in each item's
  `statics/prop_parity/<item>.json` — the reason-table mechanism Change A
  built specifically for this case.
- **Parity-prop waves**, driven directly by `statics/prop_parity/*.json`'s
  `missing` entries, organized into 5 waves by relationship (composite
  controls, menu/overlay family, native leaf/form controls, dialog/overlay
  content, Dioxus-only/composite misc) rather than by arbitrary count.
  Every `missing` entry ends the change either implemented or reclassified
  as `intentional_difference` with a written reason — never silently left
  `missing`. `as_child` stays absent (Dioxus has no merge-props
  mechanism).
- **Propagate**: re-run `registry build`, `primitive-usage sync`,
  `styling-usage sync`, `component-compat sync`, `prop-parity sync`;
  refresh consumer copies through the real CLI path (`adico add
  --replace`), never by hand-editing `apps/playground/src/components/ui/*.rs`.

## Capabilities

### Modified Capabilities

- `adico-existing-components`: "Every installed component has a hardening
  record" and "Current registry scope reaches complete applicable parity"
  widen from the 21 first-wave items to all 66, naming
  `statics/prop_parity/*.json` as the mechanical evidence source
  alongside the existing Dioxus Components/shadcn comparison.
- `adico-registry`: new requirement — every registry item's prop surface
  follows the shared conventions this change establishes (disabled
  representation, `class`, `attributes` extends, handler naming,
  controlled trio), mechanically checkable the same way behavior-ownership
  and styling classification already are.
- `adico-primitives-authorship`: its existing "feature parity with both
  reference libraries" requirement (driven by `primitive-compat diff`)
  extends to also close a gap `prop-parity` surfaces when the fix belongs
  in the primitive rather than the registry facade (e.g. `Slider`'s
  missing `class` field, since its registry facade is a bare re-export of
  the primitive's own props type).

## Impact

- New `registry/lib/variants.rs`, `registry/registry.json` entry, and
  `registryDependencies` update on every `radius`-bearing item.
- Most of `registry/ui/*.rs` (50 of 66 for `radius` alone; 41 of 66 for at
  least one parity-prop wave item; convention normalization touches
  disabled/class/attributes/handlers across many more).
- `packages/adico-primitives/src/{slider,checkbox,toast,color_picker}.rs`
  and any other primitive a registry facade re-exports verbatim, where the
  convention fix belongs upstream of the facade.
- `registry/registry.json`, `registry/generated/**`.
- `statics/primitive_usage/*.json`, `statics/styling_usage/*.json`,
  `statics/component_compatibility.json`, `statics/prop_parity/*.json`
  (all 66, re-synced).
- `packages/adico-xtask/src/styling_usage.rs`: new unit test enforcing no
  stray `rounded-*` literal outside `Radius::class()`.
- `docs/adico/component-hardening-audit.md`: updated with the newly closed
  `Checkbox`/`ToastProvider` narrowing defects.
- Consumer fixtures (`examples/basic-spa`, `examples/basic-ssr`,
  `tests/installation/*`) refreshed through the real `adico add --replace`
  path, never edited directly.
- No change to `adico-cli`'s installation mechanics or `registry-core`'s
  resolution logic — this change is entirely within registry source,
  primitive source, and their generated/derived records.
