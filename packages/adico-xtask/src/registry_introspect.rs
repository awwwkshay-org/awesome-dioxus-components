//! Shared registry-item introspection used by every `adico-xtask` command
//! that needs to know which `adico_primitives` module(s) a `registry:ui`/
//! `registry:component` item actually uses. Extracted from
//! `component_compat.rs` (originally `find_primitive_modules`) so
//! `primitive_usage.rs` and `styling_usage.rs` share the exact same
//! detection logic instead of duplicating it.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use adico_registry_core::{RegistryItemType, RegistryManifest};

pub fn registry_root(root: &Path) -> PathBuf {
    root.join("registry")
}

/// Loads every `registry:ui`/`registry:component` item from
/// `registry/registry.json`, sorted by name.
pub fn load_registry_items(root: &Path) -> Result<Vec<adico_registry_core::RegistryItem>, String> {
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

/// Resolves an item's actual `adico_primitives` module usage by scanning its
/// source file(s) for `use adico_primitives::...`/`pub use adico_primitives::...`
/// statements (reassembled across lines, since brace-grouped imports commonly
/// span multiple lines) and extracting each statement's first path segment
/// -- but only when that segment is itself followed by `::` (i.e. accessed as
/// `module::Item`, this codebase's consistent convention for a genuine
/// submodule). A bare crate-root import with no further path segment (a
/// re-exported hook like `use_controlled`, or a re-exported type like
/// `ContentAlign`) is deliberately not treated as a module name.
///
/// Does NOT consult `registry.json`'s `moduleExports` metadata: that field
/// describes where an item's own generated Rust module lives in a consumer's
/// installed `ui::` tree (installation bookkeeping), not whether the item
/// imports anything from `adico_primitives` -- for a presentational item like
/// `button`, `moduleExports` names `button` for exactly that reason, which is
/// not an `adico_primitives::button` module (no such module exists).
pub fn find_primitive_modules(
    root: &Path,
    item: &adico_registry_core::RegistryItem,
) -> Vec<String> {
    let mut modules = BTreeSet::new();
    for file in &item.files {
        let path = registry_root(root).join(&file.source);
        if let Ok(source) = fs::read_to_string(&path) {
            collect_primitive_modules_from_source(&source, &mut modules);
        }
    }
    modules.into_iter().collect()
}

fn collect_primitive_modules_from_source(source: &str, modules: &mut BTreeSet<String>) {
    let mut statement = String::new();
    let mut collecting = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if !collecting {
            if trimmed.starts_with("use adico_primitives::")
                || trimmed.starts_with("pub use adico_primitives::")
            {
                collecting = true;
                statement.clear();
            } else {
                continue;
            }
        }
        statement.push_str(trimmed);
        statement.push(' ');
        if trimmed.ends_with(';') {
            extract_modules_from_use_statement(&statement, modules);
            collecting = false;
        }
    }
}

fn extract_modules_from_use_statement(statement: &str, modules: &mut BTreeSet<String>) {
    let Some((_, rest)) = statement.split_once("adico_primitives::") else {
        return;
    };
    let rest = rest.trim_end_matches(';').trim();
    let rest = rest.strip_prefix('{').unwrap_or(rest);
    let rest = rest.strip_suffix('}').unwrap_or(rest).trim();
    for segment in rest.split(',') {
        let segment = segment.trim();
        let Some((candidate, _)) = segment.split_once("::") else {
            // No further path segment -- a bare crate-root import (hook or
            // re-exported type), not a submodule reference.
            continue;
        };
        let module: String = candidate
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !module.is_empty() {
            modules.insert(module);
        }
    }
}

/// Whether an item's `registry.json` entry declares the `adico-primitives`
/// crate dependency.
pub fn declares_adico_primitives_dependency(item: &adico_registry_core::RegistryItem) -> bool {
    item.cargo_dependencies
        .iter()
        .any(|dependency| dependency.crate_name == "adico-primitives")
}

/// Whether `module` is a genuinely importable path segment of the
/// `adico_primitives` crate: either a file/directory under
/// `packages/adico-primitives/src/`, or a top-level `pub use ... as <module>`
/// re-export alias declared in `lib.rs` (the one current example being
/// `pub use dioxus_icons::lucide as icons;`).
pub fn primitive_module_exists(root: &Path, module: &str) -> bool {
    let src = root.join("packages/adico-primitives/src");
    if src.join(format!("{module}.rs")).exists() || src.join(module).is_dir() {
        return true;
    }
    let lib_rs = src.join("lib.rs");
    let Ok(contents) = fs::read_to_string(&lib_rs) else {
        return false;
    };
    contents.contains(&format!("as {module};")) || contents.contains(&format!("mod {module};"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_bare_crate_root_imports() {
        let mut modules = BTreeSet::new();
        collect_primitive_modules_from_source(
            "use adico_primitives::{separator::Separator as SeparatorPrimitive, use_controlled};\n",
            &mut modules,
        );
        assert_eq!(
            modules,
            BTreeSet::from(["separator".to_string()]),
            "use_controlled has no further path segment and must not be treated as a module"
        );
    }

    #[test]
    fn ignores_bare_re_exported_types() {
        let mut modules = BTreeSet::new();
        collect_primitive_modules_from_source(
            "pub use adico_primitives::{ContentAlign, ContentSide};\n",
            &mut modules,
        );
        assert!(modules.is_empty(), "{modules:?}");
    }

    #[test]
    fn extracts_module_from_multiline_brace_import() {
        let mut modules = BTreeSet::new();
        collect_primitive_modules_from_source(
            "use adico_primitives::popover::{\n    PopoverContent as PopoverPrimitiveContent, PopoverTrigger as PopoverPrimitiveTrigger,\n};\n",
            &mut modules,
        );
        assert_eq!(modules, BTreeSet::from(["popover".to_string()]));
    }

    #[test]
    fn extracts_module_from_pub_use_single_line() {
        let mut modules = BTreeSet::new();
        collect_primitive_modules_from_source(
            "pub use adico_primitives::hover_card::HoverCard;\n",
            &mut modules,
        );
        assert_eq!(modules, BTreeSet::from(["hover_card".to_string()]));
    }

    #[test]
    fn extracts_every_module_from_a_multi_import_file() {
        let mut modules = BTreeSet::new();
        collect_primitive_modules_from_source(
            "use adico_primitives::calendar::DateRange;\nuse adico_primitives::date_picker::{\n    DatePicker as PrimitiveDatePicker,\n};\nuse adico_primitives::icons::ChevronDown;\nuse adico_primitives::popover::{PopoverRoot, PopoverRootProps};\n",
            &mut modules,
        );
        assert_eq!(
            modules,
            BTreeSet::from([
                "calendar".to_string(),
                "date_picker".to_string(),
                "icons".to_string(),
                "popover".to_string(),
            ])
        );
    }
}
