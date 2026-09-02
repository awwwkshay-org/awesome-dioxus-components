//! Real Rust-source introspection (via `syn`) for the compatibility-tracking
//! commands (`primitive-compat`, `component-compat`): which component functions,
//! `#[derive(Props...)]` struct fields, and `use_*` hooks a file exposes.
//!
//! Not a semantic analyzer -- it reads syntax only, so it can't tell a
//! genuine component function from an unrelated `pub fn` that happens to
//! start with an uppercase letter. Good enough for a tracking snapshot.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Serialize;
use syn::{Fields, FnArg, Item, Pat, ReturnType, Type, UseTree, Visibility};

#[derive(Debug, Default, Serialize)]
pub struct FileIntrospection {
    pub exists: bool,
    pub components: Vec<String>,
    pub props: BTreeMap<String, Vec<PropField>>,
    pub hooks_defined: Vec<String>,
    pub hooks_used: Vec<String>,
    pub enums: BTreeMap<String, EnumIntrospection>,
}

/// A public enum's variants, for callers that need to know an enum-typed
/// prop's real, current set of values (e.g. generating a control's option
/// list). Deliberately does not carry doc comments: registry source doc
/// comments are descriptive prose ("Extra-small text button."), not short
/// labels, so callers that need a label derive one from the variant
/// identifier instead.
#[derive(Debug, Default, Serialize)]
pub struct EnumIntrospection {
    /// Variant identifiers, in declaration order.
    pub variants: Vec<String>,
    /// The variant identifier carrying `#[default]`, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_variant: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PropField {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    /// The field's `#[props(default = ...)]` expression, when present.
    /// `None` for a field with no declared default, not evidence that one
    /// doesn't exist at runtime (e.g. `Option<T>` fields default to `None`
    /// implicitly without a `#[props(...)]` attribute).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Introspect one Rust source file. Returns `exists: false` (not an error)
/// if the file doesn't exist or fails to parse -- a tracking snapshot should
/// degrade gracefully, not abort the whole sync over one unreadable file.
pub fn introspect_file(path: &Path) -> FileIntrospection {
    let Ok(source) = fs::read_to_string(path) else {
        return FileIntrospection::default();
    };
    let Ok(file) = syn::parse_file(&source) else {
        return FileIntrospection::default();
    };

    let mut result = FileIntrospection {
        exists: true,
        ..Default::default()
    };
    walk_items(&file.items, &mut result);
    result.hooks_defined.sort();
    result.hooks_defined.dedup();
    result.hooks_used.sort();
    result.hooks_used.dedup();
    result
}

/// Introspect every `.rs` file under a directory (for multi-file primitive
/// modules like `combobox/` or `select/`), merging the results.
pub fn introspect_directory(dir: &Path) -> FileIntrospection {
    let mut merged = FileIntrospection {
        exists: dir.is_dir(),
        ..Default::default()
    };
    if !dir.is_dir() {
        return merged;
    }
    let mut entries = Vec::new();
    collect_rust_files(dir, &mut entries);
    entries.sort();
    for path in entries {
        let single = introspect_file(&path);
        merged.components.extend(single.components);
        merged.props.extend(single.props);
        merged.hooks_defined.extend(single.hooks_defined);
        merged.hooks_used.extend(single.hooks_used);
        merged.enums.extend(single.enums);
    }
    merged.hooks_defined.sort();
    merged.hooks_defined.dedup();
    merged.hooks_used.sort();
    merged.hooks_used.dedup();
    merged
}

fn collect_rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
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

fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn walk_items(items: &[Item], result: &mut FileIntrospection) {
    for item in items {
        match item {
            Item::Fn(item_fn) if is_public(&item_fn.vis) => {
                let name = item_fn.sig.ident.to_string();
                if name.starts_with("use_") {
                    result.hooks_defined.push(name);
                } else if name.starts_with(|c: char| c.is_ascii_uppercase())
                    && returns_element(&item_fn.sig.output)
                {
                    result.components.push(name.clone());
                    if let Some(fields) = inline_component_props(item_fn) {
                        result.props.insert(name, fields);
                    }
                }
            }
            Item::Struct(item_struct)
                if is_public(&item_struct.vis)
                    && item_struct.ident.to_string().ends_with("Props") =>
            {
                if let Fields::Named(named) = &item_struct.fields {
                    let fields = named
                        .named
                        .iter()
                        .filter(|field| is_public(&field.vis))
                        .filter_map(|field| {
                            let name = field.ident.as_ref()?.to_string();
                            Some(PropField {
                                name,
                                type_name: type_to_string(&field.ty),
                                default: props_default_expr(&field.attrs),
                            })
                        })
                        .collect();
                    result.props.insert(item_struct.ident.to_string(), fields);
                }
            }
            Item::Enum(item_enum) if is_public(&item_enum.vis) => {
                let variants: Vec<String> = item_enum
                    .variants
                    .iter()
                    .map(|variant| variant.ident.to_string())
                    .collect();
                let default_variant = item_enum
                    .variants
                    .iter()
                    .find(|variant| {
                        variant
                            .attrs
                            .iter()
                            .any(|attr| attr.path().is_ident("default"))
                    })
                    .map(|variant| variant.ident.to_string());
                result.enums.insert(
                    item_enum.ident.to_string(),
                    EnumIntrospection {
                        variants,
                        default_variant,
                    },
                );
            }
            Item::Use(item_use) => {
                let mut names = Vec::new();
                collect_use_names(&item_use.tree, &mut names);
                result
                    .hooks_used
                    .extend(names.into_iter().filter(|name| name.starts_with("use_")));
            }
            Item::Mod(item_mod) => {
                let is_test_mod = item_mod.attrs.iter().any(|attr| {
                    attr.path().is_ident("cfg") && attr.to_token_stream_string().contains("test")
                });
                if !is_test_mod && let Some((_, nested)) = &item_mod.content {
                    walk_items(nested, result);
                }
            }
            _ => {}
        }
    }
}

/// Whether a component function's return type looks like `Element` (bare or
/// through the crate's `dioxus::prelude::Element` alias) -- distinguishes an
/// actual Dioxus component from an unrelated `pub fn Something(...)`.
fn returns_element(output: &ReturnType) -> bool {
    match output {
        ReturnType::Type(_, ty) => type_to_string(ty).ends_with("Element"),
        ReturnType::Default => false,
    }
}

/// Components declared Dioxus-macro-style with plain fn parameters instead
/// of an explicit `#[derive(Props)] struct FooProps`. Returns `None` for the
/// `props: FooProps` shape, which `walk_items`'s `Item::Struct` arm already
/// covers.
fn inline_component_props(item_fn: &syn::ItemFn) -> Option<Vec<PropField>> {
    if item_fn.sig.inputs.len() == 1
        && let Some(FnArg::Typed(pat_type)) = item_fn.sig.inputs.first()
        && matches!(&*pat_type.pat, Pat::Ident(pat_ident) if pat_ident.ident == "props")
    {
        return None;
    }
    let fields: Vec<PropField> = item_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat_type) => {
                let Pat::Ident(pat_ident) = &*pat_type.pat else {
                    return None;
                };
                Some(PropField {
                    name: pat_ident.ident.to_string(),
                    type_name: type_to_string(&pat_type.ty),
                    default: None,
                })
            }
            FnArg::Receiver(_) => None,
        })
        .collect();
    if fields.is_empty() {
        None
    } else {
        Some(fields)
    }
}

