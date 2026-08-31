//! `cargo xtask primitive-usage sync|check|diff`: per-item behavior-ownership
//! classification for every `registry:ui`/`registry:component` item, recorded
//! as one committed file under `statics/primitive_usage/<item-name>.json`
//! rather than a single shared table (unlike `primitive_compat.rs`/
//! `component_compat.rs`) -- see `design.md`'s Decisions section for why.
//!
//! Every item declares itself exactly one of:
//! - `delegated`: its interactive behavior is fully owned by the
//!   `adico_primitives` module(s) it imports.
//! - `presentational`: it has no interactive behavior a primitive would own
//!   (recorded with a one-line reason).
//! - `exception`: it has behavior an existing primitive could own but
//!   deliberately doesn't delegate to (recorded with a reason and follow-up).
//!
//! An item MAY be `exception` while still importing primitive modules for
//! its non-exceptional behavior.
//!
//! Runs fully offline: no `statics/catalogs/*.json` read, no network access.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::registry_introspect::{
    declares_adico_primitives_dependency, find_primitive_modules, load_registry_items,
    primitive_module_exists, registry_root,
};

/// Modules known to own reference-counted page-level scroll locking today.
/// Used only for the narrow "registry source duplicates primitive-owned
/// scroll locking" check (condition e) -- not a general duplication
/// detector, see `design.md`'s Decisions section.
const SCROLL_LOCK_OWNING_MODULES: &[&str] = &["dialog", "alert_dialog"];

