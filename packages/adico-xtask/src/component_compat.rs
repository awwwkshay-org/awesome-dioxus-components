//! `cargo xtask component-compat sync|check`: regenerates
//! `statics/component_compatibility.json` -- the registry layer's
//! compatibility against **both** of its upstream component catalogs
//! (`statics/catalogs/shadcn.json` and
//! `statics/catalogs/dioxus-components.json`) plus a hooks-and-props-level
//! snapshot of what's actually built for every `registry:ui`/
//! `registry:component` item in `registry/registry.json`.
//!
//! This tracks current state only -- what's built, right now, against which
//! upstream -- not a multi-dimension completion ledger against an external
//! catalog (that was `parity.json`'s role; it and its `cargo xtask parity`
//! command were removed, see design.md §9's "Removed 2026-08-31" note). For
//! each registry item, this introspects the registry facade file(s) under
//! `registry/ui/` and the `adico-primitives` module(s) it composes (found via
//! the item's `moduleExports`/`use adico_primitives::` imports, not guessed
//! from naming), extracting each one's public component functions,
//! `#[derive(Props...)]` struct fields, and `use_*` hooks in use.
//!
//! Both upstream catalogs are read from `statics/catalogs/<axis>.json`,
//! produced offline-reproducibly by `cargo xtask catalog fetch <axis>` (see
//! `crate::catalog`) -- this module never touches the network. It filters
//! `catalog::AXES` by [`crate::catalog::AxisKind::Component`] rather than
//! naming `shadcn`/`dioxus-components` in its iteration.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::catalog::{self, AxisKind, CatalogSnapshot};
use crate::registry_introspect::{find_primitive_modules, load_registry_items, registry_root};
use crate::rust_introspect::{self, FileIntrospection};

fn output_path(root: &Path) -> PathBuf {
    root.join("statics/component_compatibility.json")
}

#[derive(Serialize)]
struct RegistryFacadeOutput {
    files: Vec<String>,
    components: Vec<String>,
    props: serde_json::Value,
}

#[derive(Serialize)]
struct PrimitiveOutput {
    modules: Vec<String>,
    components: Vec<String>,
    props: serde_json::Value,
    hooks_used: Vec<String>,
}

#[derive(Serialize)]
struct ComponentOutput {
    name: String,
    registry_manifest_found: bool,
    registry_facade: Option<RegistryFacadeOutput>,
    adico_primitive: Option<PrimitiveOutput>,
}

fn introspect_primitive_modules(root: &Path, modules: &[String]) -> FileIntrospection {
    let src = root.join("packages/adico-primitives/src");
    let mut merged = FileIntrospection {
        exists: false,
        ..Default::default()
    };
    for module in modules {
        let single_file = src.join(format!("{module}.rs"));
        let module_dir = src.join(module);
        let introspection = if single_file.exists() {
            rust_introspect::introspect_file(&single_file)
        } else if module_dir.is_dir() {
            rust_introspect::introspect_directory(&module_dir)
        } else {
            continue;
        };
        merged.exists = true;
        merged.components.extend(introspection.components);
        merged.props.extend(introspection.props);
        merged.hooks_defined.extend(introspection.hooks_defined);
        merged.hooks_used.extend(introspection.hooks_used);
    }
    merged.hooks_used.extend(merged.hooks_defined.clone());
    merged.hooks_used.sort();
    merged.hooks_used.dedup();
    merged
}

fn build_registry_components(
    root: &Path,
    items: &[adico_registry_core::RegistryItem],
) -> Vec<ComponentOutput> {
    items
        .iter()
        .map(|item| {
            let facade_files: Vec<String> =
                item.files.iter().map(|file| file.source.clone()).collect();
            let mut facade_components = Vec::new();
            let mut facade_props = serde_json::Map::new();
            for file in &item.files {
                let introspection =
                    rust_introspect::introspect_file(&registry_root(root).join(&file.source));
                facade_components.extend(introspection.components);
                for (name, fields) in introspection.props {
                    facade_props.insert(name, serde_json::to_value(fields).unwrap());
                }
            }

            let modules = find_primitive_modules(root, item);
            let primitive_introspection = introspect_primitive_modules(root, &modules);

            ComponentOutput {
                name: item.name.clone(),
                registry_manifest_found: true,
                registry_facade: Some(RegistryFacadeOutput {
                    files: facade_files,
                    components: facade_components,
                    props: serde_json::Value::Object(facade_props),
                }),
                adico_primitive: Some(PrimitiveOutput {
                    modules,
                    components: primitive_introspection.components,
                    props: serde_json::to_value(&primitive_introspection.props).unwrap(),
                    hooks_used: primitive_introspection.hooks_used,
                }),
            }
        })
        .collect()
}

// --- Upstream catalog axes -------------------------------------------------
//
// Both `statics/catalogs/shadcn.json` and
// `statics/catalogs/dioxus-components.json` are revision-pinned offline
// snapshots produced by `cargo xtask catalog fetch <axis>`. Status per
// catalog entry is derived by id match against the installed registry
// items -- `built` if present, `not_started` otherwise. Only genuine
// exceptions (an entry that will never become a standalone registry item, or
// was deliberately excluded) get a hand-written note below; do not hand-list
// the full catalog.