/// Extracts the `default = <expr>` argument from a `#[props(...)]` attribute,
/// if present. Dioxus' `#[derive(Props)]` also accepts a bare `#[props(default)]`
/// (meaning "use `Default::default()`"), reported here as `"default()"`.
fn props_default_expr(attrs: &[syn::Attribute]) -> Option<String> {
    use quote::ToTokens;
    for attr in attrs {
        if !attr.path().is_ident("props") {
            continue;
        }
        let mut default_value = None;
        let mut saw_bare_default = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                if meta.input.peek(syn::Token![=]) {
                    let value: syn::Expr = meta.value()?.parse()?;
                    default_value = Some(clean_token_string(value.to_token_stream().to_string()));
                } else {
                    saw_bare_default = true;
                }
            } else {
                // Consume this meta item's value (if any) so parsing the
                // rest of the attribute's other args doesn't fail.
                let _ = meta.value().and_then(|value| value.parse::<syn::Expr>());
            }
            Ok(())
        });
        if let Some(value) = default_value {
            return Some(value);
        }
        if saw_bare_default {
            return Some("default()".to_string());
        }
    }
    None
}

fn collect_use_names(tree: &UseTree, out: &mut Vec<String>) {
    match tree {
        UseTree::Path(path) => collect_use_names(&path.tree, out),
        UseTree::Name(name) => out.push(name.ident.to_string()),
        UseTree::Rename(rename) => out.push(rename.rename.to_string()),
        UseTree::Glob(_) => {}
        UseTree::Group(group) => {
            for tree in &group.items {
                collect_use_names(tree, out);
            }
        }
    }
}

