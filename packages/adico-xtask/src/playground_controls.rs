//! `cargo xtask playground-controls sync|check|diff`: generates compiled
//! Rust option-list constants for the playground's demo `SelectControl`s,
//! one file per playground UI component with at least one enum-typed prop
//! carrying a `#[default]` variant, under
//! `apps/playground/src/generated/controls/`.
//!
//! Unlike `primitive_usage.rs`/`styling_usage.rs`, this command's output is
//! compiled Rust, not committed JSON: each generated option constant ships
//! with a compile-time exhaustiveness guard over its source enum, so an
//! added/removed/renamed variant that isn't regenerated fails
//! `cargo check --locked --workspace`, not just this command's own `check`
//! subcommand -- see `design.md`'s "Compile-time exhaustiveness guard"
//! decision.
//!
//! Every prop on every playground UI component is classified into exactly
//! one of `PropShape::Bool`/`Text`/`Enum`/`Skipped(reason)`. Only `Enum`
//! shapes produce generated code; `sync`/`diff` print every `Skipped`
//! classification (with its reason) to stdout so an unsupported prop is
//! never dropped with no trace, satisfying this repository's own
//! `adico-playground-demo-controls` spec without inventing a second,
//! separately-committed record artifact.
//!
//! Runs fully offline: reads only `apps/playground/src/components/ui/*.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::rust_introspect::{EnumIntrospection, introspect_file};

fn playground_ui_dir(root: &Path) -> PathBuf {
    root.join("apps/playground/src/components/ui")
}

fn generated_dir(root: &Path) -> PathBuf {
    root.join("apps/playground/src/generated/controls")
}

/// How one prop field's type maps onto the playground's demo controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropShape {
    /// `bool` or `Option<bool>` -- a `BoolControl`. No generated code: the
    /// control operates on the primitive type directly.
    Bool,
    /// `String` -- a `TextControl`. No generated code, same reason as `Bool`.
    Text,
    /// An enum type with a `#[default]` variant declared in the same file --
    /// generates a `pub const <NAME>_OPTIONS` and its exhaustiveness guard.
    Enum(String),
    /// A type this tool does not represent as a control, with a fixed
    /// reason. Never silently dropped: `sync`/`diff` print every skip.
    Skipped(&'static str),
}

/// Classifies one prop field's type against the fixed allowlist from
/// `design.md` ("Prop-to-control mapping is a fixed allowlist"). `enums` is
/// the enclosing file's own introspected enums, since a qualifying enum
/// prop must be declared in the same file as the component that uses it.
pub fn classify_prop_type(
    type_name: &str,
    enums: &BTreeMap<String, EnumIntrospection>,
) -> PropShape {
    match type_name {
        "bool" | "Option<bool>" => return PropShape::Bool,
        "String" => return PropShape::Text,
        "Element" => return PropShape::Skipped("children has no matching demo control"),
        "Vec<Attribute>" => return PropShape::Skipped("attributes has no matching demo control"),
        "Option<String>" => {
            return PropShape::Skipped("Option<String> has no matching demo control");
        }
        _ => {}
    }
    if type_name.starts_with("EventHandler<") {
        return PropShape::Skipped("EventHandler props have no matching demo control");
    }
    if type_name.starts_with("Signal<") || type_name.starts_with("ReadSignal<") {
        return PropShape::Skipped("Signal/ReadSignal-typed props have no matching demo control");
    }
    if is_numeric_type(type_name) {
        return PropShape::Skipped("numeric props have no matching demo control yet");
    }
    match enums.get(type_name) {
        Some(info) if info.default_variant.is_some() => PropShape::Enum(type_name.to_string()),
        Some(_) => PropShape::Skipped("enum has no #[default] variant"),
        None => PropShape::Skipped("unrecognized prop type"),
    }
}

fn is_numeric_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "f32"
            | "f64"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
    )
}

/// Converts a PascalCase variant identifier into space-separated words for
/// use as a control's option label (`IconXs` -> `Icon Xs`). Deliberately
/// mechanical -- see `design.md`'s "labels are derived from the identifier,
/// not doc comments" decision.
pub fn humanize_variant_label(ident: &str) -> String {
    let mut label = String::with_capacity(ident.len() + 4);
    let chars: Vec<char> = ident.chars().collect();
    for (index, &ch) in chars.iter().enumerate() {
        if index > 0 && ch.is_uppercase() {
            let previous_is_lower = chars[index - 1].is_lowercase();
            let next_is_lower = chars.get(index + 1).is_some_and(|c| c.is_lowercase());
            if previous_is_lower || (chars[index - 1].is_uppercase() && next_is_lower) {
                label.push(' ');
            }
        }
        label.push(ch);
    }
    label
}

