//! `/docs/dark-mode`.

use dioxus::prelude::*;

use super::{GuidePage, GuideSection};
use crate::components::code_block::CodeBlock;
use crate::components::prose::{Prose, ProseInline};
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
use crate::components::ui::mode_toggle::ModeToggle;
use crate::components::ui::theme_switcher::ThemeSwitcher;

#[component]
pub fn DarkModeGuide() -> Element {
    rsx! {
        GuidePage {
            title: "Light & dark mode".to_string(),
            lead: "Appearance is a class on `<html>`, not a media query. That is a deliberate choice: it lets a user pick an appearance that differs from their operating system's.".to_string(),

            GuideSection { title: "How it works".to_string(),
                Prose { text: "The generated stylesheet declares token values twice — once on `:root` for light, once on `.dark` for dark. Adding or removing the `dark` class on the document element swaps every token at once.".to_string() }
                CodeBlock {
                    code: r#":root { --background: 0 0% 100%;     --foreground: 222.2 84% 4.9%; }
    .dark { --background: 222.2 84% 4.9%; --foreground: 210 40% 98%;   }"#
                        .to_string(),
                }
                Prose { text: "`ModeToggle` writes that class, and persists the choice under `adico-theme-mode`. `System` resolves against the OS preference; `Light` and `Dark` override it.".to_string() }
                div { class: "flex flex-wrap items-center gap-3",
                    ModeToggle {}
                    ThemeSwitcher {}
                }
            }

            GuideSection { title: "The `dark:` trap".to_string(),
                Alert { variant: AlertVariant::Warning,
                    AlertTitle {
                        ProseInline { text: "Declare the dark variant, or `dark:` utilities follow the OS".to_string() }
                    }
                    AlertDescription {
                        Prose { text: "Tailwind v4's *default* `dark` variant compiles to `@media (prefers-color-scheme: dark)`. If you rely on that default, token-driven colours follow your toggle while literal `dark:` utilities follow the operating system — and for any user whose OS and chosen appearance disagree, the two contradict each other.".to_string() }
                    }
                }
                Prose { text: "One line fixes it. It must live **above** the `adico:theme:start` marker, or the next `adico add` deletes it.".to_string() }
                CodeBlock { code: "@custom-variant dark (&:is(.dark *));".to_string() }
                Prose { text: "With it declared, `dark:hover:bg-accent/50` compiles against the class instead of the media query:".to_string() }
                CodeBlock {
                    code: r#"/* without it */
.dark\:hover\:bg-accent\/50 { @media (prefers-color-scheme: dark) { ... } }

/* with it */
.dark\:hover\:bg-accent\/50:is(.dark *):hover { ... }"#
                        .to_string(),
                }
            }

            GuideSection { title: "Reading it from Rust".to_string(),
                Prose { text: "`use_persisted_theme_mode` gives you the current mode and a setter, already wired to storage. `ThemeMode::resolve` collapses `System` into a concrete appearance.".to_string() }
                CodeBlock {
                    code: r#"use adico_primitives::theme_mode::{use_persisted_theme_mode, ThemeMode};

let (mode, set_mode) = use_persisted_theme_mode();
let is_dark = mode().resolve() == ResolvedTheme::Dark;
set_mode.call(ThemeMode::Dark);"#
                        .to_string(),
                }
            }

            GuideSection { title: "Known limitation".to_string(),
                Prose { text: "The stored preference loads *after* first mount, so a reload can briefly show the default appearance before the saved one lands. Removing that flash needs a small inline script that runs before hydration and sets the class from storage — adico does not ship one yet.".to_string() }
            }
        }
    }
}
