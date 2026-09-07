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
    /// A numeric type (`f32`/`f64`/any integer) -- a `NumberControl`. No
    /// generated code, same reason as `Bool`.
    Number,
    /// `ReadSignal<Option<bool>>` (or `Signal<Option<bool>>`, or either
    /// wrapped in an outer `Option<...>`) -- the tri-state
    /// uncontrolled/on/off idiom several overlay components' `open` prop
    /// uses -- an `OptionalBoolControl`. No generated code, same reason as
    /// `Bool`.
    OptionalBool,
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
        // The real shape every controlled-`open` prop across the overlay
        // family (`Tooltip`/`Popover`/`HoverCard`/menu family/`Sidebar`)
        // actually declares -- verified against
        // `packages/adico-primitives/src/{tooltip,popover,hover_card,menu}.rs`
        // and `registry/ui/sidebar.rs`, all of which use
        // `ReadSignal<Option<bool>>` with no outer `Option`, not the
        // `Option<ReadSignal<Option<bool>>>` design.md's own Context
        // section names -- an outer `Option` is included here too in case
        // a future component declares it that way, but no current one does.
        "ReadSignal<Option<bool>>"
        | "Signal<Option<bool>>"
        | "Option<ReadSignal<Option<bool>>>"
        | "Option<Signal<Option<bool>>>" => return PropShape::OptionalBool,
        _ => {}
    }
    if type_name.starts_with("EventHandler<") {
        return PropShape::Skipped("EventHandler props have no matching demo control");
    }
    if type_name.starts_with("Signal<") || type_name.starts_with("ReadSignal<") {
        return PropShape::Skipped("Signal/ReadSignal-typed props have no matching demo control");
    }
    if is_numeric_type(type_name) {
        return PropShape::Number;
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

/// Converts a snake_case field identifier into a Title Case label
/// (`default_open` -> `Default Open`) -- the field-name analog of
/// `humanize_pascal_case_label`, for a generated `<Comp>Controls`' per-field
/// control labels.
pub fn humanize_field_label(field_name: &str) -> String {
    field_name
        .strip_prefix("r#")
        .unwrap_or(field_name)
        .split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Converts a PascalCase identifier into space-separated Title Case words
/// (`IconXs` -> `Icon Xs`). Deliberately mechanical -- see `design.md`'s
/// "labels are derived from the identifier, not doc comments" decision. Used
/// for two distinct display labels that both start from a PascalCase
/// identifier: an enum variant's option label, and a component's
/// `ControlGroup` group label (e.g. `AccordionItem` -> `Accordion Item`) --
/// the algorithm has no variant-specific logic, so one function serves both.
pub fn humanize_pascal_case_label(ident: &str) -> String {
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

/// One qualifying (non-`Skipped`) prop field on a specific component, ready
/// for `DemoState`/`Controls`/`Preview` codegen.
struct QualifyingField<'a> {
    name: &'a str,
    shape: PropShape,
    /// The field's real declared type (e.g. `"u32"`, `"ButtonVariant"`,
    /// `"ReadSignal<Option<bool>>"`) -- kept alongside `shape` since a
    /// `Number` field's `DemoState` slot must preserve its own real numeric
    /// type, not just `f64` (which is only `NumberControl`'s own type).
    type_name: &'a str,
}

/// A component's own props, looked up by its logical name -- the
/// `#[derive(Props)]` struct is keyed under `<Name>Props` in
/// `FileIntrospection::props`, while an inline-argument component is keyed
/// directly under `<Name>` -- matching the fallback
/// `packages/adico-xtask/src/prop_parity.rs` already established for this
/// exact ambiguity.
fn props_for_component<'a>(
    component_name: &str,
    introspection: &'a crate::rust_introspect::FileIntrospection,
) -> Option<&'a Vec<crate::rust_introspect::PropField>> {
    introspection
        .props
        .get(&format!("{component_name}Props"))
        .or_else(|| introspection.props.get(component_name))
}

