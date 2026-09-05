//! `cargo xtask prop-parity sync|check|diff`: for each of the 66
//! `registry:ui`/`registry:component` items, joins the item's own declared
//! props (via `rust_introspect.rs`) against each catalog axis's
//! (`base-ui`, `dioxus-components`, `dioxus-primitives`, `shadcn`) resolved
//! upstream props and classifies every upstream prop as `present`,
//! `missing`, or `intentional_difference`, plus any adico-only prop with a
//! recorded reason as `adico_extension`. Output:
//! `statics/prop_parity/<item>.json`, one per item.
//!
//! Fully offline: reads only committed `statics/catalogs/*.json` and the
//! current registry source, exactly like `component_compat.rs`. Item and
//! part mapping reuse this project's existing mechanisms rather than
//! growing new ones: an item maps to an axis's entry by plain kebab-case id
//! equality (the same match `component_compat.rs`'s `build_catalog_axis`
//! uses), and adico's own components map to part ids via
//! `catalog::part_id_for` -- the exact function every catalog fetcher
//! already uses to derive its own part ids, so both sides of the join come
//! from one rule.
//!
//! Every `intentional_difference`/`adico_extension` reason is a string
//! literal in a fixed `const` table below, looked up at generation time --
//! **never** read back from a previously generated
//! `statics/prop_parity/*.json` file. This is `prop_parity`'s one
//! deliberate departure from `primitive_usage.rs`/`styling_usage.rs`'s
//! idiom: those preserve hand-edited JSON prose across `sync`; this
//! module's output is 100% derived from source on every run, which is what
//! lets `check` do a plain regenerate-and-byte-compare (design.md's
//! "Reasons ... live in `prop_parity.rs`" decision).
//!
//! An adico-only prop with no upstream counterpart and no entry in
//! `ADICO_EXTENSION_REASONS` is not classified at all (neither `missing`
//! nor `adico_extension`) -- Change A's evidence goal is upstream gap
//! coverage; retroactively hand-labeling every incidental adico
//! convenience field (`aria_label` and similar) as a reasoned "extension"
//! is deliberately out of scope here. The mechanism is fully implemented
//! and unit-tested so a later change (e.g. one that adds `radius`) only
//! needs to add its table entry for the classification to take effect.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use adico_registry_core::RegistryItem;
use serde::Serialize;

use crate::catalog::{self, CatalogSnapshot, PropsSource, ResolvedProps};
use crate::registry_introspect::{find_primitive_modules, load_registry_items, registry_root};
use crate::rust_introspect::{self, FileIntrospection};
use crate::write_if_changed;

fn records_dir(root: &Path) -> PathBuf {
    root.join("statics/prop_parity")
}

fn record_path(root: &Path, item_name: &str) -> PathBuf {
    records_dir(root).join(format!("{item_name}.json"))
}

