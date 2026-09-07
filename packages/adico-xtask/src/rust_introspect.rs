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
    /// Names from `components` whose function declares at least one generic
    /// type parameter (e.g. `pub fn Select<T: Clone + PartialEq + 'static>`)
    /// -- lets a caller (e.g. the playground control generator) tell a
    /// monomorphic component like `Button` apart from a generic one like
    /// `Select<T>`, which can't have a single generated demo default.
    pub generic: std::collections::BTreeSet<String>,
    /// Every `pub use adico_primitives::...` re-export found in this file,
    /// collected but not yet resolved or classified as a component -- see
    /// `resolve_reexported_components`. A private (non-`pub`) `use`, however
    /// it's aliased, is never collected here: it isn't part of this file's
    /// public surface, so a re-export scan naturally leaves it out. This
    /// field is populated for every caller of `introspect_file`/
    /// `introspect_directory`; only the playground control generator
    /// currently resolves it into `components`, so its presence changes no
    /// existing caller's behavior.
    pub primitive_reexports: Vec<PrimitiveReexport>,
}

/// One `pub use adico_primitives::...` re-export: the name in its defining
/// module (`source_name`), the module segment it's re-exported from (`None`
/// for a bare crate-root path like `adico_primitives::ContentAlign`), and
/// the local, public name a consumer of this file sees (`local_name` --
/// differs from `source_name` for a renamed re-export, e.g. dialog.rs's
/// `DialogRoot as Dialog`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrimitiveReexport {
    pub module: Option<String>,
    pub source_name: String,
    pub local_name: String,
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
        merged.generic.extend(single.generic);
        merged
            .primitive_reexports
            .extend(single.primitive_reexports);
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
                } else if is_component_fn(item_fn) {
                    if !item_fn.sig.generics.params.is_empty() {
                        result.generic.insert(name.clone());
                    }
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
                if is_public(&item_use.vis) {
                    collect_primitive_reexports_from_tree(
                        &item_use.tree,
                        ReexportPathState::Unmatched,
                        &mut result.primitive_reexports,
                    );
                }
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

/// Whether a function item is a genuine Dioxus component: public, PascalCase
/// name, returns `Element`. The single recognizer for "is this a
/// component" -- used both for a locally-defined `pub fn` in `walk_items`
/// and, via `find_and_classify_in_items`, for a name resolved from a
/// `pub use adico_primitives::...` re-export, so a re-exported component is
/// judged by exactly the same rule as a local one, never a separate one.
fn is_component_fn(item_fn: &syn::ItemFn) -> bool {
    is_public(&item_fn.vis)
        && item_fn
            .sig
            .ident
            .to_string()
            .starts_with(|c: char| c.is_ascii_uppercase())
        && returns_element(&item_fn.sig.output)
}

/// Which part of a `pub use` path is currently being walked while looking
/// for an `adico_primitives::<module>::{...}` (or bare crate-root
/// `adico_primitives::{...}`) re-export.
#[derive(Clone)]
enum ReexportPathState {
    /// Haven't yet matched `adico_primitives` as the first path segment.
    Unmatched,
    /// Matched `adico_primitives`; no further module segment seen yet.
    AtCrateRoot,
    /// Inside `adico_primitives::<module>`.
    InModule(String),
}

/// Walks one `pub use` item's tree, recording every name re-exported from
/// `adico_primitives` (crate-root or a specific module) into `out`. A path
/// that isn't rooted at `adico_primitives` at all contributes nothing.
fn collect_primitive_reexports_from_tree(
    tree: &UseTree,
    state: ReexportPathState,
    out: &mut Vec<PrimitiveReexport>,
) {
    match (tree, state) {
        (UseTree::Path(path), ReexportPathState::Unmatched) => {
            if path.ident == "adico_primitives" {
                collect_primitive_reexports_from_tree(
                    &path.tree,
                    ReexportPathState::AtCrateRoot,
                    out,
                );
            }
        }
        (UseTree::Path(path), ReexportPathState::AtCrateRoot) => {
            collect_primitive_reexports_from_tree(
                &path.tree,
                ReexportPathState::InModule(path.ident.to_string()),
                out,
            );
        }
        (UseTree::Path(path), state @ ReexportPathState::InModule(_)) => {
            // This crate's module layout is flat (no nested `mod.rs`
            // subdirectories), so a further path segment here is
            // unexpected -- keep descending under the same resolved module
            // rather than dropping the re-export.
            collect_primitive_reexports_from_tree(&path.tree, state, out);
        }
        (UseTree::Name(name), ReexportPathState::AtCrateRoot) => {
            out.push(PrimitiveReexport {
                module: None,
                source_name: name.ident.to_string(),
                local_name: name.ident.to_string(),
            });
        }
        (UseTree::Name(name), ReexportPathState::InModule(module)) => {
            out.push(PrimitiveReexport {
                module: Some(module),
                source_name: name.ident.to_string(),
                local_name: name.ident.to_string(),
            });
        }
        (UseTree::Rename(rename), ReexportPathState::AtCrateRoot) => {
            out.push(PrimitiveReexport {
                module: None,
                source_name: rename.ident.to_string(),
                local_name: rename.rename.to_string(),
            });
        }
        (UseTree::Rename(rename), ReexportPathState::InModule(module)) => {
            out.push(PrimitiveReexport {
                module: Some(module),
                source_name: rename.ident.to_string(),
                local_name: rename.rename.to_string(),
            });
        }
        (
            UseTree::Group(group),
            state @ (ReexportPathState::AtCrateRoot | ReexportPathState::InModule(_)),
        ) => {
            for item in &group.items {
                collect_primitive_reexports_from_tree(item, state.clone(), out);
            }
        }
        _ => {}
    }
}

/// A `pub use adico_primitives::...` re-export whose name could not be
/// found in its defining module at all -- renamed, moved, or deleted
/// upstream, or the module file itself is unreadable or fails to parse.
/// Carries enough detail for a caller to surface a hard, specific error
/// rather than silently dropping the component; see
/// `resolve_reexported_components`.
#[derive(Debug)]
pub struct UnresolvedReexport {
    pub local_name: String,
    pub source_name: String,
    pub module: Option<String>,
    pub reason: String,
}

impl std::fmt::Display for UnresolvedReexport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = match &self.module {
            Some(module) => format!("adico_primitives::{module}"),
            None => "adico_primitives's crate root".to_string(),
        };
        write!(
            f,
            "re-export `{}` (as `{}`) could not be resolved in {location}: {}",
            self.source_name, self.local_name, self.reason
        )
    }
}

