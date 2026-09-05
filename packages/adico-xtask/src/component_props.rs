//! `cargo xtask component-props sync|check|diff`: emits the same
//! introspected prop data `packages/adico-xtask/src/playground_controls.rs`
//! already extracts (`FileIntrospection`/`PropField`, both already
//! `Serialize`) as one committed JSON file, `statics/component_props.json`,
//! consumed by `apps/docs` at build time to render a per-component props
//! table -- proving the introspected data is genuinely reusable, not
//! playground-coupled. See `design.md`'s D5 decision.
//!
//! Reads `registry/ui/*.rs` (the canonical source `apps/docs` already reads
//! `registry/registry.json` from), not `apps/playground`'s installed copy,
//! so this has no dependency on the playground app existing at all. Runs
//! fully offline.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::rust_introspect::{PropField, introspect_file};

fn registry_ui_dir(root: &Path) -> PathBuf {
    root.join("registry/ui")
}

fn output_path(root: &Path) -> PathBuf {
    root.join("statics/component_props.json")
}

/// One registry item's components, each with their own declared prop
/// fields, in file order for `components` and declaration order for each
/// component's own fields.
type ItemComponents = BTreeMap<String, Vec<PropField>>;

fn registry_ui_items(root: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let dir = registry_ui_dir(root);
    let mut items = Vec::new();
    for entry in
        fs::read_dir(&dir).map_err(|error| format!("cannot read {}: {error}", dir.display()))?
    {
        let entry = entry.map_err(|error| format!("cannot read directory entry: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_string();
        if stem == "mod" {
            continue;
        }
        items.push((stem, path));
    }
    items.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(items)
}

/// Every component this item declares, keyed by component name, with its
/// own qualifying prop fields -- the same `<Name>Props`-or-`<Name>`
/// fallback lookup `prop_parity.rs`/`playground_controls.rs` already use.
fn item_components(item_stem: &str, path: &Path) -> ItemComponents {
    let introspection = introspect_file(path);
    let mut components = ItemComponents::new();
    for component_name in &introspection.components {
        let fields = introspection
            .props
            .get(&format!("{component_name}Props"))
            .or_else(|| introspection.props.get(component_name));
        if let Some(fields) = fields {
            components.insert(component_name.clone(), clone_fields(fields));
        }
    }
    if components.is_empty() {
        eprintln!("{item_stem}: no components with any declared props found");
    }
    components
}

fn clone_fields(fields: &[PropField]) -> Vec<PropField> {
    fields
        .iter()
        .map(|field| PropField {
            name: field.name.clone(),
            type_name: field.type_name.clone(),
            default: field.default.clone(),
        })
        .collect()
}

fn build_manifest(root: &Path) -> Result<BTreeMap<String, ItemComponents>, String> {
    let items = registry_ui_items(root)?;
    let mut manifest = BTreeMap::new();
    for (stem, path) in &items {
        manifest.insert(stem.clone(), item_components(stem, path));
    }
    Ok(manifest)
}

fn render_manifest(root: &Path) -> Result<String, String> {
    let manifest = build_manifest(root)?;
    let mut json = serde_json::to_string_pretty(&manifest)
        .map_err(|error| format!("cannot serialize component-props manifest: {error}"))?;
    json.push('\n');
    Ok(json)
}

pub fn sync(root: &Path) -> Result<(), String> {
    let json = render_manifest(root)?;
    let path = output_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(&path, &json).map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    println!("Synced {}.", path.display());
    Ok(())
}

pub fn check(root: &Path) -> Result<(), String> {
    let expected = render_manifest(root)?;
    let path = output_path(root);
    match fs::read_to_string(&path) {
        Ok(actual) if actual == expected => {
            println!("component-props check passed: {}.", path.display());
            Ok(())
        }
        Ok(_) => Err(format!("{}: is stale", path.display())),
        Err(_) => Err(format!("{}: missing", path.display())),
    }
}

pub fn diff(root: &Path) -> Result<(), String> {
    let expected = render_manifest(root)?;
    let path = output_path(root);
    let actual = fs::read_to_string(&path).ok();
    if actual.as_deref() == Some(expected.as_str()) {
        println!("component-props diff: no changes.");
    } else if actual.is_none() {
        println!("component-props diff: would create {}", path.display());
    } else {
        println!("component-props diff: would update {}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extracts_a_struct_based_component_and_an_inline_one() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("widget.rs");
        let mut file = fs::File::create(&path).expect("create fixture");
        file.write_all(
            br#"
                #[derive(Props, Clone, PartialEq)]
                pub struct WidgetProps {
                    pub disabled: bool,
                    pub label: String,
                }

                #[component]
                pub fn Widget(props: WidgetProps) -> Element {
                    rsx! {}
                }

                #[component]
                pub fn WidgetIcon(name: String) -> Element {
                    rsx! {}
                }
            "#,
        )
        .expect("write fixture");

        let components = item_components("widget", &path);
        let widget_fields = components.get("Widget").expect("Widget found");
        assert_eq!(widget_fields.len(), 2);
        assert_eq!(widget_fields[0].name, "disabled");
        assert_eq!(widget_fields[0].type_name, "bool");

        let icon_fields = components.get("WidgetIcon").expect("WidgetIcon found");
        assert_eq!(icon_fields.len(), 1);
        assert_eq!(icon_fields[0].name, "name");
    }
}
