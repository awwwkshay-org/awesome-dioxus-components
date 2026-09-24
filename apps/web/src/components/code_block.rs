//! A syntax-highlighted, copyable code block for docs examples.
//!
//! The highlighter is a small local lexer rather than `syntect` or
//! `tree-sitter`: this crate compiles to wasm, and shipping a general-purpose
//! highlighting engine to colour short RSX snippets is a poor trade. It is
//! deliberately approximate — it reads RSX and Rust well enough to scan, and it
//! is not a parser. A mis-coloured token is cosmetic.
//!
//! The one property that is *not* cosmetic is that highlighting must never
//! alter the code itself; [`tests::highlighting_never_alters_the_text`] holds
//! the lexer to that.
//!
//! App-level wiring under `apps/web/src/components/`: it composes the installed
//! `CopyButton` and requires no change to `registry/ui/*.rs`.

use dioxus::prelude::*;

use crate::components::ui::copy_button::CopyButton;

/// What a run of source text is, for colouring purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    Plain,
    Comment,
    Str,
    Keyword,
    /// A `CamelCase` identifier — component names, enums, types.
    Type,
    Number,
    /// A struct-field-ish `name:` at the head of an rsx attribute.
    Attr,
    Punct,
}

impl Token {
    fn class(self) -> &'static str {
        // Tokens borrow the theme's own palette rather than introducing a
        // separate code-colour scheme, so a theme or palette swap carries
        // through to snippets too.
        match self {
            Self::Plain => "",
            Self::Comment => "text-muted-foreground italic",
            Self::Str => "text-success",
            Self::Keyword => "text-info font-medium",
            Self::Type => "text-primary",
            Self::Number => "text-warning",
            Self::Attr => "text-foreground",
            Self::Punct => "text-muted-foreground",
        }
    }
}

const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "false", "fn", "for", "if",
    "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self",
    "static", "struct", "super", "trait", "true", "type", "use", "where", "while", "async",
    "await", "dyn",
];

/// Splits `source` into coloured runs.
///
/// Every byte of the input lands in exactly one run, in order — see the
/// round-trip test.
fn lex(source: &str) -> Vec<(Token, String)> {
    let bytes: Vec<char> = source.chars().collect();
    let mut out: Vec<(Token, String)> = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i];

        // Line comment (this also catches the doc-example markers, which is
        // fine — they are stripped before a snippet reaches here).
        if c == '/' && bytes.get(i + 1) == Some(&'/') {
            let start = i;
            while i < bytes.len() && bytes[i] != '\n' {
                i += 1;
            }
            out.push((Token::Comment, bytes[start..i].iter().collect()));
            continue;
        }

        // String literal, with escapes.
        if c == '"' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == '\\' {
                    i = (i + 2).min(bytes.len());
                    continue;
                }
                if bytes[i] == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            out.push((Token::Str, bytes[start..i].iter().collect()));
            continue;
        }

        // Identifier or keyword.
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_alphanumeric() || bytes[i] == '_') {
                i += 1;
            }
            let word: String = bytes[start..i].iter().collect();
            let token = if KEYWORDS.contains(&word.as_str()) {
                Token::Keyword
            } else if word.starts_with(|ch: char| ch.is_uppercase()) {
                Token::Type
            } else if bytes.get(i) == Some(&':') && bytes.get(i + 1) != Some(&':') {
                Token::Attr
            } else {
                Token::Plain
            };
            out.push((token, word));
            continue;
        }

        // Number.
        if c.is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_alphanumeric() || bytes[i] == '.') {
                i += 1;
            }
            out.push((Token::Number, bytes[start..i].iter().collect()));
            continue;
        }

        // Punctuation, and whitespace as plain.
        let token = if c.is_whitespace() {
            Token::Plain
        } else {
            Token::Punct
        };
        i += 1;
        // Merge with the previous run when it is the same kind, to keep the
        // emitted span count down.
        match out.last_mut() {
            Some((last, text)) if *last == token => text.push(c),
            _ => out.push((token, c.to_string())),
        }
    }

    out
}

/// Strips indentation that a source formatter added to a multi-line literal.
///
/// A multi-line raw string written flush-left inside a deeply nested `rsx!`
/// gets re-indented by `cargo fmt` — only its *continuation* lines, since the
/// first begins immediately after the opening quote. The literal's bytes
/// change, so the rendered snippet silently gains leading whitespace on every
/// line but the first.
///
/// This removes the indentation those lines share, which restores the intended
/// shape while preserving relative nesting inside the snippet. Text that is
/// already flush (anything from the examples extractor, which dedents on its
/// own) is returned unchanged.
fn normalize_indent(code: &str) -> String {
    let mut lines = code.lines();
    let Some(first) = lines.next() else {
        return String::new();
    };
    let rest: Vec<&str> = lines.collect();

    let common = rest
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    if common == 0 {
        return code.to_string();
    }

    let mut out = String::from(first);
    for line in rest {
        out.push('\n');
        out.push_str(if line.len() >= common {
            &line[common..]
        } else {
            line.trim_start()
        });
    }
    out
}