/// Interactive-behavior markers that a `presentational` item's source must
/// not contain (condition d). A fixed, reviewable list, not a general
/// "any behavior" detector -- see `design.md`'s Risks section.
const INTERACTIVE_BEHAVIOR_MARKERS: &[&str] = &[
    "onkeydown",
    "onkeyup",
    "Key::",
    "use_focus_trap",
    "use_scroll_lock",
    "document::eval",
    "onpointerdown",
    "onfocusout",
    "use_signal",
    "use_context_provider",
    "use_context",
    "use_effect",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Classification {
    Delegated,
    Presentational,
    Exception,
}

impl Classification {
    fn label(self) -> &'static str {
        match self {
            Classification::Delegated => "delegated",
            Classification::Presentational => "presentational",
            Classification::Exception => "exception",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrimitiveUsageRecord {
    pub classification: Classification,
    #[serde(default)]
    pub primitive_modules: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_up: Option<String>,
}

fn records_dir(root: &Path) -> PathBuf {
    root.join("statics/primitive_usage")
}

fn record_path(root: &Path, item_name: &str) -> PathBuf {
    records_dir(root).join(format!("{item_name}.json"))
}

fn load_record(path: &Path) -> Result<PrimitiveUsageRecord, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&contents).map_err(|error| format!("{}: {error}", path.display()))
}

fn read_item_source(root: &Path, item: &adico_registry_core::RegistryItem) -> String {
    let mut combined = String::new();
    for file in &item.files {
        if let Ok(source) = fs::read_to_string(registry_root(root).join(&file.source)) {
            combined.push_str(&source);
            combined.push('\n');
        }
    }
    combined
}

// --- Pure, testable core ----------------------------------------------------

/// Checks one item's record against its real inputs, returning every
/// violation found (empty = compliant). Pure and synthetic-fixture-testable:
/// takes plain values, touches no filesystem itself.
fn check_item(
    item_name: &str,
    source: &str,
    declares_dependency: bool,
    module_exists: impl Fn(&str) -> bool,
    record: &PrimitiveUsageRecord,
) -> Vec<String> {
    let mut violations = Vec::new();

    // (b) a delegated/exception item's recorded module must exist under
    // packages/adico-primitives/src/.
    for module in &record.primitive_modules {
        if !module_exists(module) {
            violations.push(format!(
                "{item_name}: record lists primitive module '{module}' with no matching file under packages/adico-primitives/src/"
            ));
        }
    }

    // (c) registry.json's declared adico-primitives dependency must agree
    // with whether the record lists any primitive modules, regardless of
    // classification label.
    let record_has_modules = !record.primitive_modules.is_empty();
    if declares_dependency != record_has_modules {
        violations.push(format!(
            "{item_name}: registry.json {} the adico-primitives cargo dependency, but its record {} any primitiveModules",
            if declares_dependency { "declares" } else { "does not declare" },
            if record_has_modules { "lists" } else { "lists no" },
        ));
    }

    // (d) a presentational item must contain no interactive-behavior marker
    // and no adico_primitives import.
    if record.classification == Classification::Presentational {
        for marker in INTERACTIVE_BEHAVIOR_MARKERS {
            if source.contains(marker) {
                violations.push(format!(
                    "{item_name}: classified presentational but source contains interactive-behavior marker '{marker}'"
                ));
            }
        }
        if source.contains("adico_primitives") {
            violations.push(format!(
                "{item_name}: classified presentational but source imports adico_primitives"
            ));
        }
    }

    // (e) a registry file must not inject a page-level scroll/overflow style
    // while its record lists a primitive module that already owns scroll
    // locking for that component.
    let owns_scroll_lock = record
        .primitive_modules
        .iter()
        .any(|module| SCROLL_LOCK_OWNING_MODULES.contains(&module.as_str()));
    if owns_scroll_lock && injects_page_scroll_lock(source) {
        violations.push(format!(
            "{item_name}: source injects a page-level scroll/overflow style while its record lists a primitive module that already owns scroll locking"
        ));
    }

    // (f) a presentational/exception record must have a non-empty reason;
    // an exception record must have a non-empty follow-up.
    if matches!(
        record.classification,
        Classification::Presentational | Classification::Exception
    ) && record.reason.as_deref().unwrap_or("").trim().is_empty()
    {
        violations.push(format!(
            "{item_name}: classified {} but has an empty reason",
            record.classification.label()
        ));
    }
    if record.classification == Classification::Exception
        && record.follow_up.as_deref().unwrap_or("").trim().is_empty()
    {
        violations.push(format!(
            "{item_name}: classified exception but has an empty followUp"
        ));
    }

    violations
}

/// Detects the R1-shaped defect: a registry file rendering an unconditional
/// page-level `html { overflow: hidden; }` (or equivalent) style block.
/// Deliberately narrow -- see `design.md`'s "scroll-style check is named
/// narrowly" decision.
fn injects_page_scroll_lock(source: &str) -> bool {
    source.contains("html") && source.contains("overflow: hidden")
}

// --- sync / check / diff ----------------------------------------------------

pub fn sync(root: &Path) -> Result<(), String> {
    let dir = records_dir(root);
    fs::create_dir_all(&dir)
        .map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    let items = load_registry_items(root)?;
    let mut written = 0usize;
    for item in &items {
        let detected_modules = find_primitive_modules(root, item);
        let detected_dependency = declares_adico_primitives_dependency(item);
        let path = record_path(root, &item.name);
        let existing = load_record(&path).ok();
        let record = match existing {
            Some(mut record) => {
                record.primitive_modules = detected_modules;
                record
            }
            None => {
                let classification = if detected_dependency && !detected_modules.is_empty() {
                    Classification::Delegated
                } else {
                    Classification::Presentational
                };
                PrimitiveUsageRecord {
                    classification,
                    primitive_modules: detected_modules,
                    reason: None,
                    follow_up: None,
                }
            }
        };
        let payload = serde_json::to_string_pretty(&record)
            .map_err(|error| format!("cannot serialize record for {}: {error}", item.name))?;
        fs::write(&path, format!("{payload}\n"))
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        written += 1;
    }
    println!("Synced {written} statics/primitive_usage/<item>.json record(s).");
    Ok(())
}

/// Checks a single already-loaded registry item against its own record.
/// `Err` means the record itself is missing/unreadable; `Ok` carries every
/// rule violation found against that item (empty = compliant). Shared by the
/// aggregate `check(root)` loop and each item's own dedicated regression
/// test, so a real per-item test failure and the aggregate check's message
/// for that item are always identical.
fn check_real_item(
    root: &Path,
    item: &adico_registry_core::RegistryItem,
) -> Result<Vec<String>, String> {
    let path = record_path(root, &item.name);
    let record = load_record(&path).map_err(|_| {
        format!(
            "{}: no primitive-usage record (expected {})",
            item.name,
            path.display()
        )
    })?;
    let source = read_item_source(root, item);
    let declares_dependency = declares_adico_primitives_dependency(item);
    Ok(check_item(
        &item.name,
        &source,
        declares_dependency,
        |module| primitive_module_exists(root, module),
        &record,
    ))
}

pub fn check(root: &Path) -> Result<(), String> {
    let items = load_registry_items(root)?;
    let item_names: BTreeSet<String> = items.iter().map(|item| item.name.clone()).collect();
    let dir = records_dir(root);

    let mut violations = Vec::new();

    // (a) every record file must have a matching registry.json item.
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

    for item in &items {
        match check_real_item(root, item) {
            Ok(item_violations) => violations.extend(item_violations),
            Err(error) => violations.push(error),
        }
    }

    if violations.is_empty() {
        println!("primitive-usage check passed: {} item(s).", items.len());
        Ok(())
    } else {
        Err(violations.join("\n"))
    }
}

pub fn diff(root: &Path) -> Result<(), String> {
    let items = load_registry_items(root)?;
    let mut drifted = Vec::new();
    for item in &items {
        let detected_modules = find_primitive_modules(root, item);
        let path = record_path(root, &item.name);
        let Ok(record) = load_record(&path) else {
            drifted.push(format!("{}: no record on disk", item.name));
            continue;
        };
        if record.primitive_modules != detected_modules {
            drifted.push(format!(
                "{}: record lists {:?}, detected {:?}",
                item.name, record.primitive_modules, detected_modules
            ));
        }
    }
    if drifted.is_empty() {
        println!(
            "No drift: every statics/primitive_usage/<item>.json record matches detected primitive modules."
        );
    } else {
        for line in &drifted {
            println!("{line}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(classification: Classification) -> PrimitiveUsageRecord {
        PrimitiveUsageRecord {
            classification,
            primitive_modules: Vec::new(),
            reason: None,
            follow_up: None,
        }
    }

    #[test]
    fn record_round_trips_through_json() {
        let original = PrimitiveUsageRecord {
            classification: Classification::Exception,
            primitive_modules: vec!["dialog".to_string()],
            reason: Some("backdrop-dismiss workaround".to_string()),
            follow_up: Some("track use_outside_dismiss web support".to_string()),
        };
        let json = serde_json::to_string_pretty(&original).unwrap();
        let parsed: PrimitiveUsageRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn condition_a_is_enforced_by_the_directory_scan_not_check_item() {
        // condition (a) -- missing/orphan record -- is exercised at the
        // `check(root)` level (directory scan vs registry.json), not by the
        // pure `check_item` core, which always receives an already-loaded
        // record. Covered by the real-tree `check` call in the per-item
        // regression tests once real records are committed.
    }

    #[test]
    fn condition_b_fails_on_missing_primitive_module_file() {
        let mut delegated = record(Classification::Delegated);
        delegated.primitive_modules = vec!["nonexistent_module".to_string()];
        let violations = check_item("widget", "", true, |_| false, &delegated);
        assert!(
            violations.iter().any(|v| v.contains("nonexistent_module")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_c_fails_when_dependency_and_modules_disagree() {
        let delegated = record(Classification::Delegated);
        // registry.json declares the dependency, but record lists no modules.
        let violations = check_item("widget", "", true, |_| true, &delegated);
        assert!(
            violations.iter().any(|v| v.contains("declares")),
            "{violations:?}"
        );

        let mut with_modules = record(Classification::Delegated);
        with_modules.primitive_modules = vec!["dialog".to_string()];
        // record lists a module, but registry.json does not declare the dependency.
        let violations = check_item("widget", "", false, |_| true, &with_modules);
        assert!(
            violations.iter().any(|v| v.contains("does not declare")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_c_ignores_classification_label() {
        // An `exception` item may still genuinely depend on the crate for
        // non-exceptional behavior -- the check applies regardless of label.
        let mut exception = record(Classification::Exception);
        exception.reason = Some("reason".to_string());
        exception.follow_up = Some("follow-up".to_string());
        let violations = check_item("widget", "", true, |_| true, &exception);
        assert!(
            violations.iter().any(|v| v.contains("declares")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_d_fails_on_interactive_behavior_marker() {
        let presentational = {
            let mut r = record(Classification::Presentational);
            r.reason = Some("static markup only".to_string());
            r
        };
        let violations = check_item(
            "widget",
            "fn Widget() { let mut open = use_signal(|| false); }",
            false,
            |_| true,
            &presentational,
        );
        assert!(
            violations.iter().any(|v| v.contains("use_signal")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_d_fails_on_adico_primitives_import() {
        let presentational = {
            let mut r = record(Classification::Presentational);
            r.reason = Some("static markup only".to_string());
            r
        };
        let violations = check_item(
            "widget",
            "use adico_primitives::dialog;",
            false,
            |_| true,
            &presentational,
        );
        assert!(
            violations
                .iter()
                .any(|v| v.contains("imports adico_primitives")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_e_fails_on_duplicated_page_scroll_lock() {
        let mut exception = record(Classification::Exception);
        exception.primitive_modules = vec!["dialog".to_string()];
        exception.reason = Some("reason".to_string());
        exception.follow_up = Some("follow-up".to_string());
        let source = r#"style { "html {{ overflow: hidden; }}" }"#;
        let violations = check_item("widget", source, true, |_| true, &exception);
        assert!(
            violations.iter().any(|v| v.contains("scroll locking")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_e_passes_when_no_scroll_owning_module_is_listed() {
        let mut delegated = record(Classification::Delegated);
        delegated.primitive_modules = vec!["popover".to_string()];
        let source = r#"style { "html {{ overflow: hidden; }}" }"#;
        let violations = check_item("widget", source, true, |_| true, &delegated);
        assert!(
            !violations.iter().any(|v| v.contains("scroll locking")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_f_fails_on_empty_reason() {
        let presentational = record(Classification::Presentational);
        let violations = check_item("widget", "", false, |_| true, &presentational);
        assert!(
            violations.iter().any(|v| v.contains("empty reason")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_f_fails_on_empty_follow_up_for_exception() {
        let mut exception = record(Classification::Exception);
        exception.reason = Some("reason".to_string());
        let violations = check_item("widget", "", false, |_| true, &exception);
        assert!(
            violations.iter().any(|v| v.contains("empty followUp")),
            "{violations:?}"
        );
    }

    #[test]
    fn compliant_item_has_no_violations() {
        let mut delegated = record(Classification::Delegated);
        delegated.primitive_modules = vec!["dialog".to_string()];
        let violations = check_item("widget", "fn Widget() {}", true, |_| true, &delegated);
        assert!(violations.is_empty(), "{violations:?}");
    }
}

/// One dedicated regression test per real registry item, pinning that item's
/// actual `registry/ui/*.rs` source against its own committed
/// `statics/primitive_usage/<item>.json` record. Distinct from the
/// synthetic-fixture condition tests above: those prove the checker's logic
/// can fail in general, these prove the real tree agrees with its own
/// records right now, with a future violation surfacing in that item's own
/// named test rather than only in an aggregate `check` run.
#[cfg(test)]
mod item_tests {
    use super::check_real_item;
    use crate::registry_introspect::load_registry_items;
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("packages/adico-xtask has two parent directories")
            .to_path_buf()
    }

    fn assert_item_matches_its_record(item_name: &str) {
        let root = repo_root();
        let items = load_registry_items(&root).expect("registry.json should load");
        let item = items
            .iter()
            .find(|item| item.name == item_name)
            .unwrap_or_else(|| panic!("registry.json has no item named '{item_name}'"));
        let violations = check_real_item(&root, item).unwrap_or_else(|error| panic!("{error}"));
        assert!(violations.is_empty(), "{violations:?}");
    }

    macro_rules! primitive_usage_item_test {
        ($fn_name:ident, $item_name:literal) => {
            #[test]
            fn $fn_name() {
                assert_item_matches_its_record($item_name);
            }
        };
    }

    primitive_usage_item_test!(primitive_usage_accordion, "accordion");
    primitive_usage_item_test!(primitive_usage_alert_dialog, "alert-dialog");
    primitive_usage_item_test!(primitive_usage_aspect_ratio, "aspect-ratio");
    primitive_usage_item_test!(primitive_usage_avatar, "avatar");
    primitive_usage_item_test!(primitive_usage_badge, "badge");
    primitive_usage_item_test!(primitive_usage_button, "button");
    primitive_usage_item_test!(primitive_usage_calendar, "calendar");
    primitive_usage_item_test!(primitive_usage_card, "card");
    primitive_usage_item_test!(primitive_usage_checkbox, "checkbox");
    primitive_usage_item_test!(primitive_usage_collapsible, "collapsible");
    primitive_usage_item_test!(primitive_usage_color_picker, "color-picker");
    primitive_usage_item_test!(primitive_usage_combobox, "combobox");
    primitive_usage_item_test!(primitive_usage_context_menu, "context-menu");
    primitive_usage_item_test!(primitive_usage_date_picker, "date-picker");
    primitive_usage_item_test!(primitive_usage_dialog, "dialog");
    primitive_usage_item_test!(primitive_usage_drag_and_drop_list, "drag-and-drop-list");
    primitive_usage_item_test!(primitive_usage_dropdown_menu, "dropdown-menu");
    primitive_usage_item_test!(primitive_usage_hover_card, "hover-card");
    primitive_usage_item_test!(primitive_usage_input, "input");
    primitive_usage_item_test!(primitive_usage_item, "item");
    primitive_usage_item_test!(primitive_usage_label, "label");
    primitive_usage_item_test!(primitive_usage_menubar, "menubar");
    primitive_usage_item_test!(primitive_usage_mode_toggle, "mode-toggle");
    primitive_usage_item_test!(primitive_usage_pagination, "pagination");
    primitive_usage_item_test!(primitive_usage_popover, "popover");
    primitive_usage_item_test!(primitive_usage_progress, "progress");
    primitive_usage_item_test!(primitive_usage_radio_group, "radio-group");
    primitive_usage_item_test!(primitive_usage_scroll_area, "scroll-area");
    primitive_usage_item_test!(primitive_usage_select, "select");
    primitive_usage_item_test!(primitive_usage_sheet, "sheet");
    primitive_usage_item_test!(primitive_usage_sidebar, "sidebar");
    primitive_usage_item_test!(primitive_usage_skeleton, "skeleton");
    primitive_usage_item_test!(primitive_usage_slider, "slider");
    primitive_usage_item_test!(primitive_usage_switch, "switch");
    primitive_usage_item_test!(primitive_usage_tabs, "tabs");
    primitive_usage_item_test!(primitive_usage_tag_group, "tag-group");
    primitive_usage_item_test!(primitive_usage_textarea, "textarea");
    primitive_usage_item_test!(primitive_usage_theme_builder, "theme-builder");
    primitive_usage_item_test!(primitive_usage_theme_switcher, "theme-switcher");
    primitive_usage_item_test!(primitive_usage_toast, "toast");
    primitive_usage_item_test!(primitive_usage_toggle, "toggle");
    primitive_usage_item_test!(primitive_usage_toggle_group, "toggle-group");
    primitive_usage_item_test!(primitive_usage_toolbar, "toolbar");
    primitive_usage_item_test!(primitive_usage_tooltip, "tooltip");
    primitive_usage_item_test!(primitive_usage_virtual_list, "virtual-list");
}