/// The enum names this file's Props struct(s) actually use as a qualifying
/// (`PropShape::Enum`) prop type, deduplicated and sorted so codegen output
/// is deterministic regardless of field declaration order.
fn qualifying_enum_names(
    introspection: &crate::rust_introspect::FileIntrospection,
) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for fields in introspection.props.values() {
        for field in fields {
            if let PropShape::Enum(enum_name) =
                classify_prop_type(&field.type_name, &introspection.enums)
            {
                names.insert(enum_name);
            }
        }
    }
    names
}

/// Renders one component's generated file content, or `None` if it has no
/// qualifying enum props (no file is written for such components).
fn render_component_file(
    item_stem: &str,
    introspection: &crate::rust_introspect::FileIntrospection,
) -> Option<String> {
    let enum_names = qualifying_enum_names(introspection);
    if enum_names.is_empty() {
        return None;
    }

    let mut body = String::new();
    body.push_str(
        "//! @generated by `cargo xtask playground-controls sync`. Do not edit by hand.\n",
    );
    body.push_str(&format!(
        "//! Source: `apps/playground/src/components/ui/{item_stem}.rs`.\n\n"
    ));
    body.push_str(&format!(
        "use crate::components::ui::{{{}}};\n\n",
        enum_names
            .clone()
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ")
    ));

    for enum_name in &enum_names {
        let info = &introspection.enums[enum_name];
        let const_name = format!("{}_OPTIONS", to_screaming_snake_case(enum_name));

        body.push_str(&format!(
            "/// Generated from `{enum_name}`'s declared variants.\n"
        ));
        body.push_str(&format!(
            "pub const {const_name}: &[(&str, {enum_name})] = &[\n"
        ));
        for variant in &info.variants {
            let label = humanize_variant_label(variant);
            body.push_str(&format!("    (\"{label}\", {enum_name}::{variant}),\n"));
        }
        body.push_str("];\n\n");

        body.push_str("const _: () = {\n");
        body.push_str(&format!("    fn _exhaustive(value: {enum_name}) {{\n"));
        body.push_str("        match value {\n");
        for variant in &info.variants {
            body.push_str(&format!("            {enum_name}::{variant} => {{}}\n"));
        }
        body.push_str("        }\n");
        body.push_str("    }\n");
        body.push_str("};\n\n");
    }

    Some(body)
}

/// Pipes `source` through `rustfmt --edition 2024` so generated output is
/// always in the same canonical form `cargo fmt --all --check` expects --
/// the template above deliberately doesn't hand-manage import brace style
/// or blank lines, since rustfmt is the actual authority on both.
fn format_rust_source(source: &str) -> Result<String, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot run rustfmt: {error}"))?;
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(source.as_bytes())
        .map_err(|error| format!("cannot write to rustfmt stdin: {error}"))?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("cannot read rustfmt output: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "rustfmt failed on generated source: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("rustfmt produced non-UTF8: {error}"))
}

fn to_screaming_snake_case(pascal: &str) -> String {
    let mut out = String::with_capacity(pascal.len() + 4);
    for (index, ch) in pascal.chars().enumerate() {
        if index > 0 && ch.is_uppercase() {
            out.push('_');
        }
        out.push(ch.to_ascii_uppercase());
    }
    out
}

