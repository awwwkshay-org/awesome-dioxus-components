//! `cargo xtask primitive-compat sync|check|diff`: regenerates
//! `statics/primitive_compatibility.json` -- `adico-primitives`' compatibility
//! against **both** of its upstream primitive inventories: Base UI and
//! `DioxusLabs/dioxus-components`'s `primitives/src/` tree.
//!
//! Both inventories are read from `statics/catalogs/<axis>.json`, produced
//! offline-reproducibly by `cargo xtask catalog fetch <axis>` (see
//! `crate::catalog`) -- this module never touches the network. It filters
//! `catalog::AXES` by [`crate::catalog::AxisKind::Primitive`] rather than
//! naming `base-ui`/`dioxus-primitives` in its iteration, though each
//! axis's specific hand-maintained judgment table below is inherently
//! axis-shaped and still needs its own join logic.
//!
//! What this automates:
//!   - Introspects each mapped `adico-primitives` file/module (component
//!     functions, Props struct fields, hooks used/defined) via
//!     [`crate::rust_introspect`].
//!   - Joins the fetched Base UI inventory against [`UPSTREAM_COMPONENTS`]
//!     by name, and reports (offline, from committed statics) any fetched
//!     component missing from that hand table, or any hand-table entry no
//!     longer present in the fetched inventory.
//!   - Derives the dioxus-primitives module inventory straight from
//!     `statics/catalogs/dioxus-primitives.json` -- no hand table for that
//!     axis, only a short exceptions list below.
//!
//! What it does NOT automate (edit the tables below instead): classifying a
//! Base UI component's status (built/partial/not_started/not_applicable) and
//! which `adico-primitives` file/registry item it maps to -- that needs
//! human/AI judgment, not a page scrape. Fetch never writes these fields
//! (see spec's "Hand-maintained judgment data survives fetch and sync").

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::catalog::{self, AxisKind, CatalogSnapshot};
use crate::rust_introspect::{self, FileIntrospection};

#[derive(Clone, Copy)]
struct ComponentEntry {
    name: &'static str,
    status: Status,
    adico_file: Option<&'static str>,
    adico_registry_item: Option<&'static str>,
    notes: &'static str,
}

#[derive(Clone, Copy)]
struct UtilEntry {
    name: &'static str,
    status: Status,
    adico_file: Option<&'static str>,
    notes: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    Built,
    Partial,
    NotStarted,
    NotApplicable,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Built => "built",
            Status::Partial => "partial",
            Status::NotStarted => "not_started",
            Status::NotApplicable => "not_applicable",
        }
    }
}

