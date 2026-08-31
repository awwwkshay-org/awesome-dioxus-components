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
}

#[derive(Debug, Serialize)]
pub struct PropField {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
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
                            })
                        })
                        .collect();
                    result.props.insert(item_struct.ident.to_string(), fields);
                }
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
    let raw = ty.to_token_stream().to_string();
    // `quote!`'s Display always separates tokens with a space; clean up the
    // common punctuation so `Option < String >` reads as `Option<String>`.
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