/// A highlighted code block with a copy button.
#[component]
pub fn CodeBlock(code: String, #[props(default)] class: Option<String>) -> Element {
    let code = normalize_indent(&code);
    let tokens = lex(&code);
    let class = format!(
        "group relative overflow-hidden rounded-lg border border-border bg-muted/40 {}",
        class.as_deref().unwrap_or_default()
    );
    rsx! {
        div { class,
            div { class: "absolute right-2 top-2 z-10",
                CopyButton { value: ReadSignal::new(Signal::new(code.clone())) }
            }
            pre { class: "overflow-x-auto p-4 pr-14 text-[13px] leading-relaxed",
                code { class: "font-mono",
                    for (token , text) in tokens {
                        span { class: token.class(), "{text}" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Token, lex, normalize_indent};

    /// The case this exists for: `cargo fmt` re-indenting a multi-line raw
    /// string's continuation lines inside a nested `rsx!`.
    #[test]
    fn formatter_added_indentation_is_stripped() {
        let mangled = ":root { --a: 1; }\n    .dark { --a: 2; }";
        assert_eq!(
            normalize_indent(mangled),
            ":root { --a: 1; }\n.dark { --a: 2; }"
        );
    }

    #[test]
    fn relative_nesting_inside_a_snippet_is_preserved() {
        let mangled =
            "Card {\n        CardHeader {\n            CardTitle { \"T\" }\n        }\n    }";
        assert_eq!(
            normalize_indent(mangled),
            "Card {\n    CardHeader {\n        CardTitle { \"T\" }\n    }\n}"
        );
    }

    #[test]
    fn already_flush_text_is_untouched() {
        for source in [
            "Button { \"Save\" }",
            "a\nb\nc",
            "a\n\nb",
            "",
            "single line only",
        ] {
            assert_eq!(normalize_indent(source), source);
        }
    }

    #[test]
    fn blank_continuation_lines_do_not_defeat_the_common_indent() {
        let mangled = "first\n\n    second\n    third";
        assert_eq!(normalize_indent(mangled), "first\n\nsecond\nthird");
    }

    fn rebuilt(source: &str) -> String {
        lex(source).into_iter().map(|(_, text)| text).collect()
    }

    /// The load-bearing property: colouring may never change the code. If this
    /// fails, the page is showing something other than what compiled.
    #[test]
    fn highlighting_never_alters_the_text() {
        let samples = [
            "Button { variant: ButtonVariant::Primary, \"Save\" }",
            "// a comment\nlet x = 42;\n",
            "Card {\n    CardHeader { CardTitle { \"Title\" } }\n}",
            "rsx! { div { class: \"flex gap-2\", \"text\" } }",
            "Button { size: ButtonSize::IconXs, aria_label: \"Add\", \"+\" }",
            "",
            "   \n\t  ",
            "\"escaped \\\" quote\"",
            "let n = 1.5e3;",
            "TabTrigger { value: \"tab1\".to_string(), index: 0usize, \"Tab 1\" }",
        ];
        for sample in samples {
            assert_eq!(rebuilt(sample), sample, "lexer altered: {sample:?}");
        }
    }

    #[test]
    fn an_unterminated_string_does_not_hang_or_panic() {
        let source = "Button { \"unterminated";
        assert_eq!(rebuilt(source), source);
    }

    #[test]
    fn camel_case_identifiers_are_types_and_lowercase_are_not() {
        let tokens = lex("Button { variant: x }");
        assert_eq!(tokens[0], (Token::Type, "Button".to_string()));
        assert!(
            tokens
                .iter()
                .any(|(t, s)| *t == Token::Attr && s == "variant")
        );
    }

    #[test]
    fn a_path_separator_is_not_mistaken_for_an_attribute() {
        // `ButtonVariant::Primary` — the `::` must not make `ButtonVariant`
        // read as an attribute name.
        let tokens = lex("ButtonVariant::Primary");
        assert_eq!(tokens[0], (Token::Type, "ButtonVariant".to_string()));
    }

    #[test]
    fn keywords_are_distinguished_from_identifiers() {
        let tokens = lex("let mut value = 1;");
        assert_eq!(tokens[0], (Token::Keyword, "let".to_string()));
        assert!(
            tokens
                .iter()
                .any(|(t, s)| *t == Token::Keyword && s == "mut")
        );
        // `value` is a plain identifier. Its run may carry trailing
        // whitespace: adjacent same-kind runs are merged to keep the emitted
        // span count down, and whitespace lexes as `Plain`.
        assert!(
            tokens
                .iter()
                .any(|(t, s)| *t == Token::Plain && s.trim() == "value"),
            "expected a plain `value` run, got {tokens:?}"
        );
    }
}
