//! `cargo xtask baseui-compat sync|check|diff`: regenerates
//! `packages/adico-primitives/baseui_compatibility.json` against Base UI's
//! (<https://base-ui.com/react/components>) component/util inventory and
//! this crate's actual current source.
//!
//! What this automates:
//!   - Introspects each mapped `adico-primitives` file/module (component
//!     functions, Props struct fields, hooks used/defined) via
//!     [`crate::rust_introspect`].
//!   - `sync`/`diff` best-effort live-fetch base-ui.com's own component list
//!     to flag components added/removed upstream since [`UPSTREAM_COMPONENTS`]
//!     was last reviewed by hand.
//!
//! What it does NOT automate (edit the tables below instead): classifying a
//! component's status (built/partial/not_started/not_applicable) and which
//! `adico-primitives` file/registry item it maps to -- that needs human/AI
//! judgment, not a page scrape.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::rust_introspect::{self, FileIntrospection};

const BASEUI_COMPONENTS_URL: &str = "https://base-ui.com/react/components";

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

/// The full Base UI component inventory (37 as of 2026-08-31), hand-reviewed
/// against `packages/adico-primitives/src/`. Keep in sync with
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
        status: Status::NotStarted,
        adico_file: None,
        adico_registry_item: None,
        notes: "Base-UI-parity tier target, task 7.9.",
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
        adico_file: Some("combobox/"),
        adico_registry_item: Some("combobox"),
        notes: "Multi-file module (combobox/components/*).",
    },
    ComponentEntry {
        name: "Context Menu",
        status: Status::Partial,
        adico_file: Some("context_menu.rs"),
        adico_registry_item: Some("context-menu"),
        notes: "Flat, independently-implemented menu; not yet migrated onto the unified menu.rs primitive (task 7.8e).",
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
        status: Status::NotStarted,
        adico_file: None,
        adico_registry_item: None,
        notes: "Base-UI-parity tier target, task 7.9 (field semantics).",
    },
    ComponentEntry {
        name: "Fieldset",
        status: Status::NotStarted,
        adico_file: None,
        adico_registry_item: None,
        notes: "Base-UI-parity tier target, task 7.9.",
    },
    ComponentEntry {
        name: "Form",
        status: Status::NotStarted,
        adico_file: None,
        adico_registry_item: None,
        notes: "Base-UI-parity tier target, task 7.9. shadcn's own Form was found unportable (wave5-extras); Base UI's may differ, needs its own read.",
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
        notes: "Unified anatomy built (task 7.6a), but not yet composed on positioner::Positioner, not wired to use_typeahead, and not yet consumed by any registry item (task 7.8e).",
    },
    ComponentEntry {
        name: "Menubar",
        status: Status::Partial,
        adico_file: Some("menubar.rs"),
        adico_registry_item: Some("menubar"),
        notes: "Flat, independently-implemented; not yet migrated onto menu.rs (task 7.8e).",
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
        adico_file: Some("select/"),
        adico_registry_item: Some("select"),
        notes: "Multi-file module (select/components/*, select/context.rs, select/mod.rs).",
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
    name: &'static str,
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
    checked_at: String,
    added_upstream: Vec<String>,
    removed_upstream: Vec<String>,
}

fn primitives_src(root: &Path) -> PathBuf {
    root.join("packages/adico-primitives/src")
}

fn output_path(root: &Path) -> PathBuf {
    root.join("packages/adico-primitives/baseui_compatibility.json")
}

fn introspect_for(root: &Path, file_ref: &str) -> FileIntrospection {
    let src = primitives_src(root);
    if let Some(module) = file_ref.strip_suffix('/') {
        rust_introspect::introspect_directory(&src.join(module))
    } else {
        rust_introspect::introspect_file(&src.join(file_ref))
    }
}

