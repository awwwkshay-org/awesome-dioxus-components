//! `cargo xtask styling-usage sync|check|diff`: per-item styling
//! classification for every `registry:ui`/`registry:component` item, sibling
//! to `primitive_usage.rs` and built to the exact same shape (one record per
//! item under `statics/styling_usage/<item-name>.json`, `sync`/`check`/`diff`,
//! one dedicated regression test per item, CI-gated). Behavior-ownership and
//! styling compliance are independent axes -- see `design.md`'s Decisions
//! section for why this is a separate module rather than a third field on
//! `primitive_usage.rs`'s records.
//!
//! Every item declares two independent booleans:
//! - `tailwindOnly`: styles exclusively through static Tailwind utility
//!   classes, with any exception (a genuinely runtime-computed value that
//!   cannot be a static class) recorded in `styleException`.
//! - `tokenCompliant`: every themable color is a semantic design token, with
//!   any exception (a legitimate non-token color use) recorded in
//!   `colorException`.
//!
//! Plus `inspiredBy`/`inspirationNote`, recording what the item was checked
//! against during the dual-reference styling audit (shadcn and/or
//! dioxus-components) and what, if anything, was corrected as a result.
//!
//! Runs fully offline: no `statics/catalogs/*.json` read, no network access.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::registry_introspect::{load_registry_items, registry_root};