/// Hand-reviewed judgment for the Base UI axis against
/// `packages/adico-primitives/src/`, keyed by name and joined against the
/// fetched `statics/catalogs/base-ui.json` inventory. Keep in sync with
/// design.md §8a when a component's status changes.
const UPSTREAM_COMPONENTS: &[ComponentEntry] = &[
    ComponentEntry {
        name: "Accordion",
        status: Status::Built,
        adico_file: Some("accordion.rs"),
        adico_registry_item: Some("accordion"),
        notes: "Split into Accordion (single)/AccordionMulti, matching this crate's Select/SelectMulti convention rather than a type:'single'|'multiple' prop (task 7.8b).",
    },
    ComponentEntry {
        name: "Alert Dialog",
        status: Status::Built,
        adico_file: Some("alert_dialog.rs"),
        adico_registry_item: Some("alert-dialog"),
        notes: "",
    },
    ComponentEntry {
        name: "Autocomplete",
        status: Status::Built,
        adico_file: Some("autocomplete.rs"),
        adico_registry_item: None,
        notes: "Task 7.9, second of ten. Base UI's Autocomplete parts are exactly Combobox's minus chip/chip-remove/chips/item-indicator/label, so this is a thin re-export of combobox.rs's single-value Combobox parts under Autocomplete names (matching the dropdown_menu.rs-re-exports-menu precedent, task 2.3), plus the two genuinely-missing small parts (AutocompleteStatus, AutocompleteClear) built fresh. Correction to design.md/this task's own text: 'Autocomplete SHALL compose typeahead for type-to-select' does not survive contact with typeahead.rs's actual best_match signature (single-index jump-to-match via min_by, not a multi-item relevance filter) -- filtering here composes combobox::default_combobox_filter instead, the same substring filter Combobox itself ships and this crate has tested. AutocompleteClear only clears query text, not an already-selected value -- neither combobox.rs nor selectable::use_single_selectable_value expose a clear-selection callback (a real, separate gap, not fixed here). No registry item yet -- M7/M8 scope.",
    },
    ComponentEntry {
        name: "Avatar",
        status: Status::Built,
        adico_file: Some("avatar.rs"),
        adico_registry_item: Some("avatar"),
        notes: "",
    },
    ComponentEntry {
        name: "Button",
        status: Status::Built,
        adico_file: None,
        adico_registry_item: Some("button"),
        notes: "Native <button> semantics; no dedicated primitive needed, matching Base UI's own Button.",
    },
    ComponentEntry {
        name: "Checkbox",
        status: Status::Built,
        adico_file: Some("checkbox.rs"),
        adico_registry_item: Some("checkbox"),
        notes: "",
    },
    ComponentEntry {
        name: "Checkbox Group",
        status: Status::NotStarted,
        adico_file: None,
        adico_registry_item: None,
        notes: "Base-UI-parity tier target, task 7.9.",
    },
    ComponentEntry {
        name: "Collapsible",
        status: Status::Built,
        adico_file: Some("collapsible.rs"),
        adico_registry_item: Some("collapsible"),
        notes: "",
    },
    ComponentEntry {
        name: "Combobox",
        status: Status::Built,
        adico_file: Some("combobox.rs"),
        adico_registry_item: Some("combobox"),
        notes: "Flattened from a multi-file combobox/ module to a single combobox.rs at some point before this table was last synced; do not reintroduce a directory-style adico_file path here.",
    },
    ComponentEntry {
        name: "Context Menu",
        status: Status::Partial,
        adico_file: Some("context_menu.rs"),
        adico_registry_item: Some("context-menu"),
        notes: "Deliberately kept independent of menu.rs (task 2.3): Base UI's own ContextMenuRoot delegates to Menu.Root only because Menu.Popup is Positioner-anchored from the start, and menu::MenuContent isn't. Already implements the ARIA APG generic Menu pattern correctly (role=menu/menuitem). Still Partial, not Built: ContextMenuContent's use_outside_dismiss depends on the long-lived document::eval listener pattern found (2026-08-25, via live-browser Playwright testing) to never actually register its addEventListener call in this Dioxus web runtime -- unverified and unfixed this pass (no live browser available), so outside-click dismissal is a known-broken dependency, not a working feature. See provenance/records/adico-primitives-wave3-overlays.json's git history (the record itself was removed in task 2.3's closing commit once its last source unit was re-authored).",
    },
    ComponentEntry {
        name: "Dialog",
        status: Status::Built,
        adico_file: Some("dialog.rs"),
        adico_registry_item: Some("dialog"),
        notes: "",
    },
    ComponentEntry {
        name: "Drawer",
        status: Status::Built,
        adico_file: None,
        adico_registry_item: Some("sheet"),
        notes: "adico's 'Sheet' registry item is the shadcn/Base-UI Drawer equivalent; no dedicated primitive file.",
    },
    ComponentEntry {
        name: "Field",
        status: Status::Partial,
        adico_file: Some("field.rs"),
        adico_registry_item: None,
        notes: "Task 7.9 (field semantics), first of ten primitives. Root/Label/Description/Error built, plus a use_field_control hook standing in for Base UI's polymorphic Field.Control render-prop (Dioxus has no asChild/merge-props mechanism -- a real <input> calls the hook directly and spreads the returned id/aria-describedby/aria-invalid/disabled onto itself instead of being wrapped). Still Partial, not Built: no internal validate callback, validationMode (onSubmit/onBlur/onChange) timing engine, or debounced revalidation -- invalid is caller-supplied state, matching this crate's existing controlled-prop pattern elsewhere, not an internal validation engine. FieldError's show prop is a plain boolean, not Base UI's ValidityState-key match (no Dioxus ValidityState binding to match against). No registry item yet -- that is M7/M8 scope (shadcn's 'field'/'form' items), not this task's.",
    },
    ComponentEntry {
        name: "Fieldset",
        status: Status::Built,
        adico_file: Some("fieldset.rs"),
        adico_registry_item: None,
        notes: "Task 7.9. FieldsetRoot cascades its own disabled into every nested field::FieldRoot's Dioxus-side state via context (native <fieldset disabled> already disables descendant controls for free per the HTML spec; this context cascade exists so FieldRoot's own data-disabled/use_field_control state stays in sync with that, matching Base UI's FieldsetRootContext). No registry item yet -- M7/M8 scope, not this task's.",
    },
    ComponentEntry {
        name: "Form",
        status: Status::Partial,
        adico_file: Some("form.rs"),
        adico_registry_item: None,
        notes: "Task 7.9. FormRoot relays a native <form>'s submit event (default-prevented) to an on_submit callback. Still Partial, not Built: no cross-field validation orchestration -- Base UI's Form aggregates every nested Field.Root's validate result on submit, exposes an actionsRef.validate() imperative handle, and accepts an external errors object for server-returned validation; none of that exists since field.rs itself has no internal validate callback yet (see that entry's own notes). shadcn's own Form was separately found unportable (wave5-extras) and is unrelated to this Base-UI-axis entry. No registry item yet -- M7/M8 scope, not this task's.",
    },
    ComponentEntry {
        name: "Input",
        status: Status::Built,
        adico_file: None,
        adico_registry_item: Some("input"),
        notes: "Native <input> semantics; no dedicated primitive needed.",
    },
    ComponentEntry {
        name: "Menu",
        status: Status::Partial,
        adico_file: Some("menu.rs"),
        adico_registry_item: None,
        notes: "Unified anatomy built (task 7.6a); dropdown_menu.rs re-exports its components directly (task 2.3), so it is now consumed indirectly via the 'dropdown-menu' registry item, but there is no standalone 'menu' registry item and it is still not composed on positioner::Positioner nor wired to use_typeahead.",
    },
    ComponentEntry {
        name: "Menubar",
        status: Status::Built,
        adico_file: Some("menubar.rs"),
        adico_registry_item: Some("menubar"),
        notes: "Deliberately kept independent of menu.rs (task 2.3): Base UI's own Menubar shares nothing with Menu beyond a roving-focus composite container, which menubar.rs already gets from crate::collection; its per-menu is_open/set_open-across-siblings coordination has no menu::MenuContext counterpart to adapt onto.",
    },
    ComponentEntry {
        name: "Meter",
        status: Status::NotStarted,
        adico_file: None,
        adico_registry_item: None,
        notes: "Base-UI-parity tier target, task 7.9.",
    },
    ComponentEntry {
        name: "Navigation Menu",
        status: Status::NotStarted,
        adico_file: None,
        adico_registry_item: None,
        notes: "Base-UI-parity tier target, task 7.9.",
    },
    ComponentEntry {
        name: "Number Field",
        status: Status::NotStarted,
        adico_file: None,
        adico_registry_item: None,
        notes: "Base-UI-parity tier target, task 7.9.",
    },
    ComponentEntry {
        name: "OTP Field",
        status: Status::NotStarted,
        adico_file: None,
        adico_registry_item: None,
        notes: "Base-UI-parity tier target, task 7.9.",
    },
    ComponentEntry {
        name: "Popover",
        status: Status::Built,
        adico_file: Some("popover.rs"),
        adico_registry_item: Some("popover"),
        notes: "Composes positioner::Positioner for real collision-aware placement (task 7.8c); sideOffset/Arrow prop gaps vs shadcn remain.",
    },
    ComponentEntry {
        name: "Preview Card",
        status: Status::NotStarted,
        adico_file: None,
        adico_registry_item: None,
        notes: "Base-UI-parity tier target, task 7.9 -- the original inspiration for this crate's shared-primitive redesign.",
    },
    ComponentEntry {
        name: "Progress",
        status: Status::Built,
        adico_file: Some("progress.rs"),
        adico_registry_item: Some("progress"),
        notes: "",
    },
    ComponentEntry {
        name: "Radio",
        status: Status::Built,
        adico_file: Some("radio_group.rs"),
        adico_registry_item: Some("radio-group"),
        notes: "adico names it RadioGroup/RadioItem, matching Radix's own naming more than Base UI's bare 'Radio'.",
    },
    ComponentEntry {
        name: "Scroll Area",
        status: Status::Built,
        adico_file: Some("scroll_area.rs"),
        adico_registry_item: Some("scroll-area"),
        notes: "Native-overflow/CSS toggle, not a custom-styled scrollbar-thumb sub-component.",
    },
    ComponentEntry {
        name: "Select",
        status: Status::Built,
        adico_file: Some("select.rs"),
        adico_registry_item: Some("select"),
        notes: "Flattened from a multi-file select/ module to a single select.rs at some point before this table was last synced; do not reintroduce a directory-style adico_file path here.",
    },
    ComponentEntry {
        name: "Separator",
        status: Status::Built,
        adico_file: Some("separator.rs"),
        adico_registry_item: Some("separator"),
        notes: "",
    },
    ComponentEntry {
        name: "Slider",
        status: Status::Built,
        adico_file: Some("slider.rs"),
        adico_registry_item: Some("slider"),
        notes: "Pointer-drag on web is suspected non-functional (task 7.7 finding, unverified without a browser); keyboard control is tested and works.",
    },
    ComponentEntry {
        name: "Switch",
        status: Status::Built,
        adico_file: Some("switch.rs"),
        adico_registry_item: Some("switch"),
        notes: "",
    },
    ComponentEntry {
        name: "Tabs",
        status: Status::Built,
        adico_file: Some("tabs.rs"),
        adico_registry_item: Some("tabs"),
        notes: "",
    },
    ComponentEntry {
        name: "Toast",
        status: Status::Built,
        adico_file: Some("toast.rs"),
        adico_registry_item: Some("toast"),
        notes: "Its F6 focus-region shortcut uses the same long-lived document::eval listener pattern confirmed broken elsewhere (task 7.4d finding); likely non-functional on web, unverified.",
    },
    ComponentEntry {
        name: "Toggle",
        status: Status::Built,
        adico_file: Some("toggle.rs"),
        adico_registry_item: Some("toggle"),
        notes: "",
    },
    ComponentEntry {
        name: "Toggle Group",
        status: Status::Built,
        adico_file: Some("toggle_group.rs"),
        adico_registry_item: Some("toggle-group"),
        notes: "",
    },
    ComponentEntry {
        name: "Toolbar",
        status: Status::Built,
        adico_file: Some("toolbar.rs"),
        adico_registry_item: Some("toolbar"),
        notes: "",
    },
    ComponentEntry {
        name: "Tooltip",
        status: Status::Built,
        adico_file: Some("tooltip.rs"),
        adico_registry_item: Some("tooltip"),
        notes: "Composes positioner::Positioner for real collision-aware placement (task 7.8c); TooltipProvider/sideOffset/Arrow gaps vs shadcn remain.",
    },
];