#[derive(Serialize)]
struct CatalogEntryOutput {
    name: String,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    registry_item: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<&'static str>,
}

/// Hand-maintained exceptions for the shadcn axis: a catalog entry id
/// (kebab-case, as it appears in `statics/catalogs/shadcn.json`) mapped to
/// `(forced_status, note)`. `forced_status` overrides the derived
/// built/not_started status; leave it `None` to just attach a note.
const SHADCN_EXCEPTIONS: &[(&str, Option<&str>, &str)] = &[(
    "separator",
    Some("not_applicable"),
    "No standalone shadcn-equivalent distributable: the separator primitive alone has no meaningful UI beyond the primitive itself, so it is not installed as its own registry:ui item (see docs/adico/m3-acceptance.md).",
)];

/// Same shape as [`SHADCN_EXCEPTIONS`], for the dioxus-components axis.
const DIOXUS_COMPONENT_EXCEPTIONS: &[(&str, Option<&str>, &str)] = &[
    (
        "form",
        None,
        "Excluded from migration entirely; its only real content was a demo of a native <form> element plus the label primitive, both already covered elsewhere (see docs/adico/m3-acceptance.md).",
    ),
    (
        "navbar",
        None,
        "Out of M3 scope by its own classification (NEEDS_PARITY_UPDATES, not \"suitable for current reuse\"); see docs/adico/m3-acceptance.md.",
    ),
    (
        "separator",
        Some("not_applicable"),
        "No standalone distributable: the separator primitive alone has no meaningful UI beyond the primitive itself, so it is not installed as its own registry:ui item (see docs/adico/m3-acceptance.md).",
    ),
];

fn build_catalog_axis(
    snapshot: &CatalogSnapshot,
    registry_names: &std::collections::BTreeSet<String>,
    exceptions: &'static [(&'static str, Option<&'static str>, &'static str)],
) -> (Vec<CatalogEntryOutput>, [usize; 3]) {
    let mut counts = [0usize; 3]; // built, not_started, not_applicable
    let entries = snapshot
        .entries
        .iter()
        .map(|entry| {
            let slug = &entry.id;
            let exception = exceptions.iter().find(|(name, _, _)| name == slug);
            let registry_item = registry_names.contains(slug).then(|| slug.clone());
            let derived_status = if registry_item.is_some() {
                "built"
            } else {
                "not_started"
            };
            let status = exception
                .and_then(|(_, forced, _)| *forced)
                .unwrap_or(derived_status);
            match status {
                "built" => counts[0] += 1,
                "not_applicable" => counts[2] += 1,
                _ => counts[1] += 1,
            }
            CatalogEntryOutput {
                name: slug.clone(),
                status,
                registry_item,
                notes: exception.map(|(_, _, note)| *note),
            }
        })
        .collect();
    (entries, counts)
}

fn catalog_source_summary(snapshot: &CatalogSnapshot, axis_id: &str) -> serde_json::Value {
    serde_json::json!({
        "axis": snapshot.axis,
        "source": snapshot.source,
        "revision": snapshot.revision,
        "catalog_refreshed_at": snapshot.refreshed_at,
        "catalog_path": format!("statics/catalogs/{axis_id}.json"),
        "refresh_command": format!("cargo xtask catalog fetch {axis_id}"),
    })
}

/// Builds one upstream axis's full output section (source metadata, summary
/// counts, per-entry status). Deliberately axis-id-agnostic beyond looking
/// up an optional hand-exceptions table -- an axis with no entry in that
/// match still produces a complete, correctly-derived section, which is what
/// lets a newly registered axis work without editing this file's control
/// flow (see spec's "New upstream axes are addable without changing
/// compat-tooling internals").
fn build_axis_section(
    snapshot: &CatalogSnapshot,
    axis_id: &str,
    registry_names: &std::collections::BTreeSet<String>,
) -> serde_json::Value {
    let exceptions = match axis_id {
        "shadcn" => SHADCN_EXCEPTIONS,
        "dioxus-components" => DIOXUS_COMPONENT_EXCEPTIONS,
        _ => &[],
    };
    let (entries, counts) = build_catalog_axis(snapshot, registry_names, exceptions);
    let mut section = catalog_source_summary(snapshot, axis_id);
    if let Some(map) = section.as_object_mut() {
        map.insert(
            "summary".to_string(),
            serde_json::json!({
                "total_upstream_components": snapshot.entries.len(),
                "built": counts[0],
                "not_started": counts[1],
                "not_applicable": counts[2],
            }),
        );
        map.insert(
            "components".to_string(),
            serde_json::to_value(entries).unwrap(),
        );
    }
    section
}

