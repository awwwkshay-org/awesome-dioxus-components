//! `catalog fetch shadcn`: inventory + composition + props for shadcn's
//! `apps/v4/registry/new-york-v4/ui/*.tsx` registry source.
//!
//! shadcn ships no first-party prop tables (its docs literally defer to the
//! wrapped primitive's own docs) -- confirmed by hand against the live
//! Dialog page and source before this was built. Its own source is almost
//! entirely `React.ComponentProps<typeof X>` passthrough, with the
//! occasional `& { extraProp: T }` augmentation (e.g. `DialogContent`'s
//! `showCloseButton`). This module regex-scans each component's exported
//! function signatures for that one pattern family rather than pulling in a
//! full TSX parser, and resolves the wrapped `X` back to an axis+component
//! via that same file's own `import` statement -- shadcn is (as of this
//! writing) mid-migration from `@radix-ui/*`/`radix-ui` to Base UI, so which
//! axis a given component points at varies file to file; this is recorded
//! as observed, not normalized to one axis.

use std::collections::BTreeMap;
use std::time::Duration;

use regex::Regex;

use super::case;
use super::schema::{CatalogEntry, CatalogSnapshot, CompositionRef, PartEntry, Prop, PropsSource};

const OWNER: &str = "shadcn-ui";
const REPO: &str = "ui";
const REGISTRY_DIR: &str = "apps/v4/registry/new-york-v4/ui";

pub fn fetch(_revision: Option<&str>) -> Result<CatalogSnapshot, String> {
    let sha = resolve_sha()?;
    let slugs = list_component_slugs(&sha)?;
    if slugs.is_empty() {
        return Err(format!(
            "found no .tsx files under {REGISTRY_DIR} at {sha}; the registry layout may have changed"
        ));
    }

    let mut entries = Vec::with_capacity(slugs.len());
    for slug in &slugs {
        entries.push(fetch_component_entry(&sha, slug)?);
    }

    Ok(CatalogSnapshot {
        axis: "shadcn".to_string(),
        source: format!("https://github.com/{OWNER}/{REPO}/tree/{sha}/{REGISTRY_DIR}"),
        revision: sha,
        refreshed_at: crate::today(),
        entries,
    })
}

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("adico-xtask/1.0")
        .build()
        .map_err(|error| format!("cannot build HTTP client: {error}"))
}

fn github_api_get(url: &str) -> Result<serde_json::Value, String> {
    let client = http_client()?;
    let mut request = client
        .get(url)
        .header("Accept", "application/vnd.github+json");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    let response = request
        .send()
        .map_err(|error| format!("cannot reach GitHub API at {url}: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned {} for {url}",
            response.status()
        ));
    }
    response
        .json()
        .map_err(|error| format!("invalid GitHub API response from {url}: {error}"))
}

fn resolve_sha() -> Result<String, String> {
    let repo_info = github_api_get(&format!("https://api.github.com/repos/{OWNER}/{REPO}"))?;
    let default_branch = repo_info
        .get("default_branch")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "GitHub API response missing default_branch".to_string())?;
    let commit = github_api_get(&format!(
        "https://api.github.com/repos/{OWNER}/{REPO}/commits/{default_branch}"
    ))?;
    commit
        .get("sha")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| "GitHub API response missing sha".to_string())
}

fn list_component_slugs(sha: &str) -> Result<Vec<String>, String> {
    let url =
        format!("https://api.github.com/repos/{OWNER}/{REPO}/contents/{REGISTRY_DIR}?ref={sha}");
    let listing = github_api_get(&url)?;
    let array = listing
        .as_array()
        .ok_or_else(|| format!("expected an array response from {url}"))?;
    let mut slugs: Vec<String> = array
        .iter()
        .filter_map(|entry| entry.get("name").and_then(|n| n.as_str()))
        .filter_map(|name| name.strip_suffix(".tsx"))
        .map(str::to_string)
        .collect();
    slugs.sort();
    Ok(slugs)
}