const UPSTREAM_UTILS: &[UtilEntry] = &[
    UtilEntry {
        name: "CSP Provider",
        status: Status::NotApplicable,
        adico_file: None,
        notes: "React inline-style injection concern; no Dioxus/Tailwind equivalent (design.md §8a).",
    },
    UtilEntry {
        name: "Direction Provider",
        status: Status::Built,
        adico_file: Some("direction.rs"),
        notes: "Direction/DirectionProvider/use_direction (task 7.3a).",
    },
    UtilEntry {
        name: "mergeProps",
        status: Status::NotApplicable,
        adico_file: None,
        notes: "Maps to #[props(extends = GlobalAttributes)] + Element composition, already this crate's established pattern (design.md §8a).",
    },
    UtilEntry {
        name: "useRender",
        status: Status::NotApplicable,
        adico_file: None,
        notes: "Same mapping as mergeProps.",
    },
];

/// Hand-maintained notes for dioxus-primitives modules that need one --
/// everything else on that axis (which modules exist, which are built, which
/// adico module has no dioxus-components counterpart) is derived, not listed
/// here. Keep in sync with `docs/adico/m3-acceptance.md` when a reason
/// changes.
const DIOXUS_MODULE_NOTES: &[(&str, &str)] = &[
    (
        "navbar",
        "Out of M3 scope by its own classification (NEEDS_PARITY_UPDATES, not \"suitable for current reuse\"); see docs/adico/m3-acceptance.md.",
    ),
    (
        "virtual",
        "Catalog artifact, not a real component: this fetched entry has zero parts/props of its own -- it's the parent-namespace stub for upstream's `virtual::virtual_list` module tree. The actual component is separately fetched and tracked as `virtual_list`, which correctly resolves to `virtual_list.rs`. Do not add a file-path override for this entry; that would double-count the same adico file under two module names (tried and reverted -- see task 8.1's evidence in reauthor-primitives-from-independent-spec/tasks.md).",
    ),
];