// --- Output schema -----------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Present,
    Missing,
    IntentionalDifference,
    AdicoExtension,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct PropStatus {
    name: String,
    status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct PartParity {
    id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    props: Vec<PropStatus>,
    /// Set instead of `props` when this part's upstream prop list could not
    /// be determined at all (no matching part on the axis, an
    /// `unavailable` upstream entry, or an `inherits_from` reference that
    /// didn't resolve) -- never silently treated as zero missing props.
    #[serde(skip_serializing_if = "Option::is_none")]
    unresolved: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct AxisParity {
    matched_component: Option<String>,
    parts: Vec<PartParity>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct PropParityRecord {
    adico_item: String,
    axes: BTreeMap<String, AxisParity>,
}

// --- Prop-name normalization (design.md's normalization decision) -------

/// Structural props with no adico equivalent by framework design: always
/// `intentional_difference` regardless of any adico field match, each with
/// a fixed, specific reason. Named for the common case (a React-only prop
/// with no Dioxus equivalent), but also covers `r#as` -- a Dioxus-side
/// structural prop from the upstream dioxus-components fork this crate
/// deliberately never ported, for the identical reason as `asChild`.
const REACT_ONLY_STRUCTURAL: &[(&str, &str)] = &[
    (
        "render",
        "Dioxus has no render-prop/style-object escape hatch to swap the rendered element",
    ),
    (
        "style",
        "Dioxus has no inline style-object prop; styling is done via `class`/Tailwind utilities",
    ),
    (
        "nativeButton",
        "Dioxus components render a single fixed element type; there is no prop to swap between a native <button> and a generic element",
    ),
    (
        "inputRef",
        "Dioxus uses component-local signals/mounted-node hooks for DOM access, not React ref forwarding",
    ),
    (
        "asChild",
        "Dioxus has no merge-props/slot mechanism to render a caller-supplied element in the component's place",
    ),
    (
        "actionsRef",
        "Dioxus uses component-local signals/mounted-node hooks for imperative access, not React imperative-handle refs",
    ),
    (
        "r#as",
        "The upstream dioxus-components fork's own polymorphic render-prop escape hatch (merges default behavioral attributes with caller-supplied ones and optionally renders a custom root/trigger element) is deliberately not ported anywhere in this crate -- default attributes are written directly on each element with `..props.attributes` spread after them instead, the same rationale as `asChild` above",
    ),
];

/// Explicit renames applied before generic camelCase-to-snake_case
/// conversion (`className` doesn't naturally snake-case to `class`). Only
/// safe here when the target spelling is consistent across *every* item
/// that has this upstream prop -- `className` -> `class` always is. A name
/// that denotes different concepts on different items (see
/// `ITEM_RENAMES`'s own doc comment) must not go in this table: it would
/// mark some unrelated item's same-named-but-different field `present`.
const RENAMES: &[(&str, &str)] = &[("className", "class")];

/// Per-item prop renames, keyed by `(item, raw_upstream_name)` -> adico's
/// own field name, consulted only for that one item. Unlike `RENAMES`,
/// these upstream names are *not* safe to rename globally: `keepMounted`
/// alone appears on over 30 unrelated parts across a dozen components in
/// `statics/catalogs/base-ui.json` (portals, indicators, panels), and
/// `delay`/`closeDelay` appear on every hover-intent trigger (menu,
/// popover, preview-card, tooltip) -- a global rename to `force_mount`/
/// `delay_ms`/`close_delay_ms` would mark an unrelated item's unrelated
/// field `present` by coincidence of spelling.
const ITEM_RENAMES: &[(&str, &str, &str)] = &[
    ("navigation-menu", "delay", "delay_ms"),
    ("navigation-menu", "closeDelay", "close_delay_ms"),
    ("navigation-menu", "keepMounted", "force_mount"),
];

fn item_canonical_name(item: &str, raw_name: &str) -> String {
    for (table_item, from, to) in ITEM_RENAMES {
        if *table_item == item && *from == raw_name {
            return (*to).to_string();
        }
    }
    canonical_name(raw_name)
}

/// Reason recorded for every `radius` extension entry below. `radius` has
/// no upstream counterpart on any axis -- adico exposes a consistent
/// corner-radius control shadcn/Base UI/dioxus-components/-primitives don't.
const RADIUS_REASON: &str =
    "adico extension: consistent corner-radius control not present upstream";

/// Reason recorded for every `loading`/`loading_text` extension entry
/// below. shadcn's own convention composes `<Button disabled><Spinner
/// /></Button>` by hand at each call site instead of a first-class prop.
const LOADING_REASON: &str = "adico extension: shadcn composes `<Button disabled><Spinner /></Button>` by hand; adico exposes it as a first-class prop instead";

/// Item-specific reasons for adico props with no upstream counterpart,
/// keyed by `(item, part, prop)`. Every `part` here is verified against the
/// real part id `part_id_for`/the catalog fetchers already produce for that
/// item (checked directly in `statics/prop_parity/<item>.json`, not
/// hand-derived from the naming rule alone -- several diverge from a naive
/// prefix-strip, e.g. `TagOption`/`ToggleItem`/`TabList`/`ColorArea` don't
/// share their item's own name as a literal prefix, so `part_id_for` falls
/// through to kebab-casing the whole component name instead of stripping
/// one).
const ADICO_EXTENSION_REASONS: &[(&str, &str, &str, &str)] = &[
    ("accordion", "trigger", "radius", RADIUS_REASON),
    ("alert", "root", "radius", RADIUS_REASON),
    ("alert-dialog", "content", "radius", RADIUS_REASON),
    ("attachment", "root", "radius", RADIUS_REASON),
    ("avatar", "root", "radius", RADIUS_REASON),
    ("avatar", "fallback", "radius", RADIUS_REASON),
    ("badge", "root", "radius", RADIUS_REASON),
    ("bubble", "content", "radius", RADIUS_REASON),
    ("button", "root", "radius", RADIUS_REASON),
    ("button-group", "text", "radius", RADIUS_REASON),
    ("calendar", "view", "radius", RADIUS_REASON),
    ("card", "root", "radius", RADIUS_REASON),
    ("color-picker", "color-area", "radius", RADIUS_REASON),
    ("combobox", "input", "radius", RADIUS_REASON),
    ("combobox", "list", "radius", RADIUS_REASON),
    ("command", "root", "radius", RADIUS_REASON),
    ("context-menu", "content", "radius", RADIUS_REASON),
    ("date-picker", "input", "radius", RADIUS_REASON),
    ("dialog", "content", "radius", RADIUS_REASON),
    ("drag-and-drop-list", "item", "radius", RADIUS_REASON),
    ("dropdown-menu", "content", "radius", RADIUS_REASON),
    ("dropdown-menu", "trigger", "radius", RADIUS_REASON),
    ("empty", "root", "radius", RADIUS_REASON),
    ("hover-card", "content", "radius", RADIUS_REASON),
    ("input", "root", "radius", RADIUS_REASON),
    ("input-group", "root", "radius", RADIUS_REASON),
    ("item", "root", "radius", RADIUS_REASON),
    ("kbd", "root", "radius", RADIUS_REASON),
    ("marker", "root", "radius", RADIUS_REASON),
    ("menubar", "root", "radius", RADIUS_REASON),
    ("menubar", "content", "radius", RADIUS_REASON),
    ("native-select", "root", "radius", RADIUS_REASON),
    ("navigation-menu", "trigger", "radius", RADIUS_REASON),
    ("navigation-menu", "content", "radius", RADIUS_REASON),
    ("navigation-menu", "link", "radius", RADIUS_REASON),
    ("pagination", "link", "radius", RADIUS_REASON),
    ("popover", "content", "radius", RADIUS_REASON),
    ("progress", "root", "radius", RADIUS_REASON),
    ("select", "trigger", "radius", RADIUS_REASON),
    ("select", "list", "radius", RADIUS_REASON),
    ("sidebar", "trigger", "radius", RADIUS_REASON),
    ("sidebar", "menu-button", "radius", RADIUS_REASON),
    ("slider", "track", "radius", RADIUS_REASON),
    ("slider", "range", "radius", RADIUS_REASON),
    ("slider", "thumb", "radius", RADIUS_REASON),
    ("switch", "root", "radius", RADIUS_REASON),
    ("tabs", "tab-list", "radius", RADIUS_REASON),
    ("tag-group", "tag-option", "radius", RADIUS_REASON),
    ("textarea", "root", "radius", RADIUS_REASON),
    ("toggle", "root", "radius", RADIUS_REASON),
    ("toggle-group", "toggle-item", "radius", RADIUS_REASON),
    ("toolbar", "button", "radius", RADIUS_REASON),
    ("tooltip", "content", "radius", RADIUS_REASON),
    ("button", "root", "loading", LOADING_REASON),
    ("button", "root", "loading_text", LOADING_REASON),
    ("input-group", "button", "loading", LOADING_REASON),
    ("input-group", "button", "loading_text", LOADING_REASON),
    ("pagination", "link", "loading", LOADING_REASON),
    ("pagination", "link", "loading_text", LOADING_REASON),
    ("alert-dialog", "action", "loading", LOADING_REASON),
    ("alert-dialog", "action", "loading_text", LOADING_REASON),
    ("toolbar", "button", "loading", LOADING_REASON),
    ("toolbar", "button", "loading_text", LOADING_REASON),
    ("sidebar", "menu-button", "loading", LOADING_REASON),
    ("sidebar", "menu-button", "loading_text", LOADING_REASON),
    ("data-table", "root", "loading", LOADING_REASON),
    ("data-table", "root", "loading_text", LOADING_REASON),
];

/// Reason recorded for Base UI's newer controlled-collection/virtualization
/// API on combobox-shaped components: managing the item list, its identity
/// comparison, and rendering virtualization from outside the component has
/// no established Dioxus-idiomatic equivalent yet -- adico's combobox/select
/// take `children`/registered options instead of a `items` + `itemToString*`
/// data-driven contract.
const COLLECTION_MANAGEMENT_REASON: &str = "adico extension gap: Base UI's data-driven collection-management/virtualization API (item list, identity comparison, virtualized rendering) has no established Dioxus-idiomatic equivalent yet; adico's combobox/select are children/registration-driven instead";

/// Reason recorded where adico exposes a Base UI boolean/behavioral toggle
/// as a distinct component instead of a prop (e.g. `multiple` on
/// `Combobox`/`Select` becomes `ComboboxMulti`/`SelectMulti`).
const SEPARATE_COMPONENT_REASON: &str = "adico extension gap: exposed as a separate component (not a prop toggle on this one) to keep each component's controlled-value type concrete rather than a runtime-branching union";

/// Reason recorded where an upstream root-level prop genuinely lives on one
/// of adico's own sub-parts instead, because adico decomposes the
/// composite differently than the matched axis does (e.g. `placeholder`
/// lives on `ComboboxInput`, not `Combobox` itself).
const PART_DECOMPOSITION_REASON: &str = "adico extension gap: adico exposes this on a different sub-part than the matched axis's own decomposition, not missing from the composite as a whole";

/// Reason recorded where a plain global HTML attribute (no dedicated
/// behavior of its own) is already settable through this part's generic
/// `attributes` extends field -- which spreads last, so a caller-supplied
/// value for it overrides adico's own internally generated one -- and so
/// does not need a dedicated named prop.
const ATTRIBUTES_COVERAGE_REASON: &str = "adico extension gap: settable via this part's generic `attributes` extends forwarding (spread last, so a caller-supplied value overrides adico's own internal one); no dedicated named prop for a plain global attribute with no additional behavior";

/// Reason recorded for D6's "named as an explicit follow-up block" terminal
/// state (design.md), used instead of a new `Status` variant: Change A's
/// already-approved `adico-prop-parity` delta spec fixes the status enum at
/// exactly `present | missing | intentional_difference | adico_extension`,
/// so a genuine primitive-behavior gap that can't land in this change is
/// still recorded as `intentional_difference`, with the reason text itself
/// naming it a deferred follow-up rather than a permanent design choice.
/// Native browser form participation (`FormData` inclusion, constraint
/// validation) for a non-`<input>`-rooted composite requires a hidden
/// mirror `<input>` synced to the controlled value -- a new
/// `adico-primitives` mechanism, not a prop rename, and out of scope for
/// this change's prop-surface work.
const FORM_PARTICIPATION_REASON: &str = "adico extension gap, deferred (not a permanent design choice): native form participation (FormData inclusion, constraint validation) needs a hidden mirror <input> synced to the controlled value, a new adico-primitives mechanism this change's prop-surface scope does not cover; tracked as follow-up primitive work";

/// Reason recorded where adico's disabled state is set once on a
/// composite's root and cascades to every descendant, rather than being
/// independently overridable on one sub-part the way the matched axis
/// allows.
const CASCADING_DISABLED_REASON: &str = "adico extension gap: disabled is set once on the composite's root and cascades to every descendant; independently overriding it on just this sub-part has no adico equivalent and isn't planned, since a consistent whole-component disable is the far more common need";

/// Reason recorded where the matched axis expects app-level locale/number
/// formatting integration (a `locale`/`format` pair, typically feeding
/// `Intl.NumberFormat` or similar) that this codebase has no equivalent
/// infrastructure for anywhere yet -- not specific to one component.
const LOCALE_FORMATTING_REASON: &str = "adico extension gap: no locale-aware number-formatting infrastructure exists anywhere in this codebase yet; not specific to this component, and not planned as part of this change";

/// Reason recorded for a Base UI slider-thumb positioning/stepping
/// refinement (collision avoidance between multiple thumbs, alignment
/// against the track bounds, a larger keyboard step, a minimum spacing
/// constraint between values) that adico's slider primitive doesn't
/// implement yet. Deferred, not permanent: a real primitive-behavior
/// follow-up, same terminal state as `FORM_PARTICIPATION_REASON`.
const ADVANCED_THUMB_BEHAVIOR_REASON: &str = "adico extension gap, deferred (not a permanent design choice): this is an advanced Base UI thumb-positioning/stepping refinement adico's slider primitive doesn't implement yet; tracked as follow-up primitive work, not covered by this change's prop-surface scope";

/// Reason recorded where adico only fires a continuous value-change
/// callback and has no separate "interaction settled" event distinct from
/// every intermediate value during a drag/keyboard adjustment. Deferred:
/// needs new primitive-level pointer-up/keyup-driven event plumbing, not a
/// prop rename.
const VALUE_COMMIT_REASON: &str = "adico extension gap, deferred (not a permanent design choice): adico only exposes a continuous value-change callback; a separate event that fires once only when an interaction settles (not on every intermediate value) needs new primitive-level pointer-up/keyup event plumbing, tracked as follow-up primitive work";

/// Reason recorded where adico represents an axis/direction as a plain
/// boolean flag (e.g. `horizontal`) rather than the matched axis's
/// `"horizontal" | "vertical"` string enum. Same capability, different,
/// permanent representation.
const ORIENTATION_REPRESENTATION_REASON: &str = "adico extension gap: this is exposed as a plain boolean flag (e.g. `horizontal`) rather than a `\"horizontal\" | \"vertical\"` string enum -- same capability, a different representation, not planned to change";

/// Reason recorded for a Base UI slider-thumb ARIA-customization hook
/// (a static/derived accessible label, or a caller-supplied accessible
/// value-text formatter) that adico's `SliderThumb` doesn't expose.
/// Deferred: real accessibility-surface work, not a prop rename.
const THUMB_ARIA_CUSTOMIZATION_REASON: &str = "adico extension gap, deferred (not a permanent design choice): adico's per-thumb ARIA customization (accessible label/value-text derivation) isn't exposed as an overridable hook yet; tracked as follow-up accessibility work, not covered by this change's prop-surface scope";

/// Reason recorded where a thumb-level focus/keyboard/tab-order override
/// hook (`onFocus`/`onBlur`/`onKeyDown`/`tabIndex`) has no adico
/// equivalent, because the primitive already owns that thumb's focus and
/// keyboard-driven value changes internally -- exposing raw overrides
/// risks conflicting with that internal behavior rather than composing
/// with it. Deferred: would need the primitive's own event composition to
/// change first, not a prop rename.
const THUMB_EVENT_OVERRIDE_REASON: &str = "adico extension gap, deferred (not a permanent design choice): this thumb's focus/keyboard/tab-order behavior is already owned internally by the primitive (drag/keyboard value changes); exposing it as a raw overridable hook needs the primitive's own event composition to change first, tracked as follow-up primitive work";

/// Reason recorded where a composite's popup/overlay layer has no
/// modal-vs-non-modal focus-trap toggle -- its focus-containment behavior
/// is fixed, not planned to become configurable.
const MODAL_POPUP_REASON: &str = "adico extension gap: this composite's popup/overlay layer has no modal-vs-non-modal focus-trap toggle; the popup's focus-containment behavior is fixed, not planned to become configurable";

/// Reason recorded where the matched axis lets a caller supply a custom
/// renderer/formatter for the currently selected value (a function or
/// node overriding the default text display), which adico doesn't expose.
/// Deferred: needs the primitive to thread the typed selected value out to
/// a caller-supplied render callback, real primitive-level work, not a
/// prop rename.
const CUSTOM_VALUE_RENDER_REASON: &str = "adico extension gap, deferred (not a permanent design choice): adico's value display always renders the selected option's own text; letting a caller supply a custom renderer needs the primitive to thread the typed selected value out to a caller-supplied render callback, tracked as follow-up primitive work";

/// Reason recorded where the matched axis fires a callback once an
/// open/close animation finishes, distinct from the moment the open state
/// itself changes -- adico has no such completion signal for this
/// composite yet. Deferred: needs new primitive-level transition-end
/// event plumbing, not a prop rename.
const ANIMATION_COMPLETE_REASON: &str = "adico extension gap, deferred (not a permanent design choice): adico has no callback for when an open/close animation actually finishes, distinct from when the open state itself changes; needs new primitive-level transition-end event plumbing, tracked as follow-up primitive work";

/// Reason recorded where a composite supports only one fixed layout
/// orientation (no caller-selectable horizontal/vertical toggle at all,
/// unlike `ORIENTATION_REPRESENTATION_REASON`'s case of "has it, just a
/// different representation").
const FIXED_ORIENTATION_REASON: &str = "adico extension gap, deferred (not a permanent design choice): this composite only supports one fixed layout orientation; a caller-selectable horizontal/vertical toggle needs new layout work, tracked as follow-up";

/// Reason recorded where the matched axis's controlled-value system (a
/// stable per-item identity plus a root-level `value`/`onValueChange`
/// pair tracking which item is currently open/active) has no adico
/// equivalent -- adico tracks the open item by internal index only, with
/// no caller-controllable API. Deferred: a real controlled-value feature,
/// not a prop rename.
const NO_CONTROLLED_OPEN_ITEM_REASON: &str = "adico extension gap, deferred (not a permanent design choice): adico tracks which item is open internally (by index) with no caller-controllable value/onValueChange API or stable per-item value identity; needs new primitive-level controlled-state work, tracked as follow-up";

/// Reason recorded where shadcn's own convenience toggle for a single,
/// shared cross-item animated viewport wrapper has no adico equivalent --
/// each of adico's own content parts positions independently instead.
const SHARED_VIEWPORT_REASON: &str = "adico extension gap: shadcn's own convenience toggle for a single, shared cross-item animated viewport wrapper has no adico equivalent; each content part positions independently instead, a compositional difference not planned to change";

/// Item-specific `intentional_difference` reasons for a genuine upstream
/// prop that has no adico equivalent by design, keyed by
/// `(item, part, upstream_prop_name)` using the upstream axis's own raw
/// spelling (matched before normalization, same convention as
/// `REACT_ONLY_STRUCTURAL`). Distinct from `ADICO_EXTENSION_REASONS`, which
/// documents the opposite direction -- an adico field with no upstream
/// counterpart.
const INTENTIONAL_DIFFERENCE_REASONS: &[(&str, &str, &str, &str)] = &[
    // combobox: Base UI's data-driven collection-management/virtualization API.
    ("combobox", "root", "items", COLLECTION_MANAGEMENT_REASON),
    (
        "combobox",
        "root",
        "filteredItems",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "combobox",
        "root",
        "isItemEqualToValue",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "combobox",
        "root",
        "itemToStringLabel",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "combobox",
        "root",
        "itemToStringValue",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "combobox",
        "root",
        "virtualized",
        COLLECTION_MANAGEMENT_REASON,
    ),
    ("combobox", "root", "grid", COLLECTION_MANAGEMENT_REASON),
    ("combobox", "root", "inline", COLLECTION_MANAGEMENT_REASON),
    ("combobox", "root", "limit", COLLECTION_MANAGEMENT_REASON),
    (
        "combobox",
        "root",
        "autoHighlight",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "combobox",
        "root",
        "highlightItemOnHover",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "combobox",
        "root",
        "onItemHighlighted",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "combobox",
        "root",
        "openOnInputClick",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "combobox",
        "root",
        "onOpenChangeComplete",
        ANIMATION_COMPLETE_REASON,
    ),
    (
        "combobox",
        "root",
        "autoComplete",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "combobox",
        "root",
        "defaultInputValue",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "combobox",
        "root",
        "inputValue",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "combobox",
        "root",
        "onInputValueChange",
        COLLECTION_MANAGEMENT_REASON,
    ),
    ("combobox", "root", "locale", COLLECTION_MANAGEMENT_REASON),
    (
        "combobox",
        "root",
        "loopFocus",
        COLLECTION_MANAGEMENT_REASON,
    ),
    ("combobox", "root", "modal", MODAL_POPUP_REASON),
    ("combobox", "root", "multiple", SEPARATE_COMPONENT_REASON),
    ("combobox", "root", "placeholder", PART_DECOMPOSITION_REASON),
    ("combobox", "root", "aria_label", PART_DECOMPOSITION_REASON),
    (
        "combobox",
        "root",
        "list_aria_label",
        PART_DECOMPOSITION_REASON,
    ),
    ("combobox", "root", "id", ATTRIBUTES_COVERAGE_REASON),
    ("combobox", "root", "name", FORM_PARTICIPATION_REASON),
    ("combobox", "root", "form", FORM_PARTICIPATION_REASON),
    ("combobox", "root", "required", FORM_PARTICIPATION_REASON),
    ("combobox", "root", "readOnly", FORM_PARTICIPATION_REASON),
    ("combobox", "input", "disabled", CASCADING_DISABLED_REASON),
    // slider: native form-integration (deferred), advanced Base UI
    // thumb/stepping behavior adico's primitive doesn't implement yet
    // (deferred), locale/format (no infra anywhere, permanent),
    // orientation-as-boolean (permanent), and per-thumb ARIA/event hooks
    // (deferred). `root` appears on both `base-ui` and `shadcn` (same
    // matched prop list), so each reason is entered for both axes.
    ("slider", "root", "name", FORM_PARTICIPATION_REASON),
    ("slider", "root", "form", FORM_PARTICIPATION_REASON),
    ("slider", "root", "onValueCommitted", VALUE_COMMIT_REASON),
    ("slider", "root", "locale", LOCALE_FORMATTING_REASON),
    ("slider", "root", "format", LOCALE_FORMATTING_REASON),
    (
        "slider",
        "root",
        "thumbAlignment",
        ADVANCED_THUMB_BEHAVIOR_REASON,
    ),
    (
        "slider",
        "root",
        "thumbCollisionBehavior",
        ADVANCED_THUMB_BEHAVIOR_REASON,
    ),
    (
        "slider",
        "root",
        "largeStep",
        ADVANCED_THUMB_BEHAVIOR_REASON,
    ),
    (
        "slider",
        "root",
        "minStepsBetweenValues",
        ADVANCED_THUMB_BEHAVIOR_REASON,
    ),
    (
        "slider",
        "root",
        "orientation",
        ORIENTATION_REPRESENTATION_REASON,
    ),
    (
        "slider",
        "thumb",
        "aria-valuetext",
        THUMB_ARIA_CUSTOMIZATION_REASON,
    ),
    (
        "slider",
        "thumb",
        "getAriaLabel",
        THUMB_ARIA_CUSTOMIZATION_REASON,
    ),
    (
        "slider",
        "thumb",
        "getAriaValueText",
        THUMB_ARIA_CUSTOMIZATION_REASON,
    ),
    ("slider", "thumb", "onBlur", THUMB_EVENT_OVERRIDE_REASON),
    ("slider", "thumb", "onFocus", THUMB_EVENT_OVERRIDE_REASON),
    ("slider", "thumb", "onKeyDown", THUMB_EVENT_OVERRIDE_REASON),
    ("slider", "thumb", "tabIndex", THUMB_EVENT_OVERRIDE_REASON),
    ("slider", "thumb", "disabled", CASCADING_DISABLED_REASON),
    // select: the same Base UI collection-management API as combobox,
    // native form-integration (deferred), a modal-popup toggle (permanent,
    // reused from combobox), `multiple` as a separate `SelectMulti`
    // component, and a custom value-renderer (deferred).
    (
        "select",
        "root",
        "highlightItemOnHover",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "select",
        "root",
        "autoComplete",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "select",
        "root",
        "isItemEqualToValue",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "select",
        "root",
        "itemToStringLabel",
        COLLECTION_MANAGEMENT_REASON,
    ),
    (
        "select",
        "root",
        "itemToStringValue",
        COLLECTION_MANAGEMENT_REASON,
    ),
    ("select", "root", "items", COLLECTION_MANAGEMENT_REASON),
    (
        "select",
        "root",
        "onOpenChangeComplete",
        ANIMATION_COMPLETE_REASON,
    ),
    ("select", "root", "form", FORM_PARTICIPATION_REASON),
    ("select", "root", "readOnly", FORM_PARTICIPATION_REASON),
    ("select", "root", "required", FORM_PARTICIPATION_REASON),
    ("select", "root", "id", ATTRIBUTES_COVERAGE_REASON),
    ("select", "root", "modal", MODAL_POPUP_REASON),
    ("select", "root", "multiple", SEPARATE_COMPONENT_REASON),
    ("select", "trigger", "disabled", CASCADING_DISABLED_REASON),
    ("select", "value", "children", CUSTOM_VALUE_RENDER_REASON),
    // navigation-menu: `delay`/`closeDelay`/`keepMounted` are real,
    // renamed fields (see `ITEM_RENAMES`) and so never reach this table;
    // the remaining gaps are a real controlled-open-item API adico
    // doesn't have, an animation-complete callback, and a fixed single
    // orientation.
    (
        "navigation-menu",
        "root",
        "defaultValue",
        NO_CONTROLLED_OPEN_ITEM_REASON,
    ),
    (
        "navigation-menu",
        "root",
        "value",
        NO_CONTROLLED_OPEN_ITEM_REASON,
    ),
    (
        "navigation-menu",
        "root",
        "onValueChange",
        NO_CONTROLLED_OPEN_ITEM_REASON,
    ),
    (
        "navigation-menu",
        "item",
        "value",
        NO_CONTROLLED_OPEN_ITEM_REASON,
    ),
    (
        "navigation-menu",
        "root",
        "onOpenChangeComplete",
        ANIMATION_COMPLETE_REASON,
    ),
    (
        "navigation-menu",
        "root",
        "orientation",
        FIXED_ORIENTATION_REASON,
    ),
    (
        "navigation-menu",
        "root",
        "viewport",
        SHARED_VIEWPORT_REASON,
    ),
];

fn react_only_structural_reason(raw_name: &str) -> Option<&'static str> {
    REACT_ONLY_STRUCTURAL
        .iter()
        .find(|(name, _)| *name == raw_name)
        .map(|(_, reason)| *reason)
}

fn camel_to_snake(raw_name: &str) -> String {
    let mut result = String::new();
    for (index, ch) in raw_name.chars().enumerate() {
        if ch.is_uppercase() {
            if index > 0 {
                result.push('_');
            }
            result.extend(ch.to_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

/// The name adico's own snake_case fields are compared against: an
/// explicit rename if one exists, otherwise a generic camelCase ->
/// snake_case conversion.
fn canonical_name(raw_name: &str) -> String {
    for (from, to) in RENAMES {
        if *from == raw_name {
            return (*to).to_string();
        }
    }
    camel_to_snake(raw_name)
}

fn adico_extension_reason(
    table: &[(&'static str, &'static str, &'static str, &'static str)],
    item: &str,
    part: &str,
    prop: &str,
) -> Option<&'static str> {
    table
        .iter()
        .find(|(table_item, table_part, table_prop, _)| {
            *table_item == item && *table_part == part && *table_prop == prop
        })
        .map(|(_, _, _, reason)| *reason)
}

/// Whether `raw_name` is shaped like a native DOM/Dioxus event-handler prop
/// name (`onclick`, `onchange`, `onmounted`, ...): all-lowercase, no
/// separating underscore -- Dioxus's own convention for a *semantic*
/// callback prop is always `on_snake_case` (`on_checked_change`), so this
/// pattern only ever matches a genuine native-element event, never one of
/// adico's own props.
fn looks_like_native_event_name(raw_name: &str) -> bool {
    raw_name.len() > 2
        && raw_name.starts_with("on")
        && raw_name.chars().all(|ch| ch.is_ascii_lowercase())
}

fn classify_upstream_prop(
    item: &str,
    part: &str,
    raw_name: &str,
    adico_fields: &AdicoFields,
) -> PropStatus {
    if let Some(reason) = react_only_structural_reason(raw_name) {
        return PropStatus {
            name: raw_name.to_string(),
            status: Status::IntentionalDifference,
            reason: Some(reason.to_string()),
        };
    }
    // A native event-handler name (`onchange`, `onmounted`, ...) is present
    // if adico forwards it generically via `#[props(extends = ...)]`'s
    // established `attributes: Vec<Attribute>` field, even though no field
    // is individually named after it -- otherwise every native event this
    // axis enumerates by name shows up as a false-positive `missing`
    // finding for the ~22 of 66 registry items using this convention.
    let covered_by_attributes_extend =
        looks_like_native_event_name(raw_name) && adico_fields.has_attributes_extend;
    if adico_fields
        .names
        .contains(&item_canonical_name(item, raw_name))
        || covered_by_attributes_extend
    {
        return PropStatus {
            name: raw_name.to_string(),
            status: Status::Present,
            reason: None,
        };
    }
    if let Some(reason) =
        adico_extension_reason(INTENTIONAL_DIFFERENCE_REASONS, item, part, raw_name)
    {
        return PropStatus {
            name: raw_name.to_string(),
            status: Status::IntentionalDifference,
            reason: Some(reason.to_string()),
        };
    }
    PropStatus {
        name: raw_name.to_string(),
        status: Status::Missing,
        reason: None,
    }
}

// --- Resolving an upstream part's props ---------------------------------

#[derive(Debug)]
enum ResolvedPartProps {
    Resolved(Vec<catalog::schema::Prop>),
    Unresolved(String),
}

/// Resolves one catalog part's `props_source`. An `inherits_from` reference
/// is resolved against the axis it actually names (`base-ui` for a `radix`
/// reference, the real target axis's own snapshot otherwise -- e.g.
/// `dioxus-components`' `inherits_from: "dioxus-primitives.<module>.<part>"`
/// resolves against the `dioxus-primitives` snapshot, not `base-ui`).
/// `resolve_inherits_from`'s own axis handling (radix aliasing, third-party
/// and self-reference reasons) applies unchanged; this only decides which
/// snapshot to hand it.
fn resolve_props_source(
    props_source: &PropsSource,
    snapshots: &BTreeMap<String, CatalogSnapshot>,
    base_ui: &CatalogSnapshot,
) -> ResolvedPartProps {
    match props_source {
        PropsSource::Explicit { props } => ResolvedPartProps::Resolved(props.clone()),
        PropsSource::Unavailable => {
            ResolvedPartProps::Unresolved("no prop data available for this entry".to_string())
        }
        PropsSource::InheritsFrom { reference } => {
            let axis = reference.split('.').next().unwrap_or("");
            let target = match axis {
                "radix" | "vaul" | "cmdk" => base_ui,
                other
                    if other.starts_with("@shadcn/react/") || other.starts_with("@/registry/") =>
                {
                    base_ui
                }
                other => snapshots.get(other).unwrap_or(base_ui),
            };
            match catalog::resolve_inherits_from(reference, target) {
                ResolvedProps::Resolved { props } => ResolvedPartProps::Resolved(props),
                ResolvedProps::Unresolved { reason } => ResolvedPartProps::Unresolved(reason),
            }
        }
    }
}

/// A shadcn part frequently carries **both** local `explicit` augmentation
/// props (e.g. `showCloseButton`) and a `composition` reference to the
/// upstream primitive it also wraps (`schema.rs`'s "Upstream composition
/// and adico's own composition are tracked separately" design: `composition`
/// and `props_source` are independent fields, so a part can be
/// `props_source: explicit` while `composition` still names what it wraps)
/// -- 25 of shadcn's 61 real components have this shape (`Dialog.Content`,
/// `Sheet.Content`, every Radix `*.Item`'s `inset`/`variant` augmentation,
/// etc.), not a rare edge case. Comparing only the local augmentation props
/// against adico would silently miss every prop the wrapped primitive
/// itself declares. This resolves both and merges them, local names taking
/// precedence over same-named inherited ones.
fn resolve_full_part_props(
    part: &catalog::schema::PartEntry,
    snapshots: &BTreeMap<String, CatalogSnapshot>,
    base_ui: &CatalogSnapshot,
) -> ResolvedPartProps {
    let PropsSource::Explicit { props } = &part.props_source else {
        return resolve_props_source(&part.props_source, snapshots, base_ui);
    };
    let Some(composition_ref) = part.composition.first() else {
        return ResolvedPartProps::Resolved(props.clone());
    };

    let reference = format!(
        "{}.{}.{}",
        composition_ref.axis,
        composition_ref.component,
        composition_ref.part.as_deref().unwrap_or("root")
    );
    let mut combined = props.clone();
    if let ResolvedPartProps::Resolved(inherited) =
        resolve_props_source(&PropsSource::InheritsFrom { reference }, snapshots, base_ui)
    {
        let local_names: BTreeSet<String> = combined.iter().map(|prop| prop.name.clone()).collect();
        for prop in inherited {
            if !local_names.contains(&prop.name) {
                combined.push(prop);
            }
        }
    }
    ResolvedPartProps::Resolved(combined)
}

// --- Item introspection ---------------------------------------------------

fn introspect_item(root: &Path, item: &RegistryItem) -> FileIntrospection {
    let mut merged = FileIntrospection {
        exists: false,
        ..Default::default()
    };
    for file in &item.files {
        let introspection =
            rust_introspect::introspect_file(&registry_root(root).join(&file.source));
        merged.exists |= introspection.exists;
        merged.components.extend(introspection.components);
        merged.props.extend(introspection.props);
    }
    // A registry facade that wholesale-reuses a primitive's own Props
    // struct (`pub fn Foo(props: FooProps)` plus `pub use
    // adico_primitives::<module>::FooProps`, e.g. `Slider`/`RangeSlider`)
    // has no local `FooProps` definition for `rust_introspect` to find --
    // `inline_component_props` deliberately returns `None` for the
    // `props: FooProps` shape, deferring to an `Item::Struct` match that
    // only ever fires for a struct defined *in this same file*. Fall back
    // to the primitive module's own struct definitions, but only to fill
    // in a name this item's own registry source didn't already cover --
    // never override a registry-owned entry (a locally defined `FooProps`,
    // or an inline-args component recorded under the bare `Foo` key),
    // which may legitimately narrow or widen the primitive's own surface.
    for module in find_primitive_modules(root, item) {
        let primitive_introspection = introspect_primitive_module(root, &module);
        for (struct_name, fields) in primitive_introspection.props {
            let component_key = struct_name.strip_suffix("Props").unwrap_or(&struct_name);
            if merged.props.contains_key(&struct_name) || merged.props.contains_key(component_key) {
                continue;
            }
            merged.props.insert(struct_name, fields);
        }
    }
    merged
}

fn introspect_primitive_module(root: &Path, module: &str) -> FileIntrospection {
    let src = root.join("packages/adico-primitives/src");
    let file_path = src.join(format!("{module}.rs"));
    if file_path.is_file() {
        return rust_introspect::introspect_file(&file_path);
    }
    let dir_path = src.join(module);
    if dir_path.is_dir() {
        return rust_introspect::introspect_directory(&dir_path);
    }
    FileIntrospection::default()
}

/// A component's own declared prop field names, plus whether one of them is
/// the established `#[props(extends = GlobalAttributes)]` convention's
/// `attributes: Vec<Attribute>` field -- `rust_introspect.rs` doesn't read
/// `#[props(extends = ...)]` itself (it's a struct-field attribute, not a
/// value carried on the field), but the field it's attached to is still
/// captured by name and type like any other, which is enough to detect the
/// convention without teaching `rust_introspect.rs` a new attribute shape.
struct AdicoFields {
    names: BTreeSet<String>,
    has_attributes_extend: bool,
}

fn adico_field_names(introspection: &FileIntrospection, component: &str) -> AdicoFields {
    let fields = introspection
        .props
        .get(&format!("{component}Props"))
        .or_else(|| introspection.props.get(component));
    let names = fields
        .map(|fields| fields.iter().map(|field| field.name.clone()).collect())
        .unwrap_or_default();
    let has_attributes_extend = fields.is_some_and(|fields| {
        fields
            .iter()
            .any(|field| field.name == "attributes" && field.type_name.contains("Attribute"))
    });
    AdicoFields {
        names,
        has_attributes_extend,
    }
}

// --- Building one item's record --------------------------------------------

/// Bundles the two pieces of context every part/axis lookup needs (the full
/// set of loaded axis snapshots, for resolving an `inherits_from`
/// reference's own target axis, and the `base-ui` snapshot specifically,
/// for radix alias resolution) so `build_part_parity`/`build_axis_parity`
/// don't have to thread them as separate parameters.
struct ResolutionContext<'a> {
    snapshots: &'a BTreeMap<String, CatalogSnapshot>,
    base_ui: &'a CatalogSnapshot,
}

fn build_part_parity(
    part_id: String,
    component: &str,
    entry: &catalog::schema::CatalogEntry,
    axis_id: &str,
    item_name: &str,
    introspection: &FileIntrospection,
    context: &ResolutionContext<'_>,
) -> PartParity {
    let Some(part) = entry.parts.iter().find(|candidate| candidate.id == part_id) else {
        return PartParity {
            id: part_id.clone(),
            props: Vec::new(),
            unresolved: Some(format!(
                "no matching `{part_id}` part on axis `{axis_id}`'s `{}` entry",
                entry.id
            )),
        };
    };

    match resolve_full_part_props(part, context.snapshots, context.base_ui) {
        ResolvedPartProps::Unresolved(reason) => PartParity {
            id: part_id,
            props: Vec::new(),
            unresolved: Some(reason),
        },
        ResolvedPartProps::Resolved(upstream_props) => {
            let adico_fields = adico_field_names(introspection, component);
            let mut matched_canonical: BTreeSet<String> = BTreeSet::new();
            let mut props: Vec<PropStatus> = upstream_props
                .iter()
                .map(|prop| {
                    if react_only_structural_reason(&prop.name).is_none() {
                        matched_canonical.insert(canonical_name(&prop.name));
                    }
                    classify_upstream_prop(item_name, &part_id, &prop.name, &adico_fields)
                })
                .collect();

            for field in &adico_fields.names {
                if matched_canonical.contains(field) {
                    continue;
                }
                if let Some(reason) =
                    adico_extension_reason(ADICO_EXTENSION_REASONS, item_name, &part_id, field)
                {
                    props.push(PropStatus {
                        name: field.clone(),
                        status: Status::AdicoExtension,
                        reason: Some(reason.to_string()),
                    });
                }
            }

            PartParity {
                id: part_id,
                props,
                unresolved: None,
            }
        }
    }
}

fn build_axis_parity(
    item: &RegistryItem,
    introspection: &FileIntrospection,
    axis_id: &str,
    snapshot: &CatalogSnapshot,
    context: &ResolutionContext<'_>,
) -> AxisParity {
    let Some(entry) = snapshot.entries.iter().find(|entry| entry.id == item.name) else {
        return AxisParity {
            matched_component: None,
            parts: Vec::new(),
        };
    };

    let mut seen_part_ids = BTreeSet::new();
    let parts = introspection
        .components
        .iter()
        .filter_map(|component| {
            let part_id = catalog::part_id_for(&item.name, component);
            if !seen_part_ids.insert(part_id.clone()) {
                return None;
            }
            Some(build_part_parity(
                part_id,
                component,
                entry,
                axis_id,
                &item.name,
                introspection,
                context,
            ))
        })
        .collect();

    AxisParity {
        matched_component: Some(entry.id.clone()),
        parts,
    }
}

fn load_axis_snapshots(root: &Path) -> Result<BTreeMap<String, CatalogSnapshot>, String> {
    catalog::AXES
        .iter()
        .map(|axis| {
            catalog::read_snapshot(root, axis.id).map(|snapshot| (axis.id.to_string(), snapshot))
        })
        .collect()
}

fn build_record(
    root: &Path,
    item: &RegistryItem,
    snapshots: &BTreeMap<String, CatalogSnapshot>,
) -> Result<PropParityRecord, String> {
    let base_ui = snapshots
        .get("base-ui")
        .ok_or_else(|| "no base-ui snapshot loaded".to_string())?
        .clone();
    let context = ResolutionContext {
        snapshots,
        base_ui: &base_ui,
    };
    let introspection = introspect_item(root, item);

    let axes = catalog::AXES
        .iter()
        .map(|axis| {
            let snapshot = snapshots
                .get(axis.id)
                .ok_or_else(|| format!("no `{}` snapshot loaded", axis.id))?;
            Ok((
                axis.id.to_string(),
                build_axis_parity(item, &introspection, axis.id, snapshot, &context),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;

    Ok(PropParityRecord {
        adico_item: item.name.clone(),
        axes,
    })
}

fn serialize_record(record: &PropParityRecord) -> Result<String, String> {
    let payload = serde_json::to_string_pretty(record)
        .map_err(|error| format!("cannot serialize record for {}: {error}", record.adico_item))?;
    Ok(format!("{payload}\n"))
}

// --- sync / check / diff ----------------------------------------------------

pub fn sync(root: &Path) -> Result<(), String> {
    let dir = records_dir(root);
    fs::create_dir_all(&dir)
        .map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    let items = load_registry_items(root)?;
    let snapshots = load_axis_snapshots(root)?;
    let mut written = 0usize;
    for item in &items {
        let record = build_record(root, item, &snapshots)?;
        let payload = serialize_record(&record)?;
        write_if_changed(&record_path(root, &item.name), &payload)?;
        written += 1;
    }
    println!("Synced {written} statics/prop_parity/<item>.json record(s).");
    Ok(())
}

pub fn check(root: &Path) -> Result<(), String> {
    let items = load_registry_items(root)?;
    let item_names: BTreeSet<String> = items.iter().map(|item| item.name.clone()).collect();
    let dir = records_dir(root);

    let mut violations = Vec::new();

    if dir.is_dir() {
        for entry in
            fs::read_dir(&dir).map_err(|error| format!("cannot read {}: {error}", dir.display()))?
        {
            let entry = entry.map_err(|error| format!("cannot read record entry: {error}"))?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default();
            if !item_names.contains(stem) {
                violations.push(format!(
                    "{}: record has no matching registry.json item",
                    path.display()
                ));
            }
        }
    }

    let snapshots = load_axis_snapshots(root)?;
    for item in &items {
        let record = build_record(root, item, &snapshots)?;
        let expected = serialize_record(&record)?;
        let path = record_path(root, &item.name);
        let actual = fs::read_to_string(&path).unwrap_or_default();
        if actual != expected {
            violations.push(format!(
                "{}: statics/prop_parity/{}.json is stale; run `cargo xtask prop-parity sync`",
                item.name, item.name
            ));
        }
    }

    if violations.is_empty() {
        println!("prop-parity check passed: {} item(s).", items.len());
        Ok(())
    } else {
        Err(violations.join("\n"))
    }
}

pub fn diff(root: &Path) -> Result<(), String> {
    let items = load_registry_items(root)?;
    let snapshots = load_axis_snapshots(root)?;
    let mut changed = Vec::new();
    for item in &items {
        let record = build_record(root, item, &snapshots)?;
        let expected = serialize_record(&record)?;
        let path = record_path(root, &item.name);
        let actual = fs::read_to_string(&path).unwrap_or_default();
        if actual != expected {
            changed.push(item.name.clone());
        }
    }
    if changed.is_empty() {
        println!(
            "No drift: every statics/prop_parity/<item>.json record matches what `sync` would generate."
        );
    } else {
        for name in &changed {
            println!("{name}: would change");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::schema::{CatalogEntry, PartEntry, Prop};
    use std::collections::BTreeMap as StdBTreeMap;

    fn field_names(names: &[&str]) -> AdicoFields {
        AdicoFields {
            names: names.iter().map(|name| name.to_string()).collect(),
            has_attributes_extend: false,
        }
    }

    #[test]
    fn casing_only_difference_is_present_not_missing() {
        let fields = field_names(&["on_open_change"]);
        let status = classify_upstream_prop("test-item", "test-part", "onOpenChange", &fields);
        assert_eq!(status.status, Status::Present);
    }

    #[test]
    fn class_name_rename_is_present() {
        let fields = field_names(&["class"]);
        let status = classify_upstream_prop("test-item", "test-part", "className", &fields);
        assert_eq!(status.status, Status::Present);
    }

    #[test]
    fn render_prop_is_intentional_difference_with_fixed_reason() {
        let fields = field_names(&[]);
        let status = classify_upstream_prop("test-item", "test-part", "render", &fields);
        assert_eq!(status.status, Status::IntentionalDifference);
        assert!(status.reason.is_some());
    }

    #[test]
    fn genuine_gap_is_missing() {
        let fields = field_names(&["checked"]);
        let status = classify_upstream_prop("test-item", "test-part", "readOnly", &fields);
        assert_eq!(status.status, Status::Missing);
    }

    #[test]
    fn native_event_name_is_missing_without_attributes_extend() {
        let fields = field_names(&["checked"]);
        let status = classify_upstream_prop("test-item", "test-part", "onchange", &fields);
        assert_eq!(status.status, Status::Missing);
    }

    #[test]
    fn native_event_name_is_present_via_attributes_extend() {
        let mut fields = field_names(&["checked"]);
        fields.has_attributes_extend = true;
        let status = classify_upstream_prop("test-item", "test-part", "onchange", &fields);
        assert_eq!(status.status, Status::Present);
    }

    #[test]
    fn adico_own_snake_case_callback_is_not_mistaken_for_a_native_event() {
        // `on_checked_change` is adico's own semantic-callback convention,
        // not a native DOM event -- `looks_like_native_event_name` must not
        // match it (it contains an underscore), so a genuine miss on this
        // shape still reports `missing` even with `attributes` present.
        let mut fields = field_names(&["checked"]);
        fields.has_attributes_extend = true;
        let status = classify_upstream_prop("test-item", "test-part", "on_checked_change", &fields);
        assert_eq!(status.status, Status::Missing);
    }

    #[test]
    fn adico_extension_prop_gets_reason_from_item_keyed_table() {
        const TEST_TABLE: &[(&str, &str, &str, &str)] = &[(
            "button",
            "root",
            "radius",
            "adico extension: consistent visual rounding, no upstream counterpart",
        )];
        let reason = adico_extension_reason(TEST_TABLE, "button", "root", "radius");
        assert!(reason.is_some());
        assert_eq!(
            adico_extension_reason(TEST_TABLE, "button", "root", "other"),
            None
        );
    }

    fn base_ui_fixture() -> CatalogSnapshot {
        CatalogSnapshot {
            axis: "base-ui".to_string(),
            source: "https://base-ui.com".to_string(),
            revision: "test".to_string(),
            refreshed_at: "2026-09-05".to_string(),
            entries: vec![CatalogEntry {
                id: "switch".to_string(),
                name: "Switch".to_string(),
                parts: vec![PartEntry {
                    id: "root".to_string(),
                    composition: Vec::new(),
                    props_source: PropsSource::Explicit {
                        props: vec![
                            Prop {
                                name: "checked".to_string(),
                                type_name: "boolean".to_string(),
                                default: None,
                                description: None,
                            },
                            Prop {
                                name: "readOnly".to_string(),
                                type_name: "boolean".to_string(),
                                default: None,
                                description: None,
                            },
                            Prop {
                                name: "render".to_string(),
                                type_name: "ReactElement".to_string(),
                                default: None,
                                description: None,
                            },
                        ],
                    },
                }],
            }],
        }
    }

    fn empty_snapshot(axis: &str) -> CatalogSnapshot {
        CatalogSnapshot {
            axis: axis.to_string(),
            source: "https://example.invalid".to_string(),
            revision: "test".to_string(),
            refreshed_at: "2026-09-05".to_string(),
            entries: Vec::new(),
        }
    }

    fn fixture_introspection() -> FileIntrospection {
        use crate::rust_introspect::PropField;
        let mut introspection = FileIntrospection {
            exists: true,
            ..Default::default()
        };
        introspection.components.push("Switch".to_string());
        introspection.props.insert(
            "SwitchProps".to_string(),
            vec![PropField {
                name: "checked".to_string(),
                type_name: "ReadSignal<Option<bool>>".to_string(),
                default: None,
            }],
        );
        introspection
    }

    fn fixture_item(name: &str) -> RegistryItem {
        RegistryItem {
            name: name.to_string(),
            item_type: adico_registry_core::RegistryItemType::Ui,
            description: String::new(),
            files: Vec::new(),
            registry_dependencies: Vec::new(),
            cargo_dependencies: Vec::new(),
            style: Default::default(),
            module_exports: Vec::new(),
            documentation: None,
            compatibility: None,
            provenance: None,
        }
    }

    fn fixture_registry_file(source: &str) -> adico_registry_core::RegistryFile {
        adico_registry_core::RegistryFile {
            source: source.to_string(),
            target_root: adico_registry_core::TargetRoot::Ui,
            target: source.to_string(),
            checksum: String::new(),
        }
    }

    #[test]
    fn introspect_item_falls_back_to_the_primitive_module_for_a_wholesale_reused_props_struct() {
        // Mirrors `Slider`/`RangeSlider`/`Toast`: `pub fn Widget(props:
        // WidgetProps)` plus `pub use adico_primitives::widget::WidgetProps`
        // -- no local `WidgetProps` definition for `rust_introspect` to find
        // in the registry file itself.
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("valid time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("adico-prop-parity-test-{nonce}"));
        let registry_ui = root.join("registry/ui");
        let primitive_src = root.join("packages/adico-primitives/src");
        fs::create_dir_all(&registry_ui).expect("registry/ui should be creatable");
        fs::create_dir_all(&primitive_src).expect("primitive src should be creatable");
        fs::write(
            registry_ui.join("widget.rs"),
            "use adico_primitives::widget::WidgetPrimitive;\npub use adico_primitives::widget::WidgetProps;\n\n#[component]\npub fn Widget(props: WidgetProps) -> Element {\n    rsx! { WidgetPrimitive { value: props.value } }\n}\n",
        )
        .expect("registry fixture file should be writable");
        fs::write(
            primitive_src.join("widget.rs"),
            "#[derive(Props, Clone, PartialEq)]\npub struct WidgetProps {\n    pub value: ReadSignal<bool>,\n    #[props(extends = GlobalAttributes)]\n    pub attributes: Vec<Attribute>,\n}\n",
        )
        .expect("primitive fixture file should be writable");

        let mut item = fixture_item("widget");
        item.files = vec![fixture_registry_file("ui/widget.rs")];
        let introspection = introspect_item(&root, &item);

        let fields = adico_field_names(&introspection, "Widget");
        assert!(
            fields.names.contains("value"),
            "expected the primitive's own field, got {:?}",
            fields.names
        );
        assert!(fields.has_attributes_extend);

        fs::remove_dir_all(&root).expect("temporary root should be removable");
    }

    #[test]
    fn introspect_item_never_overrides_a_registry_owned_props_definition() {
        // A registry facade may deliberately narrow or widen the
        // primitive's own field set (e.g. omitting an internal-only
        // field) -- the primitive-module fallback must never clobber
        // that with the primitive's own, different list.
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("valid time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("adico-prop-parity-test-override-{nonce}"));
        let registry_ui = root.join("registry/ui");
        let primitive_src = root.join("packages/adico-primitives/src");
        fs::create_dir_all(&registry_ui).expect("registry/ui should be creatable");
        fs::create_dir_all(&primitive_src).expect("primitive src should be creatable");
        fs::write(
            registry_ui.join("widget.rs"),
            "use adico_primitives::widget::WidgetPrimitive;\n\n#[derive(Props, Clone, PartialEq)]\npub struct WidgetProps {\n    pub registry_only_field: Option<String>,\n}\n\n#[component]\npub fn Widget(props: WidgetProps) -> Element {\n    rsx! { WidgetPrimitive {} }\n}\n",
        )
        .expect("registry fixture file should be writable");
        fs::write(
            primitive_src.join("widget.rs"),
            "#[derive(Props, Clone, PartialEq)]\npub struct WidgetProps {\n    pub primitive_only_field: ReadSignal<bool>,\n}\n",
        )
        .expect("primitive fixture file should be writable");

        let mut item = fixture_item("widget");
        item.files = vec![fixture_registry_file("ui/widget.rs")];
        let introspection = introspect_item(&root, &item);

        let fields = adico_field_names(&introspection, "Widget");
        assert!(fields.names.contains("registry_only_field"));
        assert!(!fields.names.contains("primitive_only_field"));

        fs::remove_dir_all(&root).expect("temporary root should be removable");
    }

    #[test]
    fn matched_axis_produces_present_missing_and_intentional_difference() {
        let base_ui = base_ui_fixture();
        let mut snapshots: StdBTreeMap<String, CatalogSnapshot> = StdBTreeMap::new();
        snapshots.insert("base-ui".to_string(), base_ui.clone());
        let introspection = fixture_introspection();
        let item = fixture_item("switch");

        let context = ResolutionContext {
            snapshots: &snapshots,
            base_ui: &base_ui,
        };
        let axis_parity = build_axis_parity(&item, &introspection, "base-ui", &base_ui, &context);
        assert_eq!(axis_parity.matched_component.as_deref(), Some("switch"));
        let root = axis_parity
            .parts
            .iter()
            .find(|part| part.id == "root")
            .expect("root part");
        let checked = root.props.iter().find(|p| p.name == "checked").unwrap();
        assert_eq!(checked.status, Status::Present);
        let read_only = root.props.iter().find(|p| p.name == "readOnly").unwrap();
        assert_eq!(read_only.status, Status::Missing);
        let render = root.props.iter().find(|p| p.name == "render").unwrap();
        assert_eq!(render.status, Status::IntentionalDifference);
    }

    #[test]
    fn axis_with_no_matching_component_is_null_with_empty_parts() {
        let empty = empty_snapshot("dioxus-components");
        let base_ui = base_ui_fixture();
        let mut snapshots: StdBTreeMap<String, CatalogSnapshot> = StdBTreeMap::new();
        snapshots.insert("base-ui".to_string(), base_ui.clone());
        snapshots.insert("dioxus-components".to_string(), empty.clone());
        let introspection = fixture_introspection();
        let item = fixture_item("switch");

        let context = ResolutionContext {
            snapshots: &snapshots,
            base_ui: &base_ui,
        };
        let axis_parity =
            build_axis_parity(&item, &introspection, "dioxus-components", &empty, &context);
        assert_eq!(axis_parity.matched_component, None);
        assert!(axis_parity.parts.is_empty());
    }

    #[test]
    fn unavailable_upstream_part_is_recorded_unresolved_not_zero_missing() {
        let mut snapshot = base_ui_fixture();
        snapshot.entries[0].parts[0].props_source = PropsSource::Unavailable;
        let mut snapshots: StdBTreeMap<String, CatalogSnapshot> = StdBTreeMap::new();
        snapshots.insert("base-ui".to_string(), snapshot.clone());
        let introspection = fixture_introspection();
        let item = fixture_item("switch");

        let context = ResolutionContext {
            snapshots: &snapshots,
            base_ui: &snapshot,
        };
        let axis_parity = build_axis_parity(&item, &introspection, "base-ui", &snapshot, &context);
        let root = axis_parity
            .parts
            .iter()
            .find(|part| part.id == "root")
            .expect("root part");
        assert!(root.props.is_empty());
        assert!(root.unresolved.is_some());
    }

    #[test]
    fn inherits_from_dioxus_primitives_resolves_against_that_axis_not_base_ui() {
        let base_ui = empty_snapshot("base-ui");
        let dioxus_primitives = CatalogSnapshot {
            axis: "dioxus-primitives".to_string(),
            source: "https://example.invalid".to_string(),
            revision: "test".to_string(),
            refreshed_at: "2026-09-05".to_string(),
            entries: vec![CatalogEntry {
                id: "toggle".to_string(),
                name: "Toggle".to_string(),
                parts: vec![PartEntry {
                    id: "root".to_string(),
                    composition: Vec::new(),
                    props_source: PropsSource::Explicit {
                        props: vec![Prop {
                            name: "pressed".to_string(),
                            type_name: "bool".to_string(),
                            default: None,
                            description: None,
                        }],
                    },
                }],
            }],
        };
        let dioxus_components = CatalogEntry {
            id: "toggle".to_string(),
            name: "Toggle".to_string(),
            parts: vec![PartEntry {
                id: "root".to_string(),
                composition: Vec::new(),
                props_source: PropsSource::InheritsFrom {
                    reference: "dioxus-primitives.toggle.root".to_string(),
                },
            }],
        };
        let dioxus_components_snapshot = CatalogSnapshot {
            axis: "dioxus-components".to_string(),
            source: "https://example.invalid".to_string(),
            revision: "test".to_string(),
            refreshed_at: "2026-09-05".to_string(),
            entries: vec![dioxus_components],
        };
        let mut snapshots: StdBTreeMap<String, CatalogSnapshot> = StdBTreeMap::new();
        snapshots.insert("base-ui".to_string(), base_ui.clone());
        snapshots.insert("dioxus-primitives".to_string(), dioxus_primitives);
        snapshots.insert(
            "dioxus-components".to_string(),
            dioxus_components_snapshot.clone(),
        );

        let mut introspection = FileIntrospection {
            exists: true,
            ..Default::default()
        };
        introspection.components.push("Toggle".to_string());
        let item = fixture_item("toggle");

        let context = ResolutionContext {
            snapshots: &snapshots,
            base_ui: &base_ui,
        };
        let axis_parity = build_axis_parity(
            &item,
            &introspection,
            "dioxus-components",
            &dioxus_components_snapshot,
            &context,
        );
        let root = axis_parity
            .parts
            .iter()
            .find(|part| part.id == "root")
            .expect("root part");
        assert!(root.unresolved.is_none(), "{root:?}");
        assert!(root.props.iter().any(|p| p.name == "pressed"));
    }

    /// The dual explicit-augmentation-plus-composition shape (25 of
    /// shadcn's 61 real components, e.g. `Dialog.Content`'s
    /// `showCloseButton` on top of the `dialog.popup` primitive it also
    /// wraps): the merged prop list must contain both the local
    /// augmentation prop and the composition-referenced part's own props,
    /// not only the local one.
    #[test]
    fn explicit_part_with_composition_merges_both_prop_sources() {
        let base_ui = CatalogSnapshot {
            axis: "base-ui".to_string(),
            source: "https://base-ui.com".to_string(),
            revision: "test".to_string(),
            refreshed_at: "2026-09-05".to_string(),
            entries: vec![CatalogEntry {
                id: "dialog".to_string(),
                name: "Dialog".to_string(),
                parts: vec![PartEntry {
                    id: "popup".to_string(),
                    composition: Vec::new(),
                    props_source: PropsSource::Explicit {
                        props: vec![Prop {
                            name: "dismissible".to_string(),
                            type_name: "boolean".to_string(),
                            default: None,
                            description: None,
                        }],
                    },
                }],
            }],
        };
        let dialog_content = PartEntry {
            id: "content".to_string(),
            composition: vec![catalog::schema::CompositionRef {
                axis: "radix".to_string(),
                component: "dialog".to_string(),
                part: Some("content".to_string()),
            }],
            props_source: PropsSource::Explicit {
                props: vec![Prop {
                    name: "showCloseButton".to_string(),
                    type_name: "boolean".to_string(),
                    default: None,
                    description: None,
                }],
            },
        };
        let mut snapshots: StdBTreeMap<String, CatalogSnapshot> = StdBTreeMap::new();
        snapshots.insert("base-ui".to_string(), base_ui.clone());

        let resolved = resolve_full_part_props(&dialog_content, &snapshots, &base_ui);
        match resolved {
            ResolvedPartProps::Resolved(props) => {
                let names: BTreeSet<&str> = props.iter().map(|p| p.name.as_str()).collect();
                assert!(names.contains("showCloseButton"), "{names:?}");
                assert!(
                    names.contains("dismissible"),
                    "composition-inherited prop was dropped: {names:?}"
                );
            }
            other => panic!("expected resolved, got unresolved: {other:?}"),
        }
    }
}
