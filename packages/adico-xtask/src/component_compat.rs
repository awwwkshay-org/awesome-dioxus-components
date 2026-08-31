//! `cargo xtask component-compat sync|check`: regenerates
//! `statics/component_compatibility.json` -- the registry layer's
//! compatibility against **both** of its upstream component catalogs
//! (`upstreams/shadcn/catalog.json` and
//! `upstreams/dioxus-components/catalog.json`'s `styledComponents`) plus a
//! hooks-and-props-level snapshot of what's actually built for every
//! `registry:ui`/`registry:component` item in `registry/registry.json`.
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
//! Both upstream catalogs are revision-pinned offline snapshots. There is a
//! refresh command for the dioxus-components one
//! (`cargo xtask upstream dioxus-components ...`); `upstreams/shadcn/catalog.json`
//! has no equivalent live fetcher yet -- that's a known gap, not solved here.

use std::fs;
use std::path::{Path, PathBuf};

use adico_registry_core::{RegistryItemType, RegistryManifest};
use serde::{Deserialize, Serialize};

use crate::rust_introspect::{self, FileIntrospection};

fn registry_root(root: &Path) -> PathBuf {
    root.join("registry")
}

fn output_path(root: &Path) -> PathBuf {
    root.join("statics/component_compatibility.json")
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

fn find_primitive_modules(root: &Path, item: &adico_registry_core::RegistryItem) -> Vec<String> {
    let mut modules: std::collections::BTreeSet<String> = item
        .module_exports
        .iter()
        .map(|export| export.module.clone())
        .collect();
    for file in &item.files {
        let path = registry_root(root).join(&file.source);
        if let Ok(source) = fs::read_to_string(&path) {
            for line in source.lines() {
                if let Some(rest) = line.trim().strip_prefix("use adico_primitives::") {
                    let module = rest
                        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .next()
                        .unwrap_or("");
                    if !module.is_empty() {
                        modules.insert(module.to_string());
                    }
                }
            }
        }
    }
    modules.into_iter().collect()
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

fn load_registry_items(root: &Path) -> Result<Vec<adico_registry_core::RegistryItem>, String> {
    let manifest_path = registry_root(root).join("registry.json");
    let manifest_contents = fs::read(&manifest_path)
        .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
    let manifest: RegistryManifest = serde_json::from_slice(&manifest_contents)
        .map_err(|error| format!("registry manifest is invalid: {error}"))?;

    let mut items: Vec<_> = manifest
        .items
        .into_iter()
        .filter(|item| {
            matches!(
                item.item_type,
                RegistryItemType::Ui | RegistryItemType::Component
            )
        })
        .collect();
    items.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(items)
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
// Both `upstreams/shadcn/catalog.json` and
// `upstreams/dioxus-components/catalog.json` are revision-pinned offline
// snapshots. Status per catalog entry is derived by slug match against the
// installed registry items -- `built` if present, `not_started` otherwise.
// Only genuine exceptions (an entry that will never become a standalone
// registry item, or was deliberately excluded) get a hand-written note below;
// do not hand-list the full catalog.

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShadcnCatalog {
    source: String,
    revision: String,
    refreshed_at: String,
    components: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DioxusComponentsCatalog {
    upstream: String,
    revision: String,
    refreshed_at: String,
    styled_components: Vec<String>,
}

#[derive(Serialize)]
struct CatalogEntryOutput {
    name: String,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    registry_item: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<&'static str>,
}

/// Hand-maintained exceptions for the shadcn axis: a catalog entry name
/// (kebab-case, as it appears in `upstreams/shadcn/catalog.json`) mapped to
/// `(forced_status, note)`. `forced_status` overrides the derived
/// built/not_started status; leave it `None` to just attach a note.
const SHADCN_EXCEPTIONS: &[(&str, Option<&str>, &str)] = &[(
    "separator",
    Some("not_applicable"),
    "No standalone shadcn-equivalent distributable: the separator primitive alone has no meaningful UI beyond the primitive itself, so it is not installed as its own registry:ui item (see docs/adico/m3-acceptance.md).",
)];

/// Same shape as [`SHADCN_EXCEPTIONS`], for the dioxus-components axis
/// (entries there are snake_case and normalized to kebab-case before
/// matching).
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

fn kebab_case(name: &str) -> String {
    name.replace('_', "-")
}

fn build_catalog_axis(
    catalog_names: &[String],
    normalize: impl Fn(&str) -> String,
    registry_names: &std::collections::BTreeSet<String>,
    exceptions: &'static [(&'static str, Option<&'static str>, &'static str)],
) -> (Vec<CatalogEntryOutput>, [usize; 3]) {
    let mut counts = [0usize; 3]; // built, not_started, not_applicable
    let entries = catalog_names
        .iter()
        .map(|raw_name| {
            let slug = normalize(raw_name);
            let exception = exceptions.iter().find(|(name, _, _)| *name == slug);
            let registry_item = registry_names.contains(&slug).then(|| slug.clone());
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
                name: slug,
                status,
                registry_item,
                notes: exception.map(|(_, _, note)| *note),
            }
        })
        .collect();
    (entries, counts)
}

fn read_shadcn_catalog(root: &Path) -> Result<ShadcnCatalog, String> {
    let path = root.join("upstreams/shadcn/catalog.json");
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))
}