fn build_document(root: &Path) -> Result<serde_json::Value, String> {
    let items = load_registry_items(root)?;
    let registry_names: std::collections::BTreeSet<String> =
        items.iter().map(|item| item.name.clone()).collect();
    let components = build_registry_components(root, &items);

    let mut document = serde_json::json!({
        "$schema_note": "Fully generated by `cargo xtask component-compat sync` from registry.json, statics/catalogs/shadcn.json, statics/catalogs/dioxus-components.json, and live Rust source. Hand-maintain only SHADCN_EXCEPTIONS/DIOXUS_COMPONENT_EXCEPTIONS in packages/adico-xtask/src/component_compat.rs; everything else is derived. This tracks current state only, not a completion ledger (parity.json was removed, see design.md §9).",
        "synced_at": crate::today(),
        "generator": "cargo xtask component-compat sync",
        "adico_registry": {
            "summary": { "total_components": components.len() },
            "components": components,
        },
    });

    for axis in catalog::axes_of_kind(AxisKind::Component) {
        let snapshot = catalog::read_snapshot(root, axis.id)?;
        let section = build_axis_section(&snapshot, axis.id, &registry_names);
        if let Some(map) = document.as_object_mut() {
            map.insert(axis.id.replace('-', "_"), section);
        }
    }

    Ok(document)
}

pub fn sync(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root.join("statics"))
        .map_err(|error| format!("cannot create statics/: {error}"))?;
    let document = build_document(root)?;
    let path = output_path(root);
    let payload = serde_json::to_string_pretty(&document)
        .map_err(|error| format!("cannot serialize component_compatibility.json: {error}"))?;
    fs::write(&path, format!("{payload}\n"))
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    println!("Wrote {}", path.display());
    println!(
        "  adico registry: {} components",
        document["adico_registry"]["summary"]["total_components"]
    );
    if let Some(shadcn) = document.get("shadcn").and_then(|axis| axis.get("summary")) {
        println!("  shadcn: {shadcn}");
    }
    if let Some(dioxus) = document
        .get("dioxus_components")
        .and_then(|axis| axis.get("summary"))
    {
        println!("  dioxus-components: {dioxus}");
    }
    Ok(())
}

pub fn check(root: &Path) -> Result<(), String> {
    let document = build_document(root)?;
    let path = output_path(root);
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let mut existing_value: serde_json::Value =
        serde_json::from_str(&existing).unwrap_or(serde_json::Value::Null);
    if let Some(obj) = existing_value.as_object_mut() {
        obj.remove("synced_at");
    }
    let mut comparable = document.clone();
    if let Some(obj) = comparable.as_object_mut() {
        obj.remove("synced_at");
    }
    if existing_value != comparable {
        return Err("component_compatibility.json is stale; run `cargo xtask component-compat sync` to regenerate.".to_string());
    }
    println!("component_compatibility.json is up to date.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::schema::{CatalogEntry, PartEntry, PropsSource};
    use std::collections::BTreeSet;

    fn fake_snapshot(axis: &str, ids: &[&str]) -> CatalogSnapshot {
        CatalogSnapshot {
            axis: axis.to_string(),
            source: "https://example.invalid".to_string(),
            revision: "test".to_string(),
            refreshed_at: "2026-08-31".to_string(),
            entries: ids
                .iter()
                .map(|id| CatalogEntry {
                    id: id.to_string(),
                    name: id.to_string(),
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
    fn separator_exception_survives_regardless_of_fetched_entries() {
        let registry_names: BTreeSet<String> = BTreeSet::new();
        let before = build_axis_section(
            &fake_snapshot("shadcn", &["separator"]),
            "shadcn",
            &registry_names,
        );
        let after = build_axis_section(
            &fake_snapshot("shadcn", &["separator", "new-thing"]),
            "shadcn",
            &registry_names,
        );
        assert_eq!(before["components"][0]["status"], "not_applicable");
        assert_eq!(after["components"][0]["status"], "not_applicable");
    }

    /// Proves a fifth, entirely unrecognized axis id is processed correctly
    /// by the same control flow as `shadcn`/`dioxus-components` -- no match
    /// arm, no special case, just the wildcard `_ => &[]` exceptions lookup.
    /// This is the mechanism that lets `build_document`'s
    /// `catalog::axes_of_kind(AxisKind::Component)` loop pick up a newly
    /// registered axis without any change to this file beyond registering
    /// it in `catalog::AXES`.
    #[test]
    fn unrecognized_axis_id_still_produces_a_valid_section() {
        let mut registry_names = BTreeSet::new();
        registry_names.insert("widget".to_string());
        let section = build_axis_section(
            &fake_snapshot("second-component-lib", &["widget", "gizmo"]),
            "second-component-lib",
            &registry_names,
        );
        assert_eq!(section["summary"]["total_upstream_components"], 2);
        assert_eq!(section["summary"]["built"], 1);
        assert_eq!(section["summary"]["not_started"], 1);
        assert_eq!(section["components"][0]["status"], "built");
        assert_eq!(section["components"][1]["status"], "not_started");
    }
}