fn fetch_raw(sha: &str, slug: &str) -> Result<String, String> {
    let url =
        format!("https://raw.githubusercontent.com/{OWNER}/{REPO}/{sha}/{REGISTRY_DIR}/{slug}.tsx");
    let client = http_client()?;
    let response = client
        .get(&url)
        .send()
        .map_err(|error| format!("cannot fetch {url}: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("{url} returned {}", response.status()));
    }
    response
        .text()
        .map_err(|error| format!("cannot read response body from {url}: {error}"))
}

fn fetch_component_entry(sha: &str, slug: &str) -> Result<CatalogEntry, String> {
    let source = fetch_raw(sha, slug)?;
    let parts = parse_shadcn_source(slug, &source);
    Ok(CatalogEntry {
        id: slug.to_string(),
        name: case::pascal_case(slug),
        parts,
    })
}

/// Maps an imported alias (e.g. `DialogPrimitive`) back to the axis +
/// component that alias's module import resolves to, using the file's own
/// `import { X as Alias } from "module"` / `import { X } from "module"`
/// statements. Not a real module resolver -- string matching on the import
/// specifier is enough for the small set of primitive packages shadcn uses.
fn resolve_import_axis(source: &str, alias: &str) -> Option<(String, String)> {
    let import_re = Regex::new(r#"import\s*\{([^}]*)\}\s*from\s*["']([^"']+)["']"#).ok()?;
    for capture in import_re.captures_iter(source) {
        let names = &capture[1];
        let module = &capture[2];
        for raw_name in names.split(',') {
            let raw_name = raw_name.trim();
            if raw_name.is_empty() {
                continue;
            }
            let (imported, local) = match raw_name.split_once(" as ") {
                Some((imported, local)) => (imported.trim(), local.trim()),
                None => (raw_name, raw_name),
            };
            if local == alias {
                let axis = if module.contains("radix-ui") {
                    "radix"
                } else if module.contains("base-ui") {
                    "base-ui"
                } else {
                    module
                };
                return Some((axis.to_string(), case::kebab_case(imported)));
            }
        }
    }
    None
}

struct PassthroughMatch {
    /// `Some(alias, part)` for `React.ComponentProps<typeof Alias.Part>`, or
    /// `None` for `React.ComponentProps<"tag">` (no tracked primitive).
    wraps: Option<(String, Option<String>)>,
    augmentation: Vec<Prop>,
    /// The local `cva()` alias referenced via `VariantProps<typeof Alias>`,
    /// if this signature's type expression declares one.
    variant_alias: Option<String>,
}

fn parse_type_expression(type_expr: &str) -> Option<PassthroughMatch> {
    let typeof_re = Regex::new(r"typeof\s+(\w+)(?:\.(\w+))?").ok()?;
    let augmentation_re = Regex::new(r"&\s*\{([\s\S]*)\}\s*$").ok()?;
    let variant_props_re = Regex::new(r"VariantProps<typeof\s+(\w+)>").ok()?;

    let wraps = typeof_re.captures(type_expr).map(|capture| {
        (
            capture[1].to_string(),
            capture.get(2).map(|m| m.as_str().to_string()),
        )
    });

    let variant_alias = variant_props_re
        .captures(type_expr)
        .map(|capture| capture[1].to_string());

    if wraps.is_none() && variant_alias.is_none() && !type_expr.contains("ComponentProps") {
        return None;
    }

    let augmentation = augmentation_re
        .captures(type_expr)
        .map(|capture| parse_augmentation_fields(&capture[1]))
        .unwrap_or_default();

    Some(PassthroughMatch {
        wraps,
        augmentation,
        variant_alias,
    })
}

/// Finds the index of the byte matching `open` at `open_idx`, tracking
/// string-literal boundaries (`"`, `'`, `` ` ``) so a brace/paren inside a
/// CSS-class string (e.g. Tailwind's `hover:bg-primary/90`) never throws off
/// the depth count.
fn matching_close(bytes: &[u8], open_idx: usize, open: u8, close: u8) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_string: Option<u8> = None;
    let mut i = open_idx;
    while i < bytes.len() {
        let byte = bytes[i];
        if let Some(quote) = in_string {
            if byte == b'\\' {
                i += 2;
                continue;
            }
            if byte == quote {
                in_string = None;
            }
        } else if byte == b'"' || byte == b'\'' || byte == b'`' {
            in_string = Some(byte);
        } else if byte == open {
            depth += 1;
        } else if byte == close {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Finds `<name>: {` in `body` and returns the balanced-brace content
/// between (and excluding) the matching `{`/`}`.
fn extract_named_object<'a>(body: &'a str, name: &str) -> Option<&'a str> {
    let key_re = Regex::new(&format!(r"{}\s*:\s*\{{", regex::escape(name))).ok()?;
    let capture = key_re.find(body)?;
    let open_idx = capture.end() - 1;
    let close_idx = matching_close(body.as_bytes(), open_idx, b'{', b'}')?;
    Some(&body[open_idx + 1..close_idx])
}

/// Splits a `{ key: { ... }, key2: { ... } }` object body into `(key,
/// inner_body)` pairs in declaration order, using balanced-brace scanning so
/// nested content never gets mistaken for a sibling key.
fn split_object_entries(body: &str) -> Vec<(String, String)> {
    let key_re = Regex::new(r"(\w+)\s*:\s*\{").unwrap();
    let bytes = body.as_bytes();
    let mut result = Vec::new();
    let mut pos = 0usize;
    while pos < body.len() {
        let Some(capture) = key_re.captures(&body[pos..]) else {
            break;
        };
        let whole = capture.get(0).unwrap();
        let key = capture[1].to_string();
        let open_idx = pos + whole.end() - 1;
        let Some(close_idx) = matching_close(bytes, open_idx, b'{', b'}') else {
            break;
        };
        result.push((key, body[open_idx + 1..close_idx].to_string()));
        pos = close_idx + 1;
    }
    result
}

/// Extracts top-level `identifier:` keys from an object body, skipping
/// anything inside a string literal so a Tailwind class like
/// `hover:bg-primary/90` is never mistaken for a `hover` key.
fn parse_object_keys(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let mut keys = Vec::new();
    let mut i = 0usize;
    let mut in_string: Option<u8> = None;
    while i < bytes.len() {
        let byte = bytes[i];
        if let Some(quote) = in_string {
            if byte == b'\\' {
                i += 2;
                continue;
            }
            if byte == quote {
                in_string = None;
            }
            i += 1;
            continue;
        }
        if byte == b'"' || byte == b'\'' || byte == b'`' {
            in_string = Some(byte);
            i += 1;
            continue;
        }
        if byte.is_ascii_alphabetic() || byte == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let ident = &body[start..i];
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b':' {
                keys.push(ident.to_string());
                i = j + 1;
            }
            continue;
        }
        i += 1;
    }
    keys
}

/// Extracts `key: "value"` pairs (e.g. a `defaultVariants` block) into a
/// name -> value map.
fn parse_string_value_pairs(body: &str) -> BTreeMap<String, String> {
    let pair_re = Regex::new(r#"(\w+)\s*:\s*"([^"]*)""#).unwrap();
    pair_re
        .captures_iter(body)
        .map(|capture| (capture[1].to_string(), capture[2].to_string()))
        .collect()
}

/// Parses a `cva()` call's `variants: { <group>: { <key>: <value>, ... },
/// ... }` object into one [`Prop`] per group -- `name` is the group name,
/// `type_name` the literal union of its keys, `default` the matching
/// `defaultVariants` entry, if any.
fn parse_cva_variant_groups(source: &str, alias: &str) -> Vec<Prop> {
    let Ok(const_re) = Regex::new(&format!(r"const\s+{}\s*=\s*cva\s*\(", regex::escape(alias)))
    else {
        return Vec::new();
    };
    let Some(mat) = const_re.find(source) else {
        return Vec::new();
    };
    let call_open = mat.end() - 1;
    let Some(call_close) = matching_close(source.as_bytes(), call_open, b'(', b')') else {
        return Vec::new();
    };
    let call_body = &source[call_open + 1..call_close];

    let Some(variants_body) = extract_named_object(call_body, "variants") else {
        return Vec::new();
    };
    let defaults = extract_named_object(call_body, "defaultVariants")
        .map(parse_string_value_pairs)
        .unwrap_or_default();

    split_object_entries(variants_body)
        .into_iter()
        .map(|(group_name, group_body)| {
            let keys = parse_object_keys(&group_body);
            let type_name = keys
                .iter()
                .map(|key| format!("\"{key}\""))
                .collect::<Vec<_>>()
                .join(" | ");
            let default = defaults.get(&group_name).cloned();
            Prop {
                name: group_name,
                type_name,
                default,
                description: None,
            }
        })
        .collect()
}

fn parse_augmentation_fields(body: &str) -> Vec<Prop> {
    let field_re = Regex::new(r"^\s*(\w+)(\??):\s*(.+?)\s*$").unwrap();
    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim().trim_end_matches(',');
            if trimmed.is_empty() {
                return None;
            }
            let captures = field_re.captures(trimmed)?;
            Some(Prop {
                name: captures[1].to_string(),
                type_name: captures[3].to_string(),
                default: None,
                description: None,
            })
        })
        .collect()
}