/// A component's own qualifying (controllable) fields, in declaration
/// order. Empty for a component with no representable props, or none found
/// at all (e.g. a bare `pub use` re-export with no local props visible to
/// introspection -- see this module's own doc comment on that limitation).
fn qualifying_fields<'a>(
    component_name: &str,
    introspection: &'a crate::rust_introspect::FileIntrospection,
) -> Vec<QualifyingField<'a>> {
    let Some(fields) = props_for_component(component_name, introspection) else {
        return Vec::new();
    };
    fields
        .iter()
        .filter_map(
            |field| match classify_prop_type(&field.type_name, &introspection.enums) {
                PropShape::Skipped(_) => None,
                shape => Some(QualifyingField {
                    name: &field.name,
                    shape,
                    type_name: &field.type_name,
                }),
            },
        )
        .collect()
}

/// Whether a component declares its own `children: Element` prop -- decides
/// whether its generated `Preview` takes a `children` parameter at all,
/// since not every single-root component has one (e.g. `Switch`).
fn has_children_field(
    component_name: &str,
    introspection: &crate::rust_introspect::FileIntrospection,
) -> bool {
    props_for_component(component_name, introspection).is_some_and(|fields| {
        fields
            .iter()
            .any(|field| field.name == "children" && field.type_name == "Element")
    })
}

/// The `DemoState` struct field's own type: matches the real prop type for
/// every shape except `Bool`/`Text`/`OptionalBool`, which normalize to
/// their control's own plain value type (`bool`/`String`/`Option<bool>`)
/// since the real prop type for those (e.g. `Option<bool>`, or
/// `ReadSignal<Option<bool>>`) is either identical or a reactive wrapper
/// `Controls`/`Preview` construct fresh each render, not something
/// `DemoState` itself needs to store.
fn demo_state_field_type(field: &QualifyingField) -> String {
    match &field.shape {
        PropShape::Bool => "bool".to_string(),
        PropShape::Text => "String".to_string(),
        PropShape::Number => field.type_name.to_string(),
        PropShape::OptionalBool => "Option<bool>".to_string(),
        PropShape::Enum(enum_name) => enum_name.clone(),
        PropShape::Skipped(_) => unreachable!("qualifying_fields already filtered Skipped"),
    }
}

/// Renders `pub struct <Comp>DemoState { ... }`, deriving `Default` rather
/// than hand-writing an `impl Default` block: every `PropShape`'s
/// `DemoState` field type (`bool`/`String`/a numeric primitive/
/// `Option<bool>`/an enum whose own `#[default]` variant this generator
/// already requires via `classify_prop_type`'s `Enum` match arm) has a
/// `Default::default()` that is exactly the value this generator would
/// otherwise have chosen by hand, so a derive produces identical behavior
/// with no generated code -- caught by `clippy::derivable_impls` on the
/// very first hand-written version, not anticipated in design.md.
fn render_demo_state(component_name: &str, fields: &[QualifyingField]) -> String {
    let mut body = String::new();
    body.push_str(&format!(
        "/// Generated demo state for [`{component_name}`], one field per controllable prop.\n"
    ));
    body.push_str("#[derive(Clone, Default, PartialEq)]\n");
    body.push_str(&format!("pub struct {component_name}DemoState {{\n"));
    for field in fields {
        body.push_str(&format!(
            "    pub {}: {},\n",
            field.name,
            demo_state_field_type(field)
        ));
    }
    body.push_str("}\n\n");
    body
}

/// The identifier used for a field's own local per-field `Signal` inside a
/// generated `<Comp>Controls` body. Identical to the field's own name,
/// except when a real prop is itself named `state` -- which does happen
/// (`Attachment`'s own `state` field) -- since the outer parameter is
/// always named `state` too (preserving `proposal.md`'s
/// `ButtonControls { state }` field-init-shorthand calling convention),
/// and `let mut state = use_signal(|| state().state)` would silently
/// shadow the outer parameter with a local of the wrong type. Found via a
/// real compile error on `Attachment`, not anticipated in design.md.
fn local_signal_name(field_name: &str) -> String {
    if field_name == "state" {
        "state_field".to_string()
    } else {
        field_name.to_string()
    }
}

