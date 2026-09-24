//! Renders registry-authored prose, turning markdown inline-code spans into
//! real `<code>` elements.
//!
//! Every `documentation` field in `registry/registry.json` (description,
//! composition note, accessibility, keyboard) is written in prose that uses
//! backtick-delimited inline code — `ButtonVariant`, `<button>`,
//! `registry/ui/button.rs`. Rendering those fields as plain text, which is what
//! the docs route tree did before, puts literal backtick characters on screen.
//!
//! This is a splitter, not a markdown engine. It handles exactly three
//! constructs — inline code, `**strong**`, and `*emphasis*` — because those are
//! the three the prose on this site actually uses. Inline code came first, for
//! registry documentation; emphasis was added when the guide pages needed it,
//! which is the concrete need this file's earlier note said to wait for.
//!
//! Anything else (links, lists, headings) is still out of scope. Reach for a
//! real markdown parser when prose needs those, not before — this one compiles
//! into the wasm bundle.
//!
//! App-level wiring under `apps/web/src/components/`: it composes no registry
//! component and requires no change to `registry/ui/*.rs`.

use dioxus::prelude::*;

/// One piece of a prose string.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment<'a> {
    Text(&'a str),
    Code(&'a str),
    Strong(&'a str),
    Emphasis(&'a str),
}

/// Splits `input` into text, inline code, strong, and emphasis runs.
///
/// A delimiter only opens a span if the matching closer appears later on; an
/// unpaired one is literal text, so malformed prose degrades to exactly what a
/// plain-text renderer would have shown rather than swallowing the rest of the
/// line. Empty spans are dropped — there is nothing to render.
///
/// Inline code wins over emphasis, so `*` inside a code span stays literal.
fn segments(input: &str) -> Vec<Segment<'_>> {
    // Longest delimiter first: `**` must be tried before `*`, or every strong
    // span would be read as two empty emphasis spans.
    const DELIMS: &[&str] = &["`", "**", "*"];

    fn wrap<'a>(delim: &str, inner: &'a str) -> Segment<'a> {
        match delim {
            "`" => Segment::Code(inner),
            "**" => Segment::Strong(inner),
            _ => Segment::Emphasis(inner),
        }
    }

    let mut out = Vec::new();
    let mut rest = input;

    while !rest.is_empty() {
        // The earliest delimiter that actually has a closer. An unpaired one is
        // skipped here and falls through to the trailing-text push below.
        let mut best: Option<(usize, &str)> = None;
        for delim in DELIMS {
            let Some(open) = rest.find(delim) else {
                continue;
            };
            // `**` and `*` both match at the same index; the table order means
            // `**` is considered first and wins that tie.
            if best.is_some_and(|(best_open, _)| best_open <= open) {
                continue;
            }
            if rest[open + delim.len()..].contains(delim) {
                best = Some((open, delim));
            }
        }

        let Some((open, delim)) = best else {
            break;
        };

        let after = &rest[open + delim.len()..];
        let close = after.find(delim).expect("checked above");
        if open > 0 {
            out.push(Segment::Text(&rest[..open]));
        }
        let inner = &after[..close];
        if !inner.is_empty() {
            out.push(wrap(delim, inner));
        }
        rest = &after[close + delim.len()..];
    }

    if !rest.is_empty() {
        out.push(Segment::Text(rest));
    }
    out
}

/// The same inline rendering, without the wrapping `<p>`.
///
/// For places where the text is already inside an element that owns its own
/// typography — a heading, an alert title, a table cell — and a nested
/// paragraph would be wrong.
#[component]
pub fn ProseInline(text: String) -> Element {
    rsx! {
        for segment in segments(&text) {
            match segment {
                Segment::Text(value) => rsx! { "{value}" },
                Segment::Code(value) => rsx! {
                    code { class: "rounded border border-border/60 bg-muted px-[0.3em] py-[0.15em] font-mono text-[0.875em]",
                        "{value}"
                    }
                },
                Segment::Strong(value) => rsx! {
                    strong { class: "font-semibold", "{value}" }
                },
                Segment::Emphasis(value) => rsx! { em { class: "italic", "{value}" } },
            }
        }
    }
}

