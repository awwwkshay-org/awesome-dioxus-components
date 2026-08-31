//! `catalog fetch dioxus-primitives`: inventory + props for
//! `DioxusLabs/dioxus-components`'s `primitives/src/` tree, one entry per
//! top-level module (`dialog.rs`, `combobox/`, ...). This is the fork
//! origin adico-primitives compares itself against on the
//! `primitive-compat` axis alongside Base UI.

use std::path::Path;

use crate::rust_introspect;

use super::dioxus_shared;
use super::schema::{CatalogEntry, CatalogSnapshot};

pub fn fetch(revision: Option<&str>) -> Result<CatalogSnapshot, String> {
    let (sha, _temp_dir, extracted_root) = dioxus_shared::resolve_and_fetch(revision)?;
    let primitives_src = extracted_root.join("primitives/src");
    let entries = build_entries(&primitives_src)?;

    Ok(CatalogSnapshot {
        axis: "dioxus-primitives".to_string(),
        source: format!("{}/tree/{sha}/primitives/src", dioxus_shared::source_url()),
        revision: sha,
        refreshed_at: crate::today(),
        entries,
    })
}

fn build_entries(primitives_src: &Path) -> Result<Vec<CatalogEntry>, String> {
    let mut modules = Vec::new();
    for entry in std::fs::read_dir(primitives_src)
        .map_err(|error| format!("cannot read {}: {error}", primitives_src.display()))?
    {
        let entry = entry.map_err(|error| format!("cannot read directory entry: {error}"))?;
        let path = entry.path();
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if path.is_dir() {
            if name != "js" && name != "ts" {
                modules.push((name.to_string(), path));
            }
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") && name != "lib" {
            modules.push((name.to_string(), path));
        }
    }
    modules.sort_by(|left, right| left.0.cmp(&right.0));

    Ok(modules
        .into_iter()
        .map(|(module, path)| {
            let introspection = if path.is_dir() {
                rust_introspect::introspect_directory(&path)
            } else {
                rust_introspect::introspect_file(&path)
            };
            CatalogEntry {
                id: module.replace('_', "-"),
                name: module.clone(),
                parts: dioxus_shared::parts_from_introspection(&module, &introspection),
            }
        })
        .collect())
}