fn type_to_string(ty: &Type) -> String {
    use quote::ToTokens;
    clean_token_string(ty.to_token_stream().to_string())
}

/// `quote!`'s Display always separates tokens with a space; clean up the
/// common punctuation so `Option < String >` reads as `Option<String>`.
fn clean_token_string(raw: String) -> String {
    let mut cleaned = raw;
    for (pattern, replacement) in [
        (" ::", "::"),
        (":: ", "::"),
        (" ,", ","),
        (" <", "<"),
        ("< ", "<"),
        (" >", ">"),
        ("> ", ">"),
        (" (", "("),
        ("( ", "("),
        (" )", ")"),
    ] {
        cleaned = cleaned.replace(pattern, replacement);
    }
    cleaned
}

trait AttrExt {
    fn to_token_stream_string(&self) -> String;
}

impl AttrExt for syn::Attribute {
    fn to_token_stream_string(&self) -> String {
        use quote::ToTokens;
        self.to_token_stream().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extracts_declared_and_bare_defaults() {
        let source = r#"
            #[derive(Props, Clone, PartialEq)]
            pub struct WidgetProps {
                pub open: Signal<bool>,
                #[props(default = false)]
                pub disabled: bool,
                #[props(default)]
                pub label: String,
            }
        "#;
        let mut file = tempfile::NamedTempFile::new().expect("tempfile");
        file.write_all(source.as_bytes()).expect("write fixture");
        let introspection = introspect_file(file.path());
        assert!(introspection.exists);
        let fields = introspection
            .props
            .get("WidgetProps")
            .expect("WidgetProps struct introspected");

        let open = fields.iter().find(|f| f.name == "open").unwrap();
        assert_eq!(open.default, None);

        let disabled = fields.iter().find(|f| f.name == "disabled").unwrap();
        assert_eq!(disabled.default.as_deref(), Some("false"));

        let label = fields.iter().find(|f| f.name == "label").unwrap();
        assert_eq!(label.default.as_deref(), Some("default()"));
    }

    #[test]
    fn extracts_enum_variants_and_default() {
        let source = r#"
            #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
            pub enum WidgetVariant {
                /// Primary treatment.
                #[default]
                Default,
                Destructive,
                Outline,
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum WidgetAlignment {
                Start,
                Center,
                End,
            }
        "#;
        let mut file = tempfile::NamedTempFile::new().expect("tempfile");
        file.write_all(source.as_bytes()).expect("write fixture");
        let introspection = introspect_file(file.path());
        assert!(introspection.exists);

        let variant_enum = introspection
            .enums
            .get("WidgetVariant")
            .expect("WidgetVariant enum introspected");
        assert_eq!(
            variant_enum.variants,
            vec!["Default", "Destructive", "Outline"]
        );
        assert_eq!(variant_enum.default_variant.as_deref(), Some("Default"));

        let alignment_enum = introspection
            .enums
            .get("WidgetAlignment")
            .expect("WidgetAlignment enum introspected");
        assert_eq!(alignment_enum.variants, vec!["Start", "Center", "End"]);
        assert_eq!(alignment_enum.default_variant, None);
    }
}
