//! `catalog fetch dioxus-components`: inventory + composition + props for
//! `DioxusLabs/dioxus-components`'s styled `preview/src/components/` tree.
//!
//! Styled components here mostly take the underlying primitive's own
//! `#[derive(Props)]` type directly (`pub fn Dialog(props: DialogRootProps)`)
//! rather than declaring their own -- the Dioxus-side equivalent of shadcn's
//! `React.ComponentProps<typeof X>` passthrough. [`rust_introspect`] doesn't
//! record props for that shape (by design, see its `inline_component_props`
//! doc comment), so this module does its own light `syn` scan to recover the
//! passed-through type name and records it as `inherits_from` pointing at
//! the `dioxus-primitives` axis, rather than leaving it silently empty.

use std::collections::BTreeMap;
use std::path::Path;

use syn::{FnArg, Item, Pat, Type, Visibility};

use crate::rust_introspect;

use super::dioxus_shared;
use super::schema::{CatalogEntry, CatalogSnapshot, CompositionRef, PartEntry, PropsSource};

pub fn fetch(revision: Option<&str>) -> Result<CatalogSnapshot, String> {
    let (sha, _temp_dir, extracted_root) = dioxus_shared::resolve_and_fetch(revision)?;
    let components_root = extracted_root.join("preview/src/components");
    let entries = build_entries(&components_root)?;

    Ok(CatalogSnapshot {
        axis: "dioxus-components".to_string(),
        source: format!(
            "{}/tree/{sha}/preview/src/components",
            dioxus_shared::source_url()
        ),
        revision: sha,
        refreshed_at: crate::today(),
        entries,
    })
}

fn build_entries(components_root: &Path) -> Result<Vec<CatalogEntry>, String> {
    let mut directories = Vec::new();
    for entry in std::fs::read_dir(components_root)
        .map_err(|error| format!("cannot read {}: {error}", components_root.display()))?
    {
        let entry = entry.map_err(|error| format!("cannot read directory entry: {error}"))?;
        let path = entry.path();
        if path.is_dir()
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
        {
            directories.push((name.to_string(), path));
        }
    }
    directories.sort_by(|left, right| left.0.cmp(&right.0));

    Ok(directories
        .into_iter()
        .map(|(name, dir)| build_entry(&name, &dir))
        .collect())
}

fn build_entry(component_dir_name: &str, dir: &Path) -> CatalogEntry {
    let introspection = rust_introspect::introspect_directory(dir);
    let composed_modules = find_composed_primitive_modules(dir);
    let passthrough_types = find_props_passthrough(dir);

    let parts: Vec<PartEntry> = introspection
        .components
        .iter()
        .map(|component| {
            let part_id = dioxus_shared::part_id_for(component_dir_name, component);
            let props_source = introspection
                .props
                .get(&format!("{component}Props"))
                .map(|fields| PropsSource::Explicit {
                    props: fields
                        .iter()
                        .map(|field| super::schema::Prop {
                            name: field.name.clone(),
                            type_name: field.type_name.clone(),
                            default: field.default.clone(),
                            description: None,
                        })
                        .collect(),
                })
                .or_else(|| {
                    let passthrough_type = passthrough_types.get(component)?;
                    let primitive_module = composed_modules.first()?;
                    let base_type = passthrough_type
                        .strip_suffix("Props")
                        .unwrap_or(passthrough_type);
                    let primitive_part = dioxus_shared::part_id_for(primitive_module, base_type);
                    Some(PropsSource::InheritsFrom {
                        reference: format!("dioxus-primitives.{primitive_module}.{primitive_part}"),
                    })
                })
                .unwrap_or(PropsSource::Unavailable);

            let composition = composed_modules
                .first()
                .map(|module| {
                    let part = passthrough_types.get(component).map(|props_type| {
                        let base_type = props_type.strip_suffix("Props").unwrap_or(props_type);
                        dioxus_shared::part_id_for(module, base_type)
                    });
                    vec![CompositionRef {
                        axis: "dioxus-primitives".to_string(),
                        component: module.clone(),
                        part,
                    }]
                })
                .unwrap_or_default();

            PartEntry {
                id: part_id,
                composition,
                props_source,
            }
        })
        .collect();

    CatalogEntry {
        id: component_dir_name.replace('_', "-"),
        name: component_dir_name.to_string(),
        parts,
    }
}

/// Scans every `.rs` file in a styled component's directory for
/// `use dioxus_primitives::<module>` imports, identifying which primitive
/// module(s) it composes -- same technique `component_compat.rs` already
/// uses for adico's own `use adico_primitives::` imports.
fn find_composed_primitive_modules(dir: &Path) -> Vec<String> {
    let mut modules: Vec<String> = Vec::new();
    for path in rust_files(dir) {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        for line in source.lines() {
            if let Some(rest) = line.trim().strip_prefix("use dioxus_primitives::") {
                let module = rest
                    .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .next()
                    .unwrap_or("");
                if !module.is_empty() && module != "self" && !modules.contains(&module.to_string())
                {
                    modules.push(module.to_string());
                }
            }
        }
    }
    modules
}

/// Finds `pub fn Foo(props: SomeProps) -> Element` signatures -- the
/// passthrough shape `rust_introspect` deliberately does not record props
/// for -- mapping each component name to the passed-through props type.
fn find_props_passthrough(dir: &Path) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    for path in rust_files(dir) {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(file) = syn::parse_file(&source) else {
            continue;
        };
        for item in &file.items {
            let Item::Fn(item_fn) = item else { continue };
            if !matches!(item_fn.vis, Visibility::Public(_)) {
                continue;
            }
            let name = item_fn.sig.ident.to_string();
            if !name.starts_with(|c: char| c.is_ascii_uppercase()) {
                continue;
            }
            if item_fn.sig.inputs.len() != 1 {
                continue;
            }
            let FnArg::Typed(pat_type) = item_fn.sig.inputs.first().unwrap() else {
                continue;
            };
            let is_props_arg =
                matches!(&*pat_type.pat, Pat::Ident(ident) if ident.ident == "props");
            if !is_props_arg {
                continue;
            }
            if let Type::Path(type_path) = &*pat_type.ty
                && let Some(segment) = type_path.path.segments.last()
            {
                result.insert(name, segment.ident.to_string());
            }
        }
    }
    result
}

fn rust_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(dir, &mut files);
    files
}

fn collect_rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}