/// Primitives/registry items adico has that Base UI has no equivalent for.
const ADICO_ONLY_EXTRAS: &[(&str, &str)] = &[
    ("DatePicker", "date_picker.rs"),
    ("ColorPicker", "color_picker.rs"),
    ("DragAndDropList", "drag_and_drop_list.rs"),
    ("TagGroup", "tag_group.rs"),
    ("HoverCard", "hover_card.rs"),
    ("VirtualList", "virtual_list.rs"),
    ("Calendar", "calendar.rs"),
    ("AspectRatio", "aspect_ratio.rs"),
    ("Label", "label.rs"),
    ("ThemeMode", "theme_mode.rs"),
];

#[derive(Serialize)]
struct ComponentOutput {
    name: String,
    status: &'static str,
    adico_primitive_file: Option<String>,
    adico_registry_item: Option<&'static str>,
    notes: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    adico_components: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    adico_props: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    adico_hooks_defined: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    adico_hooks_used: Option<Vec<String>>,
}

#[derive(Serialize)]
struct UtilOutput {
    name: &'static str,
    status: &'static str,
    adico_primitive_file: Option<String>,
    notes: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    adico_hooks_defined: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ExtraOutput {
    name: &'static str,
    adico_primitive_file: String,
}

#[derive(Serialize)]
struct UpstreamDrift {
    /// Not in the fetched `statics/catalogs/base-ui.json` inventory but
    /// hand-listed in [`UPSTREAM_COMPONENTS`] -- upstream may have removed
    /// or renamed it since the table was last reviewed.
    tracked_but_missing_upstream: Vec<String>,
    /// In the fetched inventory but not yet reviewed into
    /// [`UPSTREAM_COMPONENTS`] -- defaults to `not_started` until reviewed.
    upstream_but_not_yet_tracked: Vec<String>,
}

#[derive(Serialize)]
struct DioxusModuleOutput {
    module: String,
    status: &'static str,
    adico_primitive_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    adico_components: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    adico_props: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    adico_hooks_defined: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    adico_hooks_used: Option<Vec<String>>,
}

fn primitives_src(root: &Path) -> PathBuf {
    root.join("packages/adico-primitives/src")
}

fn output_path(root: &Path) -> PathBuf {
    root.join("statics/primitive_compatibility.json")
}

/// The adico-primitives crate's own top-level module names (one per
/// `src/<name>.rs` file or `src/<name>/` directory), used to find this
/// axis's status and its adico-only extras by set difference against the
/// fetched dioxus-primitives module inventory.
fn adico_primitive_modules(root: &Path) -> BTreeSet<String> {
    let mut modules = BTreeSet::new();
    let Ok(entries) = fs::read_dir(primitives_src(root)) else {
        return modules;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if path.is_dir() {
            if name != "js" && name != "ts" {
                modules.insert(name.to_string());
            }
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") && name != "lib" {
            modules.insert(name.to_string());
        }
    }
    modules
}

fn dioxus_module_file_ref(root: &Path, module: &str) -> Option<String> {
    let src = primitives_src(root);
    if src.join(format!("{module}.rs")).is_file() {
        Some(format!("{module}.rs"))
    } else if src.join(module).is_dir() {
        Some(format!("{module}/"))
    } else {
        None
    }
}

fn build_dioxus_primitives_axis(root: &Path, snapshot: &CatalogSnapshot) -> serde_json::Value {
    // Fetched entries carry the module's raw (snake_case) name in `name`
    // and a kebab-cased `id`; filesystem lookups need the raw form.
    let dioxus_modules: BTreeSet<String> = snapshot
        .entries
        .iter()
        .map(|entry| entry.name.clone())
        .collect();
    let adico_modules = adico_primitive_modules(root);

    let mut built = 0usize;
    let mut not_started = 0usize;
    let modules: Vec<DioxusModuleOutput> = dioxus_modules
        .iter()
        .map(|module| {
            let file_ref = dioxus_module_file_ref(root, module);
            let status = if file_ref.is_some() {
                built += 1;
                "built"
            } else {
                not_started += 1;
                "not_started"
            };
            let notes = DIOXUS_MODULE_NOTES
                .iter()
                .find(|(name, _)| name == module)
                .map(|(_, note)| *note);
            let introspection = file_ref
                .as_deref()
                .map(|reference| introspect_for(root, reference));
            DioxusModuleOutput {
                module: module.clone(),
                status,
                adico_primitive_file: file_ref
                    .map(|reference| format!("packages/adico-primitives/src/{reference}")),
                notes,
                adico_components: introspection.as_ref().map(|i| i.components.clone()),
                adico_props: introspection
                    .as_ref()
                    .map(|i| serde_json::to_value(&i.props).unwrap()),
                adico_hooks_defined: introspection.as_ref().map(|i| i.hooks_defined.clone()),
                adico_hooks_used: introspection.as_ref().map(|i| i.hooks_used.clone()),
            }
        })
        .collect();

    let adico_only_extras: Vec<&String> = adico_modules.difference(&dioxus_modules).collect();

    serde_json::json!({
        "source": {
            "axis": snapshot.axis,
            "source": snapshot.source,
            "revision": snapshot.revision,
            "catalog_refreshed_at": snapshot.refreshed_at,
            "catalog_path": "statics/catalogs/dioxus-primitives.json",
            "refresh_command": "cargo xtask catalog fetch dioxus-primitives",
        },
        "summary": {
            "total_modules": modules.len(),
            "built": built,
            "not_started": not_started,
            "adico_only_extras": adico_only_extras.len(),
        },
        "modules": modules,
        "adico_only_extras": adico_only_extras,
    })
}

fn introspect_for(root: &Path, file_ref: &str) -> FileIntrospection {
    let src = primitives_src(root);
    if let Some(module) = file_ref.strip_suffix('/') {
        rust_introspect::introspect_directory(&src.join(module))
    } else {
        rust_introspect::introspect_file(&src.join(file_ref))
    }
}

fn build_base_ui_axis(root: &Path, snapshot: &CatalogSnapshot) -> (serde_json::Value, [usize; 3]) {
    let mut counts = [0usize; 3]; // built, partial, not_started
    let components: Vec<ComponentOutput> = snapshot
        .entries
        .iter()
        .map(|entry| {
            let hand = UPSTREAM_COMPONENTS
                .iter()
                .find(|candidate| candidate.name == entry.name);
            let status = hand
                .map(|candidate| candidate.status)
                .unwrap_or(Status::NotStarted);
            match status {
                Status::Built => counts[0] += 1,
                Status::Partial => counts[1] += 1,
                Status::NotStarted => counts[2] += 1,
                Status::NotApplicable => {}
            }
            let adico_file = hand.and_then(|candidate| candidate.adico_file);
            let mut output = ComponentOutput {
                name: entry.name.clone(),
                status: status.as_str(),
                adico_primitive_file: adico_file
                    .map(|file| format!("packages/adico-primitives/src/{file}")),
                adico_registry_item: hand.and_then(|candidate| candidate.adico_registry_item),
                notes: hand.map(|candidate| candidate.notes).unwrap_or_default(),
                adico_components: None,
                adico_props: None,
                adico_hooks_defined: None,
                adico_hooks_used: None,
            };
            if let Some(file_ref) = adico_file {
                let introspection = introspect_for(root, file_ref);
                output.adico_components = Some(introspection.components);
                output.adico_props = Some(serde_json::to_value(&introspection.props).unwrap());
                output.adico_hooks_defined = Some(introspection.hooks_defined);
                output.adico_hooks_used = Some(introspection.hooks_used);
            }
            output
        })
        .collect();

    let utils: Vec<UtilOutput> = UPSTREAM_UTILS
        .iter()
        .map(|entry| {
            let mut output = UtilOutput {
                name: entry.name,
                status: entry.status.as_str(),
                adico_primitive_file: entry
                    .adico_file
                    .map(|file| format!("packages/adico-primitives/src/{file}")),
                notes: entry.notes,
                adico_hooks_defined: None,
            };
            if let Some(file_ref) = entry.adico_file {
                output.adico_hooks_defined = Some(introspect_for(root, file_ref).hooks_defined);
            }
            output
        })
        .collect();

    let extras: Vec<ExtraOutput> = ADICO_ONLY_EXTRAS
        .iter()
        .map(|(name, file)| ExtraOutput {
            name,
            adico_primitive_file: format!("packages/adico-primitives/src/{file}"),
        })
        .collect();

    let fetched_names: BTreeSet<&str> = snapshot
        .entries
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    let mut tracked_but_missing_upstream: Vec<String> = UPSTREAM_COMPONENTS
        .iter()
        .map(|entry| entry.name.to_string())
        .filter(|name| !fetched_names.contains(name.as_str()))
        .collect();
    tracked_but_missing_upstream.sort();
    let tracked_names: BTreeSet<&str> =
        UPSTREAM_COMPONENTS.iter().map(|entry| entry.name).collect();
    let mut upstream_but_not_yet_tracked: Vec<String> = snapshot
        .entries
        .iter()
        .map(|entry| entry.name.clone())
        .filter(|name| !tracked_names.contains(name.as_str()))
        .collect();
    upstream_but_not_yet_tracked.sort();

    let document = serde_json::json!({
        "source": {
            "axis": snapshot.axis,
            "source": snapshot.source,
            "revision": snapshot.revision,
            "catalog_refreshed_at": snapshot.refreshed_at,
            "catalog_path": "statics/catalogs/base-ui.json",
            "refresh_command": "cargo xtask catalog fetch base-ui",
        },
        "summary": {
            "total_upstream_components": snapshot.entries.len(),
            "total_upstream_utils": UPSTREAM_UTILS.len(),
            "adico_only_extras": ADICO_ONLY_EXTRAS.len(),
            "components_built": counts[0],
            "components_partial": counts[1],
            "components_not_started": counts[2],
        },
        "upstream_drift_check": UpstreamDrift {
            tracked_but_missing_upstream,
            upstream_but_not_yet_tracked,
        },
        "components": components,
        "utils": utils,
        "adico_only_extras": extras,
    });
    (document, counts)
}

fn build_document(root: &Path) -> Result<serde_json::Value, String> {
    let mut axes = serde_json::Map::new();
    let mut counts = [0usize; 3];

    for axis in catalog::axes_of_kind(AxisKind::Primitive) {
        let snapshot = catalog::read_snapshot(root, axis.id)?;
        match axis.id {
            "base-ui" => {
                let (document, base_counts) = build_base_ui_axis(root, &snapshot);
                counts = base_counts;
                axes.insert("base_ui".to_string(), document);
            }
            "dioxus-primitives" => {
                axes.insert(
                    "dioxus_primitives".to_string(),
                    build_dioxus_primitives_axis(root, &snapshot),
                );
            }
            other => {
                // A newly registered primitive axis with no bespoke
                // hand-table integration yet: still surface its raw fetched
                // inventory rather than silently dropping it.
                axes.insert(
                    other.replace('-', "_"),
                    serde_json::json!({
                        "source": {
                            "axis": snapshot.axis,
                            "source": snapshot.source,
                            "revision": snapshot.revision,
                            "catalog_refreshed_at": snapshot.refreshed_at,
                        },
                        "summary": { "total_entries": snapshot.entries.len() },
                        "entries": snapshot.entries,
                    }),
                );
            }
        }
    }

    let mut document = serde_json::json!({
        "$schema_note": "Hand-maintain UPSTREAM_COMPONENTS/UPSTREAM_UTILS/ADICO_ONLY_EXTRAS/DIOXUS_MODULE_NOTES in packages/adico-xtask/src/primitive_compat.rs; everything else here is regenerated from repo introspection and committed statics/catalogs/*.json (see `cargo xtask catalog fetch`).",
        "synced_at": crate::today(),
        "generator": "cargo xtask primitive-compat sync",
    });
    if let Some(map) = document.as_object_mut() {
        for (key, value) in axes {
            map.insert(key, value);
        }
    }
    let _ = counts; // retained for the caller's println summary
    Ok(document)
}

/// Strips fields that legitimately vary run-to-run (the sync timestamp) so
/// `check` compares only the parts that should be byte-identical.
fn strip_volatile_fields(document: &mut serde_json::Value) {
    if let Some(obj) = document.as_object_mut() {
        obj.remove("synced_at");
    }
}

pub fn sync(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root.join("statics"))
        .map_err(|error| format!("cannot create statics/: {error}"))?;
    let document = build_document(root)?;
    let path = output_path(root);
    let payload = serde_json::to_string_pretty(&document)
        .map_err(|error| format!("cannot serialize primitive_compatibility.json: {error}"))?;
    fs::write(&path, format!("{payload}\n"))
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    println!("Wrote {}", path.display());
    if let Some(base_ui_summary) = document.get("base_ui").and_then(|axis| axis.get("summary")) {
        println!("  base_ui: {base_ui_summary}");
    }
    if let Some(dioxus_summary) = document
        .get("dioxus_primitives")
        .and_then(|axis| axis.get("summary"))
    {
        println!("  dioxus_primitives: {dioxus_summary}");
    }
    if let Some(drift) = document
        .get("base_ui")
        .and_then(|axis| axis.get("upstream_drift_check"))
    {
        let missing = drift["tracked_but_missing_upstream"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);
        let untracked = drift["upstream_but_not_yet_tracked"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);
        if missing == 0 && untracked == 0 {
            println!("  base_ui upstream drift check: no changes detected");
        } else {
            println!("  base_ui upstream drift detected: {drift}");
        }
    }
    Ok(())
}