/// Tailwind's default-palette color classes flagged by the token-compliance
/// check. Not exhaustive of every shade, but the common families used
/// anywhere in this codebase's source -- extend this list, don't rewrite the
/// detection strategy, if a new one shows up.
const DEFAULT_PALETTE_PREFIXES: &[&str] = &[
    "bg-white",
    "text-white",
    "border-white",
    "bg-black",
    "text-black",
    "border-black",
    "bg-slate-",
    "text-slate-",
    "border-slate-",
    "bg-gray-",
    "text-gray-",
    "border-gray-",
    "bg-blue-",
    "text-blue-",
    "border-blue-",
    "bg-red-",
    "text-red-",
    "border-red-",
    "bg-green-",
    "text-green-",
    "border-green-",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct StyleException {
    pub description: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ColorException {
    pub value: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InspirationSource {
    Shadcn,
    DioxusComponents,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StylingUsageRecord {
    pub tailwind_only: bool,
    #[serde(default)]
    pub style_exception: Vec<StyleException>,
    pub token_compliant: bool,
    #[serde(default)]
    pub color_exception: Vec<ColorException>,
    #[serde(default)]
    pub inspired_by: Vec<InspirationSource>,
    #[serde(default)]
    pub inspiration_note: String,
}

fn records_dir(root: &Path) -> PathBuf {
    root.join("statics/styling_usage")
}

fn record_path(root: &Path, item_name: &str) -> PathBuf {
    records_dir(root).join(format!("{item_name}.json"))
}

fn load_record(path: &Path) -> Result<StylingUsageRecord, String> {
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

/// Whether the source contains a raw `style { ... }` block or `style:`
/// attribute at all. Callers combine this with an item's recorded
/// exceptions -- this function alone cannot tell a dynamic value from a
/// static one, it only detects the construct's presence (matching
/// `design.md`'s "narrower and different" scope for what the check verifies).
fn contains_raw_style_construct(source: &str) -> bool {
    source.contains("style {") || source.contains("style:")
}

/// Whether the source contains a raw hex/rgb color literal or one of the
/// known Tailwind default-palette color-class prefixes.
fn contains_non_token_color(source: &str) -> bool {
    if source.contains("#")
        && source
            .lines()
            .any(|line| line.contains('#') && line.contains('"'))
    {
        // A conservative, low-noise signal: a hex literal inside a quoted
        // string on the same line (class list or inline color value).
        for line in source.lines() {
            if line.contains('"') && line.contains('#') {
                let after_hash = line.split_once('#').map(|(_, rest)| rest).unwrap_or("");
                if after_hash.chars().take(6).all(|c| c.is_ascii_hexdigit())
                    && after_hash.chars().take(3).count() >= 3
                {
                    return true;
                }
            }
        }
    }
    DEFAULT_PALETTE_PREFIXES
        .iter()
        .any(|prefix| source.contains(prefix))
}

// --- Pure, testable core ----------------------------------------------------

fn check_item(item_name: &str, source: &str, record: &StylingUsageRecord) -> Vec<String> {
    let mut violations = Vec::new();

    if record.tailwind_only
        && contains_raw_style_construct(source)
        && record.style_exception.is_empty()
    {
        violations.push(format!(
            "{item_name}: classified tailwindOnly but source contains a raw style construct with no matching styleException"
        ));
    }

    if record.token_compliant
        && contains_non_token_color(source)
        && record.color_exception.is_empty()
    {
        violations.push(format!(
            "{item_name}: classified tokenCompliant but source contains a non-token color with no matching colorException"
        ));
    }

    for exception in &record.style_exception {
        if exception.reason.trim().is_empty() {
            violations.push(format!(
                "{item_name}: styleException '{}' has an empty reason",
                exception.description
            ));
        }
    }
    for exception in &record.color_exception {
        if exception.reason.trim().is_empty() {
            violations.push(format!(
                "{item_name}: colorException '{}' has an empty reason",
                exception.value
            ));
        }
    }

    violations
}

// --- sync / check / diff ----------------------------------------------------

pub fn sync(root: &Path) -> Result<(), String> {
    let dir = records_dir(root);
    fs::create_dir_all(&dir)
        .map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    let items = load_registry_items(root)?;
    let mut written = 0usize;
    for item in &items {
        let source = read_item_source(root, item);
        let path = record_path(root, &item.name);
        let existing = load_record(&path).ok();
        let record = match existing {
            Some(record) => record,
            None => StylingUsageRecord {
                tailwind_only: !contains_raw_style_construct(&source),
                style_exception: Vec::new(),
                token_compliant: !contains_non_token_color(&source),
                color_exception: Vec::new(),
                inspired_by: Vec::new(),
                inspiration_note: String::new(),
            },
        };
        let payload = serde_json::to_string_pretty(&record)
            .map_err(|error| format!("cannot serialize record for {}: {error}", item.name))?;
        fs::write(&path, format!("{payload}\n"))
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        written += 1;
    }
    println!("Synced {written} statics/styling_usage/<item>.json record(s).");
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

    for item in &items {
        let path = record_path(root, &item.name);
        let Ok(record) = load_record(&path) else {
            violations.push(format!(
                "{}: no styling-usage record (expected {})",
                item.name,
                path.display()
            ));
            continue;
        };
        let source = read_item_source(root, item);
        violations.extend(check_item(&item.name, &source, &record));
    }

    if violations.is_empty() {
        println!("styling-usage check passed: {} item(s).", items.len());
        Ok(())
    } else {
        Err(violations.join("\n"))
    }
}

pub fn diff(root: &Path) -> Result<(), String> {
    let items = load_registry_items(root)?;
    let mut drifted = Vec::new();
    for item in &items {
        let source = read_item_source(root, item);
        let path = record_path(root, &item.name);
        let Ok(record) = load_record(&path) else {
            drifted.push(format!("{}: no record on disk", item.name));
            continue;
        };
        let detected_has_raw_style = contains_raw_style_construct(&source);
        let recorded_tailwind_only_without_exception =
            record.tailwind_only && record.style_exception.is_empty();
        if detected_has_raw_style && recorded_tailwind_only_without_exception {
            drifted.push(format!(
                "{}: record says tailwindOnly with no exception, but source now contains a raw style construct",
                item.name
            ));
        }
        let detected_has_non_token_color = contains_non_token_color(&source);
        let recorded_token_compliant_without_exception =
            record.token_compliant && record.color_exception.is_empty();
        if detected_has_non_token_color && recorded_token_compliant_without_exception {
            drifted.push(format!(
                "{}: record says tokenCompliant with no exception, but source now contains a non-token color",
                item.name
            ));
        }
    }
    if drifted.is_empty() {
        println!(
            "No drift: every statics/styling_usage/<item>.json record matches detected source state."
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

    fn compliant_record() -> StylingUsageRecord {
        StylingUsageRecord {
            tailwind_only: true,
            style_exception: Vec::new(),
            token_compliant: true,
            color_exception: Vec::new(),
            inspired_by: Vec::new(),
            inspiration_note: String::new(),
        }
    }

    #[test]
    fn record_round_trips_through_json() {
        let original = StylingUsageRecord {
            tailwind_only: false,
            style_exception: vec![StyleException {
                description: "computed indicator width".to_string(),
                reason: "runtime-computed percentage".to_string(),
            }],
            token_compliant: false,
            color_exception: vec![ColorException {
                value: "text-white".to_string(),
                reason: "matches upstream shadcn destructive variant".to_string(),
            }],
            inspired_by: vec![
                InspirationSource::Shadcn,
                InspirationSource::DioxusComponents,
            ],
            inspiration_note: "matches upstream".to_string(),
        };
        let json = serde_json::to_string_pretty(&original).unwrap();
        let parsed: StylingUsageRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn condition_b_fails_on_uncovered_raw_style() {
        let record = compliant_record();
        let violations = check_item(
            "widget",
            r#"style: "position: fixed; z-index: 50;","#,
            &record,
        );
        assert!(
            violations.iter().any(|v| v.contains("raw style construct")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_b_passes_when_style_exception_recorded() {
        let mut record = compliant_record();
        record.style_exception.push(StyleException {
            description: "computed width".to_string(),
            reason: "runtime-computed value".to_string(),
        });
        let violations = check_item("widget", r#"style: "width: {pct}%;","#, &record);
        assert!(
            !violations.iter().any(|v| v.contains("raw style construct")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_c_fails_on_uncovered_non_token_color() {
        let record = compliant_record();
        let violations = check_item("widget", r#"class: "bg-slate-500 rounded","#, &record);
        assert!(
            violations.iter().any(|v| v.contains("non-token color")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_c_passes_when_color_exception_recorded() {
        let mut record = compliant_record();
        record.color_exception.push(ColorException {
            value: "bg-black/80".to_string(),
            reason: "matches upstream shadcn overlay".to_string(),
        });
        let violations = check_item("widget", r#"class: "bg-black/80","#, &record);
        assert!(
            !violations.iter().any(|v| v.contains("non-token color")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_d_fails_on_empty_style_exception_reason() {
        let mut record = compliant_record();
        record.style_exception.push(StyleException {
            description: "computed width".to_string(),
            reason: String::new(),
        });
        let violations = check_item("widget", "", &record);
        assert!(
            violations.iter().any(|v| v.contains("empty reason")),
            "{violations:?}"
        );
    }

    #[test]
    fn condition_d_fails_on_empty_color_exception_reason() {
        let mut record = compliant_record();
        record.color_exception.push(ColorException {
            value: "text-white".to_string(),
            reason: String::new(),
        });
        let violations = check_item("widget", "", &record);
        assert!(
            violations.iter().any(|v| v.contains("empty reason")),
            "{violations:?}"
        );
    }

    #[test]
    fn compliant_item_has_no_violations() {
        let record = compliant_record();
        let violations = check_item("widget", r#"class: "bg-primary text-foreground","#, &record);
        assert!(violations.is_empty(), "{violations:?}");
    }
}