/// Loads and parses the primitives module file a re-export's `module`
/// resolves to (`lib.rs` for a bare crate-root path). `primitives_src_dir`
/// is `packages/adico-primitives/src`, whose flat layout means every
/// `pub mod <name>;` maps directly to `<name>.rs` in that directory.
fn load_primitive_module(
    primitives_src_dir: &Path,
    module: Option<&str>,
) -> Result<syn::File, String> {
    let file_name = module.unwrap_or("lib");
    let path = primitives_src_dir.join(format!("{file_name}.rs"));
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    syn::parse_file(&source).map_err(|error| format!("cannot parse {}: {error}", path.display()))
}

/// Looks up `name` among a parsed file's items (recursing into non-test
/// `mod` blocks, matching `walk_items`'s own recursion), classifying it by
/// `is_component_fn`. `Some(true)` means it's a genuine component;
/// `Some(false)` means the name exists but as something else (a struct, an
/// enum, or a non-component fn such as a hook); `None` means nothing by
/// that name exists at all -- the caller's "unresolvable" case.
fn find_and_classify_in_items(items: &[Item], name: &str) -> Option<bool> {
    for item in items {
        match item {
            Item::Fn(item_fn) if item_fn.sig.ident == name => {
                return Some(is_component_fn(item_fn));
            }
            Item::Struct(item_struct) if item_struct.ident == name => return Some(false),
            Item::Enum(item_enum) if item_enum.ident == name => return Some(false),
            // A type alias, const, trait, or union sharing the re-exported
            // name is unambiguously not a component -- found here (`Some`),
            // not the "no item by that name found" (`None`) case. Found via
            // a real re-export, `color_picker.rs`'s `pub type Color =
            // Srgb<u8>;`, hard-failing `sync` before this arm existed.
            Item::Type(item_type) if item_type.ident == name => return Some(false),
            Item::Const(item_const) if item_const.ident == name => return Some(false),
            Item::Trait(item_trait) if item_trait.ident == name => return Some(false),
            Item::Union(item_union) if item_union.ident == name => return Some(false),
            Item::Mod(item_mod) => {
                if let Some((_, nested)) = &item_mod.content
                    && let Some(found) = find_and_classify_in_items(nested, name)
                {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    // `<Name>PropsWithOwner` is generated by Dioxus's `#[component]`/`Props`
    // derive macro for a `<Name>Props` struct -- it exists only after macro
    // expansion, invisible to this syntax-only parse. Recognize it by its
    // fixed naming convention, paired with the literal `<Name>Props` struct
    // it's generated from, rather than erroring on a name this tool can
    // never see as written source. Found via a real re-export, toast.rs's
    // `ToastPropsWithOwner` (Dioxus's macro-generated companion to
    // `ToastProps`), hard-failing `sync` before this fallback existed.
    if let Some(base) = name.strip_suffix("PropsWithOwner")
        && find_and_classify_in_items(items, &format!("{base}Props")).is_some()
    {
        return Some(false);
    }
    None
}

/// Resolves every `pub use adico_primitives::...` re-export in `reexports`
/// into the subset that are genuine, renderable components -- returning
/// their *local* (public) names, ready to merge into
/// `FileIntrospection::components` (or an equivalent field a caller
/// maintains) exactly as a locally-defined component would be.
/// `primitives_src_dir` is `packages/adico-primitives/src`.
///
/// A re-export that resolves to a non-component item (a type, enum, or
/// hook) is excluded from the result -- that is the intended filter, not a
/// failure. A re-export that cannot be found at all (its module file is
/// unreadable, or nothing by that name exists in it) is a hard error: this
/// generator's existing "never silently drop" guarantee (`PropShape::Skipped`
/// reasons are always printed) extends to component discovery the same way,
/// rather than proceeding as if the component didn't exist.
pub fn resolve_reexported_components(
    reexports: &[PrimitiveReexport],
    primitives_src_dir: &Path,
) -> Result<Vec<String>, UnresolvedReexport> {
    let mut modules: BTreeMap<Option<String>, Result<syn::File, String>> = BTreeMap::new();
    let mut resolved = Vec::new();
    for reexport in reexports {
        let file_result = modules.entry(reexport.module.clone()).or_insert_with(|| {
            load_primitive_module(primitives_src_dir, reexport.module.as_deref())
        });
        let file = match file_result {
            Ok(file) => file,
            Err(reason) => {
                return Err(UnresolvedReexport {
                    local_name: reexport.local_name.clone(),
                    source_name: reexport.source_name.clone(),
                    module: reexport.module.clone(),
                    reason: reason.clone(),
                });
            }
        };
        match find_and_classify_in_items(&file.items, &reexport.source_name) {
            Some(true) => resolved.push(reexport.local_name.clone()),
            Some(false) => {}
            None => {
                return Err(UnresolvedReexport {
                    local_name: reexport.local_name.clone(),
                    source_name: reexport.source_name.clone(),
                    module: reexport.module.clone(),
                    reason: "no item by that name found".to_string(),
                });
            }
        }
    }
    Ok(resolved)
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

    #[test]
    fn flags_a_generic_component_and_leaves_a_monomorphic_one_unflagged() {
        let source = r#"
            #[component]
            pub fn Widget(props: WidgetProps) -> Element {
                rsx! {}
            }

            #[component]
            pub fn GenericWidget<T: Clone + PartialEq + 'static>(props: GenericWidgetProps<T>) -> Element {
                rsx! {}
            }
        "#;
        let mut file = tempfile::NamedTempFile::new().expect("tempfile");
        file.write_all(source.as_bytes()).expect("write fixture");
        let introspection = introspect_file(file.path());
        assert!(introspection.components.contains(&"Widget".to_string()));
        assert!(
            introspection
                .components
                .contains(&"GenericWidget".to_string())
        );
        assert!(!introspection.generic.contains("Widget"));
        assert!(introspection.generic.contains("GenericWidget"));
    }

    fn fixture_file(source: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().expect("tempfile");
        file.write_all(source.as_bytes()).expect("write fixture");
        file
    }

    #[test]
    fn collects_plain_and_renamed_primitive_reexports_but_not_private_ones() {
        let source = r#"
            pub use adico_primitives::accordion::{Accordion, AccordionMulti};
            pub use adico_primitives::dialog::DialogRoot as Dialog;
            pub use adico_primitives::ContentAlign;
            use adico_primitives::dialog::DialogContent as DialogPrimitiveContent;
        "#;
        let file = fixture_file(source);
        let introspection = introspect_file(file.path());
        assert!(introspection.exists);
        assert_eq!(
            introspection.primitive_reexports,
            vec![
                PrimitiveReexport {
                    module: Some("accordion".to_string()),
                    source_name: "Accordion".to_string(),
                    local_name: "Accordion".to_string(),
                },
                PrimitiveReexport {
                    module: Some("accordion".to_string()),
                    source_name: "AccordionMulti".to_string(),
                    local_name: "AccordionMulti".to_string(),
                },
                PrimitiveReexport {
                    module: Some("dialog".to_string()),
                    source_name: "DialogRoot".to_string(),
                    local_name: "Dialog".to_string(),
                },
                PrimitiveReexport {
                    module: None,
                    source_name: "ContentAlign".to_string(),
                    local_name: "ContentAlign".to_string(),
                },
            ]
        );
    }

    fn primitives_fixture_dir(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        for (name, source) in files {
            fs::write(dir.path().join(format!("{name}.rs")), source).expect("write module");
        }
        dir
    }

    fn reexport(module: Option<&str>, name: &str) -> PrimitiveReexport {
        PrimitiveReexport {
            module: module.map(str::to_string),
            source_name: name.to_string(),
            local_name: name.to_string(),
        }
    }

    #[test]
    fn resolves_an_inline_arg_component_reexport() {
        let dir = primitives_fixture_dir(&[(
            "accordion",
            "pub fn Accordion(children: Element) -> Element { rsx! {} }",
        )]);
        let resolved =
            resolve_reexported_components(&[reexport(Some("accordion"), "Accordion")], dir.path())
                .expect("resolves");
        assert_eq!(resolved, vec!["Accordion".to_string()]);
    }

    #[test]
    fn resolves_a_props_struct_style_component_reexport() {
        let dir = primitives_fixture_dir(&[(
            "card",
            "pub struct CardProps { pub children: Element }\n\
             pub fn Card(props: CardProps) -> Element { rsx! {} }",
        )]);
        let resolved = resolve_reexported_components(&[reexport(Some("card"), "Card")], dir.path())
            .expect("resolves");
        assert_eq!(resolved, vec!["Card".to_string()]);
    }

    #[test]
    fn excludes_an_enum_reexport() {
        let dir = primitives_fixture_dir(&[(
            "checkbox",
            "pub enum CheckboxState { Checked, Unchecked }",
        )]);
        let resolved = resolve_reexported_components(
            &[reexport(Some("checkbox"), "CheckboxState")],
            dir.path(),
        )
        .expect("resolves without error");
        assert!(resolved.is_empty());
    }

    #[test]
    fn excludes_a_type_alias_reexport() {
        // Regression test: `color_picker.rs` re-exports `Color`, a bare
        // `pub type Color = Srgb<u8>;` alias with no matching struct/enum/fn
        // -- this must resolve as "found, not a component," not as
        // unresolvable.
        let dir = primitives_fixture_dir(&[("color_picker", "pub type Color = u32;")]);
        let resolved =
            resolve_reexported_components(&[reexport(Some("color_picker"), "Color")], dir.path())
                .expect("a type alias resolves without error");
        assert!(resolved.is_empty());
    }

    #[test]
    fn excludes_a_macro_generated_props_with_owner_reexport() {
        // Regression test: `toast.rs` re-exports `ToastPropsWithOwner`, a
        // type Dioxus's component macro generates from `ToastProps` --
        // never written as a literal source item, so it must resolve via
        // the `<Name>Props` naming fallback, not as unresolvable.
        let dir = primitives_fixture_dir(&[(
            "toast",
            "pub struct ToastProps { pub title: String }\n\
             pub fn Toast(props: ToastProps) -> Element { rsx! {} }",
        )]);
        let resolved = resolve_reexported_components(
            &[reexport(Some("toast"), "ToastPropsWithOwner")],
            dir.path(),
        )
        .expect("a macro-generated PropsWithOwner type resolves without error");
        assert!(resolved.is_empty());
    }

    #[test]
    fn excludes_a_props_only_struct_reexport() {
        let dir =
            primitives_fixture_dir(&[("slider", "pub struct SliderProps { pub value: f64 }")]);
        let resolved =
            resolve_reexported_components(&[reexport(Some("slider"), "SliderProps")], dir.path())
                .expect("resolves without error");
        assert!(resolved.is_empty());
    }

    #[test]
    fn excludes_a_hook_reexport() {
        let dir =
            primitives_fixture_dir(&[("toast", "pub fn use_toast() -> ToastHandle { todo!() }")]);
        let resolved =
            resolve_reexported_components(&[reexport(Some("toast"), "use_toast")], dir.path())
                .expect("resolves without error");
        assert!(resolved.is_empty());
    }

    #[test]
    fn resolves_a_bare_crate_root_reexport_via_lib_fallback() {
        let dir =
            primitives_fixture_dir(&[("lib", "pub enum ContentAlign { Start, Center, End }")]);
        let resolved = resolve_reexported_components(&[reexport(None, "ContentAlign")], dir.path())
            .expect("resolves without error");
        assert!(resolved.is_empty());
    }

    #[test]
    fn errors_when_reexported_name_is_not_found_in_its_resolved_module() {
        let dir =
            primitives_fixture_dir(&[("accordion", "pub fn Accordion() -> Element { rsx! {} }")]);
        let error = resolve_reexported_components(
            &[reexport(Some("accordion"), "AccordionMulti")],
            dir.path(),
        )
        .expect_err("AccordionMulti does not exist in the fixture module");
        assert_eq!(error.local_name, "AccordionMulti");
        assert_eq!(error.module.as_deref(), Some("accordion"));
    }

    #[test]
    fn errors_when_the_resolved_module_file_does_not_exist() {
        let dir = primitives_fixture_dir(&[]);
        let error =
            resolve_reexported_components(&[reexport(Some("missing"), "Missing")], dir.path())
                .expect_err("missing.rs does not exist in the fixture dir");
        assert_eq!(error.local_name, "Missing");
        assert_eq!(error.module.as_deref(), Some("missing"));
    }
}