fn parse_shadcn_source(slug: &str, source: &str) -> Vec<PartEntry> {
    let signature_re = Regex::new(r"function\s+(\w+)\s*\(\s*\{[^{}]*\}\s*:\s*([^)]+)\)").unwrap();

    let component_prefix = case::pascal_case(slug);
    let mut parts_by_name: BTreeMap<String, PartEntry> = BTreeMap::new();

    for capture in signature_re.captures_iter(source) {
        let function_name = capture[1].to_string();
        let type_expr = capture[2].trim();
        let part_id = case::part_id_for(&component_prefix, &function_name);

        let Some(matched) = parse_type_expression(type_expr) else {
            parts_by_name.insert(
                part_id.clone(),
                PartEntry {
                    id: part_id,
                    composition: Vec::new(),
                    props_source: PropsSource::Unavailable,
                },
            );
            continue;
        };

        let composition = matched
            .wraps
            .as_ref()
            .and_then(|(alias, part)| {
                resolve_import_axis(source, alias).map(|axis_ref| (axis_ref, part))
            })
            .map(|((axis, component), part)| {
                vec![CompositionRef {
                    axis,
                    component,
                    part: part.as_deref().map(case::kebab_case),
                }]
            })
            .unwrap_or_default();

        let mut explicit_props = matched
            .variant_alias
            .as_ref()
            .map(|alias| parse_cva_variant_groups(source, alias))
            .unwrap_or_default();
        explicit_props.extend(matched.augmentation);

        let props_source = if !explicit_props.is_empty() {
            PropsSource::Explicit {
                props: explicit_props,
            }
        } else if let Some(composition_ref) = composition.first() {
            let part = composition_ref
                .part
                .clone()
                .unwrap_or_else(|| "root".to_string());
            PropsSource::InheritsFrom {
                reference: format!(
                    "{}.{}.{}",
                    composition_ref.axis, composition_ref.component, part
                ),
            }
        } else {
            PropsSource::Unavailable
        };

        parts_by_name.insert(
            part_id.clone(),
            PartEntry {
                id: part_id,
                composition,
                props_source,
            },
        );
    }

    parts_by_name.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIALOG_FIXTURE: &str = r#"
import * as React from "react"
import { Dialog as DialogPrimitive } from "radix-ui"

function DialogTrigger({
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Trigger>) {
  return <DialogPrimitive.Trigger data-slot="dialog-trigger" {...props} />
}

function DialogContent({
  className,
  children,
  showCloseButton = true,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Content> & {
  showCloseButton?: boolean
}) {
  return <div />
}
"#;

    #[test]
    fn trigger_is_pure_passthrough() {
        let parts = parse_shadcn_source("dialog", DIALOG_FIXTURE);
        let trigger = parts
            .iter()
            .find(|p| p.id == "trigger")
            .expect("trigger part");
        match &trigger.props_source {
            PropsSource::InheritsFrom { reference } => {
                assert_eq!(reference, "radix.dialog.trigger")
            }
            other => panic!("expected inherits_from, got {other:?}"),
        }
        assert_eq!(trigger.composition.len(), 1);
        assert_eq!(trigger.composition[0].axis, "radix");
        assert_eq!(trigger.composition[0].component, "dialog");
    }

    #[test]
    fn content_has_explicit_augmentation_and_composition() {
        let parts = parse_shadcn_source("dialog", DIALOG_FIXTURE);
        let content = parts
            .iter()
            .find(|p| p.id == "content")
            .expect("content part");
        match &content.props_source {
            PropsSource::Explicit { props } => {
                assert_eq!(props.len(), 1);
                assert_eq!(props[0].name, "showCloseButton");
            }
            other => panic!("expected explicit, got {other:?}"),
        }
        assert_eq!(content.composition.len(), 1);
        assert_eq!(content.composition[0].component, "dialog");
        assert_eq!(content.composition[0].part.as_deref(), Some("content"));
    }

    #[test]
    fn no_match_yields_unavailable() {
        let parts = parse_shadcn_source(
            "widget",
            "function WidgetHeader({ className, ...props }: { className?: string }) { return <div/> }",
        );
        let header = parts
            .iter()
            .find(|p| p.id == "header")
            .expect("header part");
        assert!(matches!(header.props_source, PropsSource::Unavailable));
    }

    const BUTTON_VARIANTS_CVA: &str = r#"
const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all disabled:pointer-events-none hover:bg-primary/90",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground shadow-xs hover:bg-primary/90",
        destructive: "bg-destructive text-white shadow-xs hover:bg-destructive/90",
        outline: "border bg-background shadow-xs hover:bg-accent",
        secondary: "bg-secondary text-secondary-foreground shadow-xs hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-9 px-4 py-2",
        sm: "h-8 px-3",
        lg: "h-10 px-6",
        icon: "size-9",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)
"#;

    #[test]
    fn parses_cva_variant_groups() {
        let props = parse_cva_variant_groups(BUTTON_VARIANTS_CVA, "buttonVariants");
        assert_eq!(props.len(), 2);

        let variant = &props[0];
        assert_eq!(variant.name, "variant");
        assert_eq!(
            variant.type_name,
            "\"default\" | \"destructive\" | \"outline\" | \"secondary\" | \"ghost\" | \"link\""
        );
        assert_eq!(variant.default.as_deref(), Some("default"));

        let size = &props[1];
        assert_eq!(size.name, "size");
        assert_eq!(size.type_name, "\"default\" | \"sm\" | \"lg\" | \"icon\"");
        assert_eq!(size.default.as_deref(), Some("default"));
    }

    #[test]
    fn button_signature_merges_variant_groups_and_augmentation() {
        let source = format!(
            "{BUTTON_VARIANTS_CVA}\nfunction Button({{\n  className,\n  variant,\n  size,\n  asChild = false,\n  ...props\n}}: React.ComponentProps<\"button\"> &\n  VariantProps<typeof buttonVariants> & {{\n  asChild?: boolean\n}}) {{ return <button /> }}\n"
        );
        let parts = parse_shadcn_source("button", &source);
        let root = parts.iter().find(|p| p.id == "root").expect("root part");
        match &root.props_source {
            PropsSource::Explicit { props } => {
                let names: Vec<&str> = props.iter().map(|p| p.name.as_str()).collect();
                assert_eq!(names, vec!["variant", "size", "asChild"]);
            }
            other => panic!("expected explicit, got {other:?}"),
        }
    }

    /// A native-tag passthrough combined with a `cva()` variant group (no
    /// `React.ComponentProps<typeof OtherComponent>` reference at all) is
    /// the shape that keeps most non-`Button` parts among the 139
    /// `unavailable` parts today (e.g. shadcn's real `alert.tsx`). The
    /// `typeof alertVariants` inside `VariantProps<...>` also happens to
    /// match `parse_type_expression`'s unrelated `typeof` composition
    /// regex, but since `alertVariants` is a local `const`, not an import,
    /// `resolve_import_axis` returns `None` for it and composition stays
    /// empty -- only the variant-group extraction fires.
    #[test]
    fn native_tag_passthrough_with_cva_variant_groups_is_explicit() {
        let source = r#"
const alertVariants = cva(
  "relative w-full rounded-lg border px-4 py-3 text-sm",
  {
    variants: {
      variant: {
        default: "bg-card text-card-foreground",
        destructive: "text-destructive bg-card",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

function Alert({
  className,
  variant,
  ...props
}: React.ComponentProps<"div"> & VariantProps<typeof alertVariants>) {
  return <div />
}
"#;
        let parts = parse_shadcn_source("alert", source);
        let root = parts.iter().find(|p| p.id == "root").expect("root part");
        assert!(
            root.composition.is_empty(),
            "a local cva alias must not be treated as upstream composition"
        );
        match &root.props_source {
            PropsSource::Explicit { props } => {
                assert_eq!(props.len(), 1);
                assert_eq!(props[0].name, "variant");
                assert_eq!(props[0].type_name, "\"default\" | \"destructive\"");
            }
            other => panic!("expected explicit, got {other:?}"),
        }
    }
}