/// Renders `#[component] pub fn <Comp>Controls(state: Signal<<Comp>DemoState>) -> Element`.
/// Each field gets its own local `Signal`, seeded from `state`'s current
/// value, bound to the matching control; one combined `use_effect` writes
/// every local signal's value back into `state` on change. A local signal
/// per field (rather than a generated field-projecting lens over `state`
/// directly) keeps every existing control's `Signal<T>` signature exactly
/// as Section 1 left it -- see `design.md`'s D2, which left this exact
/// choice open as a Task 2 implementation detail.
fn render_controls_component(component_name: &str, fields: &[QualifyingField]) -> String {
    let mut body = String::new();
    body.push_str(&format!(
        "#[component]\npub fn {component_name}Controls(mut state: Signal<{component_name}DemoState>) -> Element {{\n"
    ));
    for field in fields {
        let local = local_signal_name(field.name);
        // `NumberControl` is always `Signal<f64>`; only cast when the real
        // field type isn't already `f64` (a redundant same-type cast is a
        // clippy error under this baseline's `-D warnings`).
        let seed = if matches!(field.shape, PropShape::Number) && field.type_name != "f64" {
            format!("state().{} as f64", field.name)
        } else {
            format!("state().{}", field.name)
        };
        body.push_str(&format!("    let {local} = use_signal(|| {seed});\n"));
    }
    body.push_str("    use_effect(move || {\n");
    body.push_str(&format!("        state.set({component_name}DemoState {{\n"));
    for field in fields {
        let local = local_signal_name(field.name);
        let value = if matches!(field.shape, PropShape::Number) && field.type_name != "f64" {
            format!("{local}() as {}", field.type_name)
        } else {
            format!("{local}()")
        };
        body.push_str(&format!("            {}: {value},\n", field.name));
    }
    body.push_str("        });\n    });\n");
    body.push_str("    rsx! {\n");
    let group_label = humanize_pascal_case_label(component_name);
    body.push_str(&format!(
        "        ControlGroup {{ part: \"{group_label}\",\n"
    ));
    for field in fields {
        let label = humanize_field_label(field.name);
        let name = local_signal_name(field.name);
        match &field.shape {
            PropShape::Bool => {
                body.push_str(&format!(
                    "        BoolControl {{ label: \"{label}\", value: {name} }}\n"
                ));
            }
            PropShape::Text => {
                body.push_str(&format!(
                    "        TextControl {{ label: \"{label}\", value: {name} }}\n"
                ));
            }
            PropShape::Number => {
                body.push_str(&format!(
                    "        NumberControl {{ label: \"{label}\", value: {name} }}\n"
                ));
            }
            PropShape::OptionalBool => {
                body.push_str(&format!(
                    "        OptionalBoolControl {{ label: \"{label}\", value: {name} }}\n"
                ));
            }
            PropShape::Enum(enum_name) => {
                let const_name = format!("{}_OPTIONS", to_screaming_snake_case(enum_name));
                body.push_str(&format!(
                    "        SelectControl {{ label: \"{label}\", value: {name}, options: {const_name} }}\n"
                ));
            }
            PropShape::Skipped(_) => unreachable!("qualifying_fields already filtered Skipped"),
        }
    }
    body.push_str("        }\n    }\n}\n\n");
    body
}

/// Renders `#[component] pub fn <Comp>Controls() -> Element` for a component
/// with no controllable props at all -- an explicit, labeled empty-state
/// group instead of skipping the component entirely (see design.md's "every
/// discovered component gets a ControlGroup" decision). No `DemoState`
/// struct is generated to pair with it: there is nothing for one to hold,
/// and a `Signal<...>` parameter bound to an empty struct would be unused
/// under this workspace's `-D warnings` baseline.
fn render_empty_controls_component(component_name: &str) -> String {
    let group_label = humanize_pascal_case_label(component_name);
    format!(
        "#[component]\npub fn {component_name}Controls() -> Element {{\n    rsx! {{\n        ControlGroup {{ part: \"{group_label}\",\n            p {{ class: \"text-sm text-muted-foreground\", \"No adjustable props.\" }}\n        }}\n    }}\n}}\n\n"
    )
}