fn build_document(root: &Path, check_upstream: bool) -> (serde_json::Value, [usize; 4]) {
    let mut counts = [0usize; 4]; // built, partial, not_started, not_applicable
    let components: Vec<ComponentOutput> = UPSTREAM_COMPONENTS
        .iter()
        .map(|entry| {
            match entry.status {
                Status::Built => counts[0] += 1,
                Status::Partial => counts[1] += 1,
                Status::NotStarted => counts[2] += 1,
                Status::NotApplicable => counts[3] += 1,
            }
            let mut output = ComponentOutput {
                name: entry.name,
                status: entry.status.as_str(),
                adico_primitive_file: entry
                    .adico_file
                    .map(|file| format!("packages/adico-primitives/src/{file}")),
                adico_registry_item: entry.adico_registry_item,
                notes: entry.notes,
                adico_components: None,
                adico_props: None,
                adico_hooks_defined: None,
                adico_hooks_used: None,
            };
            if let Some(file_ref) = entry.adico_file {
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

    let drift = if check_upstream {
        fetch_upstream_drift()
    } else {
        None
    };

    let document = serde_json::json!({
        "$schema_note": "Hand-maintain UPSTREAM_COMPONENTS/UPSTREAM_UTILS/ADICO_ONLY_EXTRAS in packages/adico-xtask/src/baseui_compat.rs; everything else here is regenerated from repo introspection.",
        "source": BASEUI_COMPONENTS_URL,
        "synced_at": today(),
        "generator": "cargo xtask baseui-compat sync",
        "summary": {
            "total_upstream_components": UPSTREAM_COMPONENTS.len(),
            "total_upstream_utils": UPSTREAM_UTILS.len(),
            "adico_only_extras": ADICO_ONLY_EXTRAS.len(),
            "components_built": counts[0],
            "components_partial": counts[1],
            "components_not_started": counts[2],
        },
        "upstream_drift_check": drift,
        "components": components,
        "utils": utils,
        "adico_only_extras": extras,
    });
    (document, counts)
}

fn today() -> String {
    let output = std::process::Command::new("date").arg("+%Y-%m-%d").output();
    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

/// Best-effort live check of Base UI's own component list, to flag upstream
/// drift. Returns `None` (not an error) if unreachable -- this command must
/// still work fully offline for the sync/write path.
fn fetch_upstream_names() -> Option<Vec<String>> {
    let response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("adico-xtask/1.0")
        .build()
        .ok()?
        .get(BASEUI_COMPONENTS_URL)
        .send()
        .ok()?;
    let html = response.text().ok()?;

    let mut slugs: Vec<String> = Vec::new();
    let marker = "/react/components/";
    let mut remaining = html.as_str();
    while let Some(start) = remaining.find(marker) {
        let after = &remaining[start + marker.len()..];
        let end = after
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(after.len());
        let slug = &after[..end];
        if !slug.is_empty() {
            slugs.push(slug.to_string());
        }
        remaining = &after[end..];
    }
    slugs.sort();
    slugs.dedup();
    // Next.js embeds internal page-cache-id links matching this same URL
    // shape (e.g. "page-694493450857fab0"); filter those out.
    let names = slugs
        .into_iter()
        .filter(|slug| {
            !(slug.starts_with("page-") && slug[5..].chars().all(|c| c.is_ascii_hexdigit()))
        })
        .map(|slug| {
            slug.split('-')
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    Some(names)
}

fn fetch_upstream_drift() -> Option<serde_json::Value> {
    let live = fetch_upstream_names()?;
    let tracked: std::collections::HashSet<String> = UPSTREAM_COMPONENTS
        .iter()
        .map(|entry| entry.name.to_lowercase())
        .collect();
    let live_set: std::collections::HashSet<String> =
        live.iter().map(|name| name.to_lowercase()).collect();
    let mut added: Vec<String> = live
        .iter()
        .filter(|name| !tracked.contains(&name.to_lowercase()))
        .cloned()
        .collect();
    added.sort();
    let mut removed: Vec<String> = UPSTREAM_COMPONENTS
        .iter()
        .map(|entry| entry.name.to_string())
        .filter(|name| !live_set.contains(&name.to_lowercase()))
        .collect();
    removed.sort();
    Some(
        serde_json::to_value(UpstreamDrift {
            checked_at: chrono_now(),
            added_upstream: added,
            removed_upstream: removed,
        })
        .unwrap(),
    )
}

fn chrono_now() -> String {
    let output = std::process::Command::new("date")
        .arg("-u")
        .arg("+%Y-%m-%dT%H:%M:%SZ")
        .output();
    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

pub fn sync(root: &Path) -> Result<(), String> {
    let (document, counts) = build_document(root, true);
    let path = output_path(root);
    let payload = serde_json::to_string_pretty(&document)
        .map_err(|error| format!("cannot serialize baseui_compatibility.json: {error}"))?;
    fs::write(&path, format!("{payload}\n"))
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    println!("Wrote {}", path.display());
    println!(
        "  components: {} tracked (built={}, partial={}, not_started={})",
        UPSTREAM_COMPONENTS.len(),
        counts[0],
        counts[1],
        counts[2]
    );
    if let Some(drift) = document
        .get("upstream_drift_check")
        .filter(|value| !value.is_null())
    {
        let added = drift["added_upstream"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);
        let removed = drift["removed_upstream"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);
        if added == 0 && removed == 0 {
            println!("  upstream drift check: no changes detected");
        } else {
            println!("  upstream drift detected: {drift}");
        }
    } else {
        println!("  upstream drift check skipped (network unreachable)");
    }
    Ok(())
}

pub fn check(root: &Path) -> Result<(), String> {
    let (document, _) = build_document(root, false);
    let path = output_path(root);
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let mut existing_value: serde_json::Value =
        serde_json::from_str(&existing).unwrap_or(serde_json::Value::Null);
    if let Some(obj) = existing_value.as_object_mut() {
        obj.remove("synced_at");
        obj.remove("upstream_drift_check");
    }
    let mut comparable = document.clone();
    if let Some(obj) = comparable.as_object_mut() {
        obj.remove("synced_at");
        obj.remove("upstream_drift_check");
    }
    if existing_value != comparable {
        return Err("baseui_compatibility.json is stale; run `cargo xtask baseui-compat sync` to regenerate.".to_string());
    }
    println!("baseui_compatibility.json is up to date.");
    Ok(())
}

pub fn diff() -> Result<(), String> {
    let Some(live) = fetch_upstream_names() else {
        return Err("could not reach base-ui.com; drift check skipped.".to_string());
    };
    let tracked: std::collections::HashSet<String> = UPSTREAM_COMPONENTS
        .iter()
        .map(|entry| entry.name.to_lowercase())
        .collect();
    let live_set: std::collections::HashSet<String> =
        live.iter().map(|name| name.to_lowercase()).collect();
    let mut added: Vec<&String> = live
        .iter()
        .filter(|name| !tracked.contains(&name.to_lowercase()))
        .collect();
    added.sort();
    let mut removed: Vec<&str> = UPSTREAM_COMPONENTS
        .iter()
        .map(|entry| entry.name)
        .filter(|name| !live_set.contains(&name.to_lowercase()))
        .collect();
    removed.sort();
    if added.is_empty() && removed.is_empty() {
        println!("No upstream drift: tracked list matches base-ui.com.");
    } else {
        if !added.is_empty() {
            println!("Added upstream (not yet tracked): {added:?}");
        }
        if !removed.is_empty() {
            println!("Removed upstream (still tracked): {removed:?}");
        }
    }
    Ok(())
}