pub fn check(root: &Path) -> Result<(), String> {
    let document = build_document(root)?;
    let path = output_path(root);
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let mut existing_value: serde_json::Value =
        serde_json::from_str(&existing).unwrap_or(serde_json::Value::Null);
    strip_volatile_fields(&mut existing_value);
    let mut comparable = document.clone();
    strip_volatile_fields(&mut comparable);
    if existing_value != comparable {
        return Err("primitive_compatibility.json is stale; run `cargo xtask primitive-compat sync` to regenerate.".to_string());
    }
    println!("primitive_compatibility.json is up to date.");
    Ok(())
}

/// Offline drift report: fetched `statics/catalogs/base-ui.json` vs.
/// [`UPSTREAM_COMPONENTS`]. Run `cargo xtask catalog fetch base-ui` first to
/// refresh the committed snapshot -- this command itself never touches the
/// network (see spec's "Catalog fetch is the sole network-touching command").
pub fn diff(root: &Path) -> Result<(), String> {
    let snapshot = catalog::read_snapshot(root, "base-ui")?;
    let (document, _) = build_base_ui_axis(root, &snapshot);
    let drift = &document["upstream_drift_check"];
    let missing = drift["tracked_but_missing_upstream"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let untracked = drift["upstream_but_not_yet_tracked"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if missing.is_empty() && untracked.is_empty() {
        println!("No drift: UPSTREAM_COMPONENTS matches statics/catalogs/base-ui.json.");
    } else {
        if !untracked.is_empty() {
            println!("In statics/catalogs/base-ui.json but not yet tracked: {untracked:?}");
        }
        if !missing.is_empty() {
            println!(
                "Tracked in UPSTREAM_COMPONENTS but missing from statics/catalogs/base-ui.json: {missing:?}"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::schema::{CatalogEntry, PartEntry, PropsSource};

    fn fake_snapshot(entry_names: &[&str]) -> CatalogSnapshot {
        CatalogSnapshot {
            axis: "base-ui".to_string(),
            source: "https://base-ui.com/react/components".to_string(),
            revision: "test".to_string(),
            refreshed_at: "2026-08-31".to_string(),
            entries: entry_names
                .iter()
                .map(|name| CatalogEntry {
                    id: name.to_lowercase().replace(' ', "-"),
                    name: name.to_string(),
                    parts: vec![PartEntry {
                        id: "root".to_string(),
                        composition: Vec::new(),
                        props_source: PropsSource::Unavailable,
                    }],
                })
                .collect(),
        }
    }

    #[test]
    fn hand_maintained_status_survives_fetched_snapshot_changes() {
        let root = std::env::temp_dir();
        // "Dialog" is hand-tracked as Built in UPSTREAM_COMPONENTS; this must
        // hold no matter what a freshly fetched snapshot contains, because
        // fetch never writes the status field.
        let (before, _) = build_base_ui_axis(&root, &fake_snapshot(&["Dialog"]));
        let (after, _) = build_base_ui_axis(&root, &fake_snapshot(&["Dialog", "Some New Thing"]));
        assert_eq!(before["components"][0]["status"], "built");
        assert_eq!(after["components"][0]["status"], "built");
    }

    #[test]
    fn untracked_fetched_entry_defaults_to_not_started() {
        let root = std::env::temp_dir();
        let (document, _) = build_base_ui_axis(&root, &fake_snapshot(&["Some New Thing"]));
        assert_eq!(document["components"][0]["status"], "not_started");
        assert_eq!(
            document["upstream_drift_check"]["upstream_but_not_yet_tracked"][0],
            "Some New Thing"
        );
    }
}