/// Prose with inline code and emphasis rendered as real elements.
///
/// `class` styles the wrapping `<p>`; inline code carries its own treatment so
/// callers do not have to restate it at every call site.
#[component]
pub fn Prose(text: String, #[props(default)] class: Option<String>) -> Element {
    let class = format!(
        "text-pretty {}",
        class.as_deref().unwrap_or("text-sm text-muted-foreground")
    );
    rsx! {
        p { class,
            for segment in segments(&text) {
                match segment {
                    Segment::Text(value) => rsx! { "{value}" },
                    Segment::Code(value) => rsx! {
                        code { class: "rounded border border-border/60 bg-muted px-[0.3em] py-[0.15em] font-mono text-[0.875em] text-foreground",
                            "{value}"
                        }
                    },
                    Segment::Strong(value) => rsx! {
                        strong { class: "font-semibold text-foreground", "{value}" }
                    },
                    Segment::Emphasis(value) => rsx! {
                        em { class: "italic", "{value}" }
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Segment, segments};

    #[test]
    fn plain_text_is_one_segment() {
        assert_eq!(
            segments("no code here"),
            vec![Segment::Text("no code here")]
        );
    }

    #[test]
    fn a_single_pair_splits_into_three() {
        assert_eq!(
            segments("a `Button` here"),
            vec![
                Segment::Text("a "),
                Segment::Code("Button"),
                Segment::Text(" here"),
            ]
        );
    }

    #[test]
    fn multiple_pairs_all_split() {
        assert_eq!(
            segments("`a` and `b`"),
            vec![
                Segment::Code("a"),
                Segment::Text(" and "),
                Segment::Code("b"),
            ]
        );
    }

    #[test]
    fn an_unpaired_trailing_backtick_stays_literal() {
        assert_eq!(
            segments("a `Button` and an orphan `"),
            vec![
                Segment::Text("a "),
                Segment::Code("Button"),
                Segment::Text(" and an orphan `"),
            ]
        );
    }

    #[test]
    fn an_empty_span_is_dropped() {
        assert_eq!(
            segments("before `` after"),
            vec![Segment::Text("before "), Segment::Text(" after")]
        );
    }

    #[test]
    fn an_empty_input_yields_nothing() {
        assert_eq!(segments(""), vec![]);
    }

    #[test]
    fn code_at_both_ends_needs_no_surrounding_text() {
        assert_eq!(segments("`only`"), vec![Segment::Code("only")]);
    }

    #[test]
    fn strong_and_emphasis_are_separate_segments() {
        assert_eq!(
            segments("it does **not** create *anything*"),
            vec![
                Segment::Text("it does "),
                Segment::Strong("not"),
                Segment::Text(" create "),
                Segment::Emphasis("anything"),
            ]
        );
    }

    /// `**` must win the tie against `*` at the same index, or every strong
    /// span would parse as two empty emphasis spans.
    #[test]
    fn a_double_star_is_not_read_as_two_single_stars() {
        assert_eq!(segments("**bold**"), vec![Segment::Strong("bold")]);
    }

    /// Code wins over emphasis, so a `*` inside a code span stays literal --
    /// which matters for paths like `registry/ui/*.rs`.
    #[test]
    fn a_star_inside_code_is_not_emphasis() {
        assert_eq!(
            segments("see `registry/ui/*.rs` for details"),
            vec![
                Segment::Text("see "),
                Segment::Code("registry/ui/*.rs"),
                Segment::Text(" for details"),
            ]
        );
    }

    #[test]
    fn an_unpaired_star_stays_literal() {
        assert_eq!(segments("2 * 3 = 6"), vec![Segment::Text("2 * 3 = 6")]);
    }

    #[test]
    fn code_and_emphasis_mix_in_one_string() {
        assert_eq!(
            segments("write it **above** the `adico:theme:start` marker"),
            vec![
                Segment::Text("write it "),
                Segment::Strong("above"),
                Segment::Text(" the "),
                Segment::Code("adico:theme:start"),
                Segment::Text(" marker"),
            ]
        );
    }

    /// The real shape this exists for: `registry/ui/button.rs`'s accessibility
    /// text, which mixes an HTML tag, a Rust path, and an attribute name.
    #[test]
    fn a_real_registry_accessibility_string_renders_every_span() {
        let input = "A native `<button>`; visible content is caller-composed \
                     through `children`, so icon-only usage should include an \
                     explicit `aria-label`.";
        let parsed = segments(input);
        let code: Vec<_> = parsed
            .iter()
            .filter_map(|segment| match segment {
                Segment::Code(value) => Some(*value),
                _ => None,
            })
            .collect();
        assert_eq!(code, vec!["<button>", "children", "aria-label"]);
        // No literal backtick survives in any text segment.
        for segment in &parsed {
            if let Segment::Text(value) = segment {
                assert!(
                    !value.contains('`'),
                    "text segment kept a backtick: {value}"
                );
            }
        }
    }
}