/// One playground UI item's classification pass: its generated file content
/// (if it has any qualifying enum prop) and every `Skipped` prop found,
/// for `sync`/`diff` to print as this item's trace of excluded props.
struct ItemPlan {
    item_stem: String,
    generated_content: Option<String>,
    skipped: Vec<(String, &'static str)>,
}

fn plan_item(item_stem: &str, source_path: &Path) -> Result<ItemPlan, String> {
    let introspection = introspect_file(source_path);
    let mut skipped = Vec::new();
    for fields in introspection.props.values() {
        for field in fields {
            if let PropShape::Skipped(reason) =
                classify_prop_type(&field.type_name, &introspection.enums)
            {
                skipped.push((field.name.clone(), reason));
            }
        }
    }
    let generated_content = render_component_file(item_stem, &introspection)
        .map(|content| format_rust_source(&content))
        .transpose()?;
    Ok(ItemPlan {
        item_stem: item_stem.to_string(),
        generated_content,
        skipped,
    })
}

/// Every playground UI component source file (excluding the `mod.rs`
/// barrel), by its file stem (e.g. `button`), sorted for determinism.
fn playground_ui_items(root: &Path) -> Result<Vec<(String, PathBuf)>, String> {
    let dir = playground_ui_dir(root);
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

fn generated_mod_rs(item_stems: &[String]) -> String {
    let mut body = String::new();
    body.push_str(
        "//! @generated by `cargo xtask playground-controls sync`. Do not edit by hand.\n\n",
    );
    for stem in item_stems {
        body.push_str(&format!("pub mod {stem};\n"));
    }
    body.push('\n');
    for stem in item_stems {
        body.push_str(&format!("pub use {stem}::*;\n"));
    }
    body
}

// --- sync / check / diff ----------------------------------------------------

pub fn sync(root: &Path) -> Result<(), String> {
    let items = playground_ui_items(root)?;
    let dir = generated_dir(root);
    fs::create_dir_all(&dir)
        .map_err(|error| format!("cannot create {}: {error}", dir.display()))?;

    let mut generated_stems = Vec::new();
    for (stem, path) in &items {
        let plan = plan_item(stem, path)?;
        for (prop_name, reason) in &plan.skipped {
            println!("{stem}: skipped `{prop_name}` ({reason})");
        }
        let file_path = dir.join(format!("{stem}.rs"));
        match plan.generated_content {
            Some(content) => {
                fs::write(&file_path, content)
                    .map_err(|error| format!("cannot write {}: {error}", file_path.display()))?;
                generated_stems.push(plan.item_stem);
            }
            None => {
                if file_path.exists() {
                    fs::remove_file(&file_path).map_err(|error| {
                        format!("cannot remove stale {}: {error}", file_path.display())
                    })?;
                }
            }
        }
    }

    let mod_rs_path = dir.join("mod.rs");
    let mod_rs_content = format_rust_source(&generated_mod_rs(&generated_stems))?;
    fs::write(&mod_rs_path, mod_rs_content)
        .map_err(|error| format!("cannot write {}: {error}", mod_rs_path.display()))?;

    println!(
        "Synced {} apps/playground/src/generated/controls/<item>.rs file(s).",
        generated_stems.len()
    );
    Ok(())
}

pub fn check(root: &Path) -> Result<(), String> {
    let items = playground_ui_items(root)?;
    let dir = generated_dir(root);
    let mut violations = Vec::new();
    let mut expected_stems = Vec::new();

    for (stem, path) in &items {
        let plan = plan_item(stem, path)?;
        let file_path = dir.join(format!("{stem}.rs"));
        let on_disk = fs::read_to_string(&file_path).ok();
        match (&plan.generated_content, &on_disk) {
            (Some(expected), Some(actual)) if expected == actual => {
                expected_stems.push(plan.item_stem.clone());
            }
            (Some(_), Some(_)) => violations.push(format!(
                "{}: generated file is stale (source enum changed without regenerating)",
                file_path.display()
            )),
            (Some(_), None) => violations.push(format!(
                "{}: missing generated file (component has a qualifying enum prop)",
                file_path.display()
            )),
            (None, Some(_)) => violations.push(format!(
                "{}: generated file should not exist (no qualifying enum prop)",
                file_path.display()
            )),
            (None, None) => {}
        }
    }

    if dir.is_dir() {
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
            if !items.iter().any(|(item_stem, _)| item_stem == &stem) {
                violations.push(format!(
                    "{}: generated file has no matching playground UI source file",
                    path.display()
                ));
            }
        }
    }

    let expected_mod_rs = format_rust_source(&generated_mod_rs(&expected_stems))?;
    let mod_rs_path = dir.join("mod.rs");
    match fs::read_to_string(&mod_rs_path) {
        Ok(actual) if actual == expected_mod_rs => {}
        Ok(_) => violations.push(format!("{}: is stale", mod_rs_path.display())),
        Err(_) => violations.push(format!("{}: missing", mod_rs_path.display())),
    }

    if violations.is_empty() {
        println!("playground-controls check passed: {} item(s).", items.len());
        Ok(())
    } else {
        Err(violations.join("\n"))
    }
}

pub fn diff(root: &Path) -> Result<(), String> {
    let items = playground_ui_items(root)?;
    let dir = generated_dir(root);
    let mut changed = 0usize;

    for (stem, path) in &items {
        let plan = plan_item(stem, path)?;
        for (prop_name, reason) in &plan.skipped {
            println!("{stem}: skipped `{prop_name}` ({reason})");
        }
        let file_path = dir.join(format!("{stem}.rs"));
        let on_disk = fs::read_to_string(&file_path).ok();
        if plan.generated_content != on_disk {
            changed += 1;
            match (&plan.generated_content, &on_disk) {
                (Some(_), None) => println!("{stem}: would create {}", file_path.display()),
                (None, Some(_)) => println!("{stem}: would remove {}", file_path.display()),
                (Some(_), Some(_)) => println!("{stem}: would update {}", file_path.display()),
                (None, None) => {}
            }
        }
    }

    if changed == 0 {
        println!("playground-controls diff: no changes.");
    } else {
        println!("playground-controls diff: {changed} item(s) would change.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn enum_with_default(variants: &[&str], default: Option<&str>) -> EnumIntrospection {
        EnumIntrospection {
            variants: variants.iter().map(|value| value.to_string()).collect(),
            default_variant: default.map(|value| value.to_string()),
        }
    }

    #[test]
    fn classifies_every_supported_and_skipped_shape() {
        let mut enums = BTreeMap::new();
        enums.insert(
            "WidgetVariant".to_string(),
            enum_with_default(&["Default", "Destructive"], Some("Default")),
        );
        enums.insert(
            "WidgetAlignment".to_string(),
            enum_with_default(&["Start", "End"], None),
        );

        assert_eq!(classify_prop_type("bool", &enums), PropShape::Bool);
        assert_eq!(classify_prop_type("Option<bool>", &enums), PropShape::Bool);
        assert_eq!(classify_prop_type("String", &enums), PropShape::Text);
        assert_eq!(
            classify_prop_type("WidgetVariant", &enums),
            PropShape::Enum("WidgetVariant".to_string())
        );

        assert_eq!(
            classify_prop_type("WidgetAlignment", &enums),
            PropShape::Skipped("enum has no #[default] variant")
        );
        assert_eq!(
            classify_prop_type("Element", &enums),
            PropShape::Skipped("children has no matching demo control")
        );
        assert_eq!(
            classify_prop_type("Vec<Attribute>", &enums),
            PropShape::Skipped("attributes has no matching demo control")
        );
        assert_eq!(
            classify_prop_type("Option<String>", &enums),
            PropShape::Skipped("Option<String> has no matching demo control")
        );
        assert_eq!(
            classify_prop_type("EventHandler<MouseEvent>", &enums),
            PropShape::Skipped("EventHandler props have no matching demo control")
        );
        assert_eq!(
            classify_prop_type("Signal<bool>", &enums),
            PropShape::Skipped("Signal/ReadSignal-typed props have no matching demo control")
        );
        assert_eq!(
            classify_prop_type("ReadSignal<Option<f64>>", &enums),
            PropShape::Skipped("Signal/ReadSignal-typed props have no matching demo control")
        );
        assert_eq!(
            classify_prop_type("f64", &enums),
            PropShape::Skipped("numeric props have no matching demo control yet")
        );
        assert_eq!(
            classify_prop_type("SomeUnknownType", &enums),
            PropShape::Skipped("unrecognized prop type")
        );
    }

    #[test]
    fn humanizes_a_single_word_identifier() {
        assert_eq!(humanize_variant_label("Default"), "Default");
        assert_eq!(humanize_variant_label("Destructive"), "Destructive");
    }

    #[test]
    fn humanizes_a_two_word_pascal_case_identifier() {
        assert_eq!(humanize_variant_label("IconLarge"), "Icon Large");
    }

    #[test]
    fn humanizes_an_identifier_with_an_acronym_like_run() {
        assert_eq!(humanize_variant_label("IconXs"), "Icon Xs");
        assert_eq!(humanize_variant_label("IconSm"), "Icon Sm");
    }

    #[test]
    fn screaming_snake_case_matches_rust_const_naming() {
        assert_eq!(to_screaming_snake_case("ButtonVariant"), "BUTTON_VARIANT");
        assert_eq!(to_screaming_snake_case("Sidebar"), "SIDEBAR");
    }
}