fn read_dioxus_components_catalog(root: &Path) -> Result<DioxusComponentsCatalog, String> {
    let path = root.join("upstreams/dioxus-components/catalog.json");
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))
}

fn build_document(root: &Path) -> Result<serde_json::Value, String> {
    let items = load_registry_items(root)?;
    let registry_names: std::collections::BTreeSet<String> =
        items.iter().map(|item| item.name.clone()).collect();
    let components = build_registry_components(root, &items);

    let shadcn_catalog = read_shadcn_catalog(root)?;
    let (shadcn_entries, shadcn_counts) = build_catalog_axis(
        &shadcn_catalog.components,
        |name| name.to_string(),
        &registry_names,
        SHADCN_EXCEPTIONS,
    );

    let dioxus_catalog = read_dioxus_components_catalog(root)?;
    let (dioxus_entries, dioxus_counts) = build_catalog_axis(
        &dioxus_catalog.styled_components,
        kebab_case,
        &registry_names,
        DIOXUS_COMPONENT_EXCEPTIONS,
    );

    Ok(serde_json::json!({
        "$schema_note": "Fully generated by `cargo xtask component-compat sync` from registry.json, upstreams/shadcn/catalog.json, upstreams/dioxus-components/catalog.json, and live Rust source. Hand-maintain only SHADCN_EXCEPTIONS/DIOXUS_COMPONENT_EXCEPTIONS in packages/adico-xtask/src/component_compat.rs; everything else is derived. This tracks current state only, not a completion ledger (parity.json was removed, see design.md §9).",
        "synced_at": today(),
        "generator": "cargo xtask component-compat sync",
        "adico_registry": {
            "summary": { "total_components": components.len() },
            "components": components,
        },
        "shadcn": {
            "source": shadcn_catalog.source,
            "revision": shadcn_catalog.revision,
            "catalog_refreshed_at": shadcn_catalog.refreshed_at,
            "catalog_path": "upstreams/shadcn/catalog.json",
            "refresh_command": null,
            "summary": {
                "total_upstream_components": shadcn_catalog.components.len(),
                "built": shadcn_counts[0],
                "not_started": shadcn_counts[1],
                "not_applicable": shadcn_counts[2],
            },
            "components": shadcn_entries,
        },
        "dioxus_components": {
            "upstream": dioxus_catalog.upstream,
            "revision": dioxus_catalog.revision,
            "catalog_refreshed_at": dioxus_catalog.refreshed_at,
            "catalog_path": "upstreams/dioxus-components/catalog.json",
            "refresh_command": "cargo xtask upstream dioxus-components --source <local-clone> --refreshed-at <YYYY-MM-DD> --write",
            "summary": {
                "total_upstream_components": dioxus_catalog.styled_components.len(),
                "built": dioxus_counts[0],
                "not_started": dioxus_counts[1],
                "not_applicable": dioxus_counts[2],
            },
            "components": dioxus_entries,
        },
    }))
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
    println!("  shadcn: {}", document["shadcn"]["summary"]);
    println!(
        "  dioxus-components: {}",
        document["dioxus_components"]["summary"]
    );
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