/// Renders `#[component] pub fn <Comp>Preview(state: <Comp>DemoState, [children: Element])
/// -> Element`, invoking the real component with every field -- only called
/// for a single-root, non-generic item (`design.md`'s D3).
fn render_preview_component(
    component_name: &str,
    fields: &[QualifyingField],
    has_children: bool,
) -> String {
    let mut body = String::new();
    if has_children {
        body.push_str(&format!(
            "#[component]\npub fn {component_name}Preview(state: {component_name}DemoState, children: Element) -> Element {{\n"
        ));
    } else {
        body.push_str(&format!(
            "#[component]\npub fn {component_name}Preview(state: {component_name}DemoState) -> Element {{\n"
        ));
    }
    body.push_str(&format!("    rsx! {{\n        {component_name} {{\n"));
    for field in fields {
        let expr = if matches!(field.shape, PropShape::OptionalBool) {
            format!("ReadSignal::from(Signal::new(state.{}))", field.name)
        } else {
            format!("state.{}", field.name)
        };
        body.push_str(&format!("            {}: {expr},\n", field.name));
    }
    if has_children {
        body.push_str("            {children}\n");
    }
    body.push_str("        }\n    }\n}\n\n");
    body
}

/// Renders one item's generated file content, or `None` if it has neither a
/// qualifying enum prop (the pre-existing `_OPTIONS` constants) nor any
/// component with at least one controllable prop (the new
/// `DemoState`/`Controls`/`Preview` triad) -- no file is written for such
/// items.
fn render_component_file(
    item_stem: &str,
    introspection: &crate::rust_introspect::FileIntrospection,
) -> Option<String> {
    let enum_names = qualifying_enum_names(introspection);

    // Single-root, non-generic detection (design.md's D3): exactly one
    // locally-visible component. Note this is unaffected by re-export
    // resolution: a multi-part item's re-exported root (e.g. `Accordion`)
    // simply becomes one more entry in `introspection.components` alongside
    // its locally-defined siblings, so `sole_root` still correctly stays
    // `None` for it.
    let sole_root = match introspection.components.as_slice() {
        [only] if !introspection.generic.contains(only) => Some(only.as_str()),
        _ => None,
    };

    let mut used_controls = BTreeSet::new();
    let mut used_component_names = BTreeSet::new();
    let mut demo_sections = String::new();
    for component_name in &introspection.components {
        let fields = qualifying_fields(component_name, introspection);
        if fields.is_empty() {
            // No controllable props -- including every resolved re-exported
            // component, whose real props live in `adico-primitives` and
            // aren't extracted by this generator (a separate, out-of-scope
            // prop-coverage gap). Still gets a labeled, empty-state group
            // rather than being skipped -- see design.md's "every discovered
            // component gets a ControlGroup" decision. No `Preview` either,
            // matching this item's own pre-existing behavior for any
            // zero-field component.
            used_controls.insert("ControlGroup");
            demo_sections.push_str(&render_empty_controls_component(component_name));
            continue;
        }
        used_controls.insert("ControlGroup");
        for field in &fields {
            used_controls.insert(match &field.shape {
                PropShape::Bool => "BoolControl",
                PropShape::Text => "TextControl",
                PropShape::Number => "NumberControl",
                PropShape::OptionalBool => "OptionalBoolControl",
                PropShape::Enum(_) => "SelectControl",
                PropShape::Skipped(_) => unreachable!("qualifying_fields already filtered Skipped"),
            });
        }
        demo_sections.push_str(&render_demo_state(component_name, &fields));
        demo_sections.push_str(&render_controls_component(component_name, &fields));
        if sole_root == Some(component_name.as_str()) {
            used_component_names.insert(component_name.clone());
            let has_children = has_children_field(component_name, introspection);
            demo_sections.push_str(&render_preview_component(
                component_name,
                &fields,
                has_children,
            ));
        }
    }

    if enum_names.is_empty() && demo_sections.is_empty() {
        return None;
    }

    let mut body = String::new();
    body.push_str(
        "//! @generated by `cargo xtask playground-controls sync`. Do not edit by hand.\n",
    );
    body.push_str(&format!(
        "//! Source: `apps/playground/src/components/ui/{item_stem}.rs`.\n\n"
    ));
    if !demo_sections.is_empty() {
        body.push_str("use dioxus::prelude::*;\n\n");
    }
    if !used_controls.is_empty() {
        body.push_str(&format!(
            "use crate::components::controls::{{{}}};\n",
            used_controls.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    let ui_imports: BTreeSet<String> = enum_names
        .iter()
        .cloned()
        .chain(used_component_names)
        .collect();
    if !ui_imports.is_empty() {
        body.push_str(&format!(
            "use crate::components::ui::{{{}}};\n",
            ui_imports.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    body.push('\n');

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
            let label = humanize_pascal_case_label(variant);
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

    body.push_str(&demo_sections);

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

/// Merges newly discovered, resolved re-exported component names into
/// `components`, skipping a name already present -- defensive: shouldn't
/// happen in practice (a locally-defined component and a re-export would
/// have to share a name), but guards against emitting the same generated
/// item twice.
fn merge_resolved_components(components: &mut Vec<String>, resolved: Vec<String>) {
    for name in resolved {
        if !components.contains(&name) {
            components.push(name);
        }
    }
}

fn plan_item(item_stem: &str, source_path: &Path, root: &Path) -> Result<ItemPlan, String> {
    let mut introspection = introspect_file(source_path);
    let primitives_src_dir = root.join("packages/adico-primitives/src");
    let resolved = crate::rust_introspect::resolve_reexported_components(
        &introspection.primitive_reexports,
        &primitives_src_dir,
    )
    .map_err(|unresolved| format!("{item_stem}: {unresolved}"))?;
    merge_resolved_components(&mut introspection.components, resolved);

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
        let plan = plan_item(stem, path, root)?;
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
        let plan = plan_item(stem, path, root)?;
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
        let plan = plan_item(stem, path, root)?;
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
            classify_prop_type("SomeUnknownType", &enums),
            PropShape::Skipped("unrecognized prop type")
        );
    }

    #[test]
    fn classifies_numeric_types_as_number() {
        let enums = BTreeMap::new();
        for numeric in [
            "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64",
            "u128", "usize",
        ] {
            assert_eq!(classify_prop_type(numeric, &enums), PropShape::Number);
        }
    }

    #[test]
    fn classifies_the_controlled_open_shape_as_optional_bool() {
        let enums = BTreeMap::new();
        for shape in [
            "ReadSignal<Option<bool>>",
            "Signal<Option<bool>>",
            "Option<ReadSignal<Option<bool>>>",
            "Option<Signal<Option<bool>>>",
        ] {
            assert_eq!(classify_prop_type(shape, &enums), PropShape::OptionalBool);
        }
    }

    #[test]
    fn a_bool_wrapped_signal_over_a_non_bool_type_is_still_skipped() {
        let enums = BTreeMap::new();
        assert_eq!(
            classify_prop_type("ReadSignal<Option<f64>>", &enums),
            PropShape::Skipped("Signal/ReadSignal-typed props have no matching demo control")
        );
    }

    #[test]
    fn humanizes_a_snake_case_field_name() {
        assert_eq!(humanize_field_label("variant"), "Variant");
        assert_eq!(humanize_field_label("default_open"), "Default Open");
        assert_eq!(
            humanize_field_label("allow_multiple_pressed"),
            "Allow Multiple Pressed"
        );
    }

    #[test]
    fn strips_the_raw_identifier_prefix_before_humanizing() {
        // A field literally named `type` (a reserved word) is a raw
        // identifier, `r#type` -- found producing the label "R#type" on
        // `Input`'s real `r#type` field, not anticipated when this
        // function was written.
        assert_eq!(humanize_field_label("r#type"), "Type");
    }

    #[test]
    fn humanizes_a_single_word_identifier() {
        assert_eq!(humanize_pascal_case_label("Default"), "Default");
        assert_eq!(humanize_pascal_case_label("Destructive"), "Destructive");
    }

    #[test]
    fn humanizes_a_two_word_pascal_case_identifier() {
        assert_eq!(humanize_pascal_case_label("IconLarge"), "Icon Large");
    }

    #[test]
    fn humanizes_an_identifier_with_an_acronym_like_run() {
        assert_eq!(humanize_pascal_case_label("IconXs"), "Icon Xs");
        assert_eq!(humanize_pascal_case_label("IconSm"), "Icon Sm");
    }

    #[test]
    fn screaming_snake_case_matches_rust_const_naming() {
        assert_eq!(to_screaming_snake_case("ButtonVariant"), "BUTTON_VARIANT");
        assert_eq!(to_screaming_snake_case("Sidebar"), "SIDEBAR");
    }

    #[test]
    fn a_field_literally_named_state_does_not_shadow_the_outer_signal() {
        // Regression test for a real bug found on `Attachment`, whose own
        // `state` field collided with the generated `Controls` function's
        // outer `state: Signal<...>` parameter.
        assert_eq!(local_signal_name("state"), "state_field");
        assert_eq!(local_signal_name("variant"), "variant");
    }

    #[test]
    fn a_generated_controls_panel_body_is_wrapped_in_a_humanized_control_group_label() {
        let fields = vec![QualifyingField {
            name: "index",
            shape: PropShape::Number,
            type_name: "usize",
        }];
        let body = render_controls_component("AccordionItem", &fields);
        assert!(body.contains("ControlGroup { part: \"Accordion Item\","));
        assert!(!body.contains("\"AccordionItem\""));
        assert!(body.contains("pub fn AccordionItemControls"));
        assert!(body.contains("NumberControl { label: \"Index\", value: index }"));
    }

    #[test]
    fn a_zero_field_component_emits_an_empty_state_control_group_with_a_humanized_label() {
        let body = render_empty_controls_component("AccordionTrigger");
        assert!(body.contains("pub fn AccordionTriggerControls() -> Element"));
        assert!(body.contains("ControlGroup { part: \"Accordion Trigger\","));
        assert!(!body.contains("\"AccordionTrigger\""));
        assert!(body.contains("No adjustable props."));
        assert!(!body.contains("DemoState"));
    }

    /// A single-leaf-control file (only `NumberControl`, as `accordion.rs`
    /// has today) must still grow its `use crate::components::controls`
    /// line to include `ControlGroup` alongside it, in the braced form --
    /// see design.md's "import-set safety" decision (verified against the
    /// generator's own emitter, which always writes the braced form
    /// regardless of count and lets `rustfmt` -- not this code -- decide
    /// whether to unbrace a single name in the final committed output).
    #[test]
    fn a_single_leaf_control_file_grows_its_import_line_to_include_control_group() {
        let mut introspection = crate::rust_introspect::FileIntrospection {
            components: vec!["AccordionItem".to_string()],
            ..Default::default()
        };
        introspection.props.insert(
            "AccordionItem".to_string(),
            vec![crate::rust_introspect::PropField {
                name: "index".to_string(),
                type_name: "usize".to_string(),
                default: None,
            }],
        );
        let body =
            render_component_file("accordion", &introspection).expect("has a qualifying field");
        assert!(
            body.contains("use crate::components::controls::{ControlGroup, NumberControl};"),
            "unexpected import line in:\n{body}"
        );
    }

    #[test]
    fn a_component_with_no_qualifying_fields_still_produces_a_file_with_an_empty_state_group() {
        let introspection = crate::rust_introspect::FileIntrospection {
            components: vec!["AccordionTrigger".to_string()],
            ..Default::default()
        };
        let body = render_component_file("accordion", &introspection)
            .expect("zero-field components are no longer skipped");
        assert!(body.contains("use crate::components::controls::{ControlGroup};"));
        assert!(body.contains("pub fn AccordionTriggerControls() -> Element"));
        assert!(body.contains("No adjustable props."));
    }

    #[test]
    fn merge_resolved_components_grows_the_list_and_skips_duplicates() {
        let mut components = vec!["AccordionItem".to_string()];
        merge_resolved_components(
            &mut components,
            vec!["Accordion".to_string(), "AccordionItem".to_string()],
        );
        assert_eq!(
            components,
            vec!["AccordionItem".to_string(), "Accordion".to_string()]
        );
    }
}
