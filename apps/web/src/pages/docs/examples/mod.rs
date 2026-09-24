//! Live, per-component examples for `/docs/components/:name`.
//!
//! # Why the code shown is the code that ran
//!
//! Each example module embeds its own source with
//! `include_str!` and wraps each example body in
//! `// doc-example:start <id>` / `// doc-example:end` markers. [`extract`]
//! slices that region back out at render time.
//!
//! So the snippet on the page is literally the bytes the compiler compiled.
//! There is no generated artifact, no `sync` command, and no CI gate, because
//! there is nothing that *could* fall out of date — the alternative designs
//! (an xtask emitting JSON, checked in CI) all leave a window where the
//! committed snippet is stale and rely on someone noticing.
//!
//! # Why each example is a component
//!
//! Examples call hooks (a controlled `Tabs`, an open `Dialog`). If they were
//! spliced into `DocsComponent`'s own body, the page's hook count would vary
//! with the `:name` route parameter and navigating between two components with
//! different example counts would violate hook ordering. Each example is a
//! real `#[component]`, so it owns its hook list.
//!
//! # Adding examples for a component
//!
//! Add `<item_name>.rs` here, following `button.rs`, and add one arm to
//! [`for_item`]. Components with no module fall through to the prose-and-props
//! page — nothing errors, and no page is ever worse than before.

use dioxus::prelude::*;

mod accordion;
mod alert;
mod badge;
mod button;
mod card;
mod checkbox;
mod dialog;
mod dropdown_menu;
mod input;
mod label;
mod select;
mod switch;
mod table;
mod tabs;
mod textarea;
mod tooltip;

/// One example's identity. The rendered body lives in the module's `render`,
/// and its source is recovered from the module's `SRC` by [`extract`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocExampleMeta {
    /// Matches the `// doc-example:start <id>` marker and the module's
    /// `render` match arm.
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

/// Everything the docs page needs to render one component's examples.
pub struct ItemExamples {
    pub metas: &'static [DocExampleMeta],
    pub render: fn(&str) -> Element,
    /// The example module's own source, via `include_str!`.
    pub source: &'static str,
}

/// Pulls the source region an example was written in, dedented.
///
/// Returns `None` when the markers are absent or unbalanced, so a mis-marked
/// example degrades to "no code shown" rather than to *wrong* code shown.
pub fn extract(source: &str, id: &str) -> Option<String> {
    let open = format!("// doc-example:start {id}\n");
    let start = source.find(&open)? + open.len();
    let len = source[start..].find("// doc-example:end")?;
    let body = &source[start..start + len];

    // Dedent by the common indentation of non-blank lines, so a body nested
    // two levels inside a component reads as top-level code.
    let indent = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    let dedented = body
        .lines()
        .map(|line| {
            if line.len() >= indent {
                &line[indent..]
            } else {
                line.trim_start()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let trimmed = dedented.trim().to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}

/// The examples authored for `name`, or `None` if there are none yet.
pub fn for_item(name: &str) -> Option<ItemExamples> {
    macro_rules! item {
        ($module:ident) => {
            Some(ItemExamples {
                metas: $module::METAS,
                render: $module::render,
                source: $module::SRC,
            })
        };
    }
    match name {
        "accordion" => item!(accordion),
        "alert" => item!(alert),
        "badge" => item!(badge),
        "button" => item!(button),
        "card" => item!(card),
        "checkbox" => item!(checkbox),
        "dialog" => item!(dialog),
        "dropdown-menu" => item!(dropdown_menu),
        "input" => item!(input),
        "label" => item!(label),
        "select" => item!(select),
        "switch" => item!(switch),
        "table" => item!(table),
        "tabs" => item!(tabs),
        "textarea" => item!(textarea),
        "tooltip" => item!(tooltip),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{extract, for_item};

    const SAMPLE: &str = r#"
#[component]
fn Variants() -> Element {
    rsx! {
        // doc-example:start variants
        Button { variant: ButtonVariant::Primary, "Save" }
        Button { variant: ButtonVariant::Ghost, "Cancel" }
        // doc-example:end
    }
}
"#;

    #[test]
    fn a_marked_region_round_trips_dedented() {
        assert_eq!(
            extract(SAMPLE, "variants").unwrap(),
            "Button { variant: ButtonVariant::Primary, \"Save\" }\n\
             Button { variant: ButtonVariant::Ghost, \"Cancel\" }"
        );
    }

    #[test]
    fn an_unknown_id_yields_none() {
        assert!(extract(SAMPLE, "nope").is_none());
    }

    #[test]
    fn a_missing_end_marker_yields_none_rather_than_panicking() {
        let truncated = "// doc-example:start solo\nBody without an end marker\n";
        assert!(extract(truncated, "solo").is_none());
    }

    #[test]
    fn an_empty_region_yields_none() {
        let empty = "// doc-example:start blank\n\n// doc-example:end\n";
        assert!(extract(empty, "blank").is_none());
    }

    /// The real hazard: rsx! bodies are brace-heavy and multi-level. The
    /// extractor is purely marker-based, so nesting must not confuse it.
    #[test]
    fn a_nested_brace_heavy_body_survives_intact() {
        let nested = "    // doc-example:start nested\n    \
                      Card {\n        CardHeader {\n            CardTitle { \"T\" }\n        \
                      }\n    }\n    // doc-example:end\n";
        assert_eq!(
            extract(nested, "nested").unwrap(),
            "Card {\n    CardHeader {\n        CardTitle { \"T\" }\n    }\n}"
        );
    }

    /// Every id a module advertises must actually resolve to source, or the
    /// page would show an example with a silently empty Code tab.
    #[test]
    fn every_advertised_example_id_resolves_to_source() {
        let items = [
            "accordion",
            "alert",
            "badge",
            "button",
            "card",
            "checkbox",
            "dialog",
            "dropdown-menu",
            "input",
            "label",
            "select",
            "switch",
            "table",
            "tabs",
            "textarea",
            "tooltip",
        ];
        for item in items {
            let examples = for_item(item).unwrap_or_else(|| panic!("{item} has no examples"));
            assert!(
                !examples.metas.is_empty(),
                "{item} advertises an empty example list"
            );
            for meta in examples.metas {
                assert!(
                    extract(examples.source, meta.id).is_some(),
                    "{item}: example `{}` has no extractable source — check its \
                     `// doc-example:start {}` / `// doc-example:end` markers",
                    meta.id,
                    meta.id,
                );
            }
        }
    }

    #[test]
    fn an_item_without_a_module_falls_through() {
        assert!(for_item("virtual-list").is_none());
        assert!(for_item("not-a-component").is_none());
    }
}
