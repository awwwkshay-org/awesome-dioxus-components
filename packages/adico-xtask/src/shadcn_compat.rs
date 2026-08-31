//! `cargo xtask shadcn-compat sync|check`: regenerates
//! `registry/shadcn_compatibility.json`, a hooks-and-props-level snapshot of
//! what's actually built for every `registry:ui`/`registry:component` item
//! in `registry/registry.json`.
//!
//! This tracks current state only -- what's built, right now -- not a
//! multi-dimension completion ledger against an external catalog (that was
//! `parity.json`'s role; it and its `cargo xtask parity` command were
//! removed, see design.md §9's "Removed 2026-08-31" note). For each
//! registry item, this introspects the registry facade file(s) under
//! `registry/ui/` and the `adico-primitives` module(s) it composes (found
//! via the item's `moduleExports`/`use adico_primitives::` imports, not
//! guessed from naming), extracting each one's public component functions,
//! `#[derive(Props...)]` struct fields, and `use_*` hooks in use.

use std::fs;
use std::path::Path;

use adico_registry_core::{RegistryItemType, RegistryManifest};
use serde::Serialize;

use crate::rust_introspect::{self, FileIntrospection};

fn registry_root(root: &Path) -> std::path::PathBuf {
    root.join("registry")
}

fn output_path(root: &Path) -> std::path::PathBuf {
    registry_root(root).join("shadcn_compatibility.json")
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

fn build_document(root: &Path) -> Result<serde_json::Value, String> {
    let manifest_path = registry_root(root).join("registry.json");
    let manifest_contents = fs::read(&manifest_path)
        .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
    let manifest: RegistryManifest = serde_json::from_slice(&manifest_contents)
        .map_err(|error| format!("registry manifest is invalid: {error}"))?;

    let mut items: Vec<_> = manifest
        .items
        .iter()
        .filter(|item| {
            matches!(
                item.item_type,
                RegistryItemType::Ui | RegistryItemType::Component
            )
        })
        .collect();
    items.sort_by(|left, right| left.name.cmp(&right.name));

    let components: Vec<ComponentOutput> = items
        .into_iter()
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
        .collect();

    Ok(serde_json::json!({
        "$schema_note": "Fully generated by `cargo xtask shadcn-compat sync` from registry.json + live Rust source. This tracks current prop/hook shape only, not a completion ledger (parity.json was removed, see design.md §9).",
        "synced_at": today(),
        "generator": "cargo xtask shadcn-compat sync",
        "summary": {
            "total_components": components.len(),
        },
        "components": components,
    }))
}

pub fn sync(root: &Path) -> Result<(), String> {
    let document = build_document(root)?;
    let path = output_path(root);
    let payload = serde_json::to_string_pretty(&document)
        .map_err(|error| format!("cannot serialize shadcn_compatibility.json: {error}"))?;
    fs::write(&path, format!("{payload}\n"))
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    println!("Wrote {}", path.display());
    println!("  components: {}", document["summary"]["total_components"]);
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
        return Err("shadcn_compatibility.json is stale; run `cargo xtask shadcn-compat sync` to regenerate.".to_string());
    }
    println!("shadcn_compatibility.json is up to date.");
    Ok(())
}
