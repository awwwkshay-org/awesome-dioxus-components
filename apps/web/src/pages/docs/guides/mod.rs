//! Conceptual guides: the things a reader needs before the component
//! reference is useful. One page per route, per `adico-web-structure`'s
//! one-page-per-file rule.

mod dark_mode;
mod installation;
mod spacing;
mod tailwind;
mod theming;
mod typography;

pub use dark_mode::DarkModeGuide;
pub use installation::InstallationGuide;
pub use spacing::SpacingGuide;
pub use tailwind::TailwindGuide;
pub use theming::ThemingGuide;
pub use typography::TypographyGuide;

use dioxus::prelude::*;

use crate::components::prose::{Prose, ProseInline};

/// Every guide route, in reading order. One list, used by the docs sidebar
/// and by `/docs`'s own guides section, so a new guide appears in both from a
/// single edit here.
pub const GUIDES: &[(&str, &str, &str)] = &[
    (
        "/docs/installation",
        "Installation",
        "Add adico to a project and install your first component.",
    ),
    (
        "/docs/tailwind",
        "Tailwind & Dioxus",
        "How the per-project stylesheet is wired, compiled, and regenerated.",
    ),
    (
        "/docs/theming",
        "Theming",
        "Semantic tokens, palettes, radius, and a live theme editor.",
    ),
    (
        "/docs/dark-mode",
        "Light & dark mode",
        "How appearance is switched, persisted, and applied.",
    ),
    (
        "/docs/typography",
        "Typography",
        "The type scale, font tokens, and swapping the typeface.",
    ),
    (
        "/docs/spacing",
        "Spacing & radius",
        "The spacing rhythm and the shared radius scale.",
    ),
];

/// Shared chrome for a guide page: constrained column, title, and lead.
#[component]
pub fn GuidePage(title: String, lead: String, children: Element) -> Element {
    rsx! {
        article { class: "mx-auto flex w-full max-w-3xl flex-col gap-8 px-6 py-12",
            header { class: "flex flex-col gap-3",
                h1 { class: "text-h1", "{title}" }
                Prose { text: lead, class: "text-lead text-muted-foreground" }
            }
            {children}
        }
    }
}

/// One titled section of a guide.
#[component]
pub fn GuideSection(title: String, children: Element) -> Element {
    rsx! {
        section { class: "flex flex-col gap-4",
            // Inline rendering, not a raw string: guide section titles
            // legitimately name code (`dark:`, `--radius`), and a literal
            // backtick on screen is the exact defect `Prose` exists to fix.
            h2 { class: "text-h2", ProseInline { text: title } }
            {children}
        }
    }
}
