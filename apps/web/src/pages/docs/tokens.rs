//! The theme tokens this project actually has installed, read from its own
//! `tailwind.css` at compile time.
//!
//! The token block is **not owned by this app**: `adico add` regenerates
//! everything between `adico:theme:start` and `adico:theme:end` wholesale
//! (`packages/adico-cli/src/css.rs`, `plan_theme_install`). A hand-written
//! table in the theming guide would therefore go stale the first time a
//! registry item introduces a token, and nothing would catch it.
//!
//! So the guide derives its table from the stylesheet instead. Installing an
//! item that adds a token updates the page with no edit to the page.

use std::sync::OnceLock;

/// The compile input, not the generated `assets/tailwind.css` output — the
/// input is where the readable `--name: value;` declarations live.
const TAILWIND_CSS: &str = include_str!("../../../tailwind.css");

/// One semantic token and its value in each appearance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeToken {
    /// Without the leading `--`, e.g. `background`.
    pub name: String,
    /// Raw HSL components, e.g. `0 0% 100%` — the value is wrapped by
    /// `hsl(var(--name))` at use site, which is why it has no `hsl()` here.
    pub light: String,
    pub dark: String,
}

impl ThemeToken {
    /// A CSS colour for this token in the given appearance, suitable for a
    /// swatch. Returns `None` for non-colour tokens such as `--radius`.
    pub fn swatch(&self, dark: bool) -> Option<String> {
        let value = if dark { &self.dark } else { &self.light };
        value.contains('%').then(|| format!("hsl({value})"))
    }
}

/// Pulls `--name: value;` declarations out of one `selector { … }` block.
///
/// Deliberately tolerant: it reads declaration lines and stops at the block's
/// closing brace. A format change upstream degrades this to an empty list —
/// the guide then renders no swatches — rather than panicking on a page load.
fn parse_block(css: &str, selector: &str) -> Vec<(String, String)> {
    let Some(start) = css.find(selector) else {
        return Vec::new();
    };
    let after = &css[start + selector.len()..];
    let Some(end) = after.find("\n}") else {
        return Vec::new();
    };

    after[..end]
        .lines()
        .filter_map(|line| {
            let line = line.trim().strip_prefix("--")?;
            let (name, value) = line.split_once(':')?;
            let value = value.trim().trim_end_matches(';').trim();
            (!value.is_empty()).then(|| (name.trim().to_string(), value.to_string()))
        })
        .collect()
}

/// Every token declared in the project's stylesheet, in declaration order.
///
/// Declaration order is deliberate: the CLI emits related tokens adjacently
/// (surfaces, then roles, then structural, then sidebar), so preserving it
/// groups the rendered table the way the generator intended, with no second
/// grouping table here to keep in sync.
pub fn theme_tokens() -> &'static [ThemeToken] {
    static TOKENS: OnceLock<Vec<ThemeToken>> = OnceLock::new();
    TOKENS.get_or_init(|| {
        let light = parse_block(TAILWIND_CSS, "\n:root {");
        let dark = parse_block(TAILWIND_CSS, "\n.dark {");
        light
            .into_iter()
            .map(|(name, light_value)| {
                let dark_value = dark
                    .iter()
                    .find(|(dark_name, _)| *dark_name == name)
                    .map(|(_, value)| value.clone())
                    // A token declared only in `:root` is appearance-neutral
                    // (`--radius` is the live example), so its dark value is
                    // its light value rather than a missing entry.
                    .unwrap_or_else(|| light_value.clone());
                ThemeToken {
                    name,
                    light: light_value,
                    dark: dark_value,
                }
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_block, theme_tokens};

    /// Pins the parser against the real committed stylesheet. If `adico add`
    /// ever changes the block format, this fails here rather than silently
    /// emptying the theming guide in the browser.
    #[test]
    fn the_real_stylesheet_parses_into_tokens() {
        let tokens = theme_tokens();
        assert!(
            tokens.len() >= 30,
            "expected the installed token set, got {} tokens",
            tokens.len()
        );
        for token in tokens {
            assert!(!token.name.starts_with("--"), "name kept its `--` prefix");
            assert!(!token.light.is_empty(), "{} has no light value", token.name);
            assert!(!token.dark.is_empty(), "{} has no dark value", token.name);
        }
    }

    #[test]
    fn a_known_token_has_its_expected_light_and_dark_values() {
        let tokens = theme_tokens();
        let background = tokens
            .iter()
            .find(|token| token.name == "background")
            .expect("--background is always installed");
        assert_eq!(background.light, "0 0% 100%");
        assert_ne!(
            background.dark, background.light,
            "--background must differ between appearances"
        );
    }

    /// `--radius` is declared only in `:root`; it must still appear, carrying
    /// the same value for both appearances rather than being dropped.
    #[test]
    fn an_appearance_neutral_token_is_kept() {
        let tokens = theme_tokens();
        let radius = tokens
            .iter()
            .find(|token| token.name == "radius")
            .expect("--radius is always installed");
        assert_eq!(radius.light, radius.dark);
        assert!(radius.swatch(false).is_none(), "--radius is not a colour");
    }

    #[test]
    fn colour_tokens_render_a_swatch() {
        let tokens = theme_tokens();
        let primary = tokens.iter().find(|t| t.name == "primary").unwrap();
        assert!(primary.swatch(false).unwrap().starts_with("hsl("));
        assert_ne!(primary.swatch(true), primary.swatch(false));
    }

    #[test]
    fn a_missing_block_yields_an_empty_list_rather_than_panicking() {
        assert!(parse_block("body { color: red; }", "\n:root {").is_empty());
        assert!(parse_block("", "\n:root {").is_empty());
        // Unterminated block.
        assert!(parse_block("\n:root {\n  --a: 1;", "\n:root {").is_empty());
    }

    #[test]
    fn non_declaration_lines_are_skipped() {
        let css = "\n:root {\n  /* a comment */\n  --a: 1 2% 3%;\n  color: red;\n}\n";
        let parsed = parse_block(css, "\n:root {");
        assert_eq!(parsed, vec![("a".to_string(), "1 2% 3%".to_string())]);
    }
}
