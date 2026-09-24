//! `/docs/installation`.

use dioxus::prelude::*;

use super::{GuidePage, GuideSection};
use crate::components::code_block::CodeBlock;
use crate::components::cta_link::{CtaLink, CtaLinkVariant};
use crate::components::prose::Prose;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};

#[component]
pub fn InstallationGuide() -> Element {
    rsx! {
        GuidePage {
            title: "Installation".to_string(),
            lead: "adico installs component *source* into your project. There is no component crate to depend on — `adico add` copies real Rust files into your `src/`, and from that moment you own them.".to_string(),

            GuideSection { title: "Install the CLI".to_string(),
                Prose { text: "Homebrew, a prebuilt binary, or from source.".to_string() }
                CodeBlock { code: "brew install awwwkshay-org/tap/adico".to_string() }
                CodeBlock {
                    code: "cargo install --git https://github.com/awwwkshay-org/awesome-dioxus-components \\\n    --locked --package adico-cli"
                        .to_string(),
                }
            }

            GuideSection { title: "Initialise a project".to_string(),
                Prose { text: "Run this once, from your Dioxus project's root. It writes `components.json` — the file that records where your components, hooks, and lib helpers live — and seeds the theme tokens.".to_string() }
                CodeBlock { code: "adico init".to_string() }
            }

            GuideSection { title: "Add components".to_string(),
                Prose { text: "Each item is copied into the paths `components.json` declares, along with anything it depends on. Adding a component that needs `cn` brings `cn` with it.".to_string() }
                CodeBlock { code: "adico add button\nadico add card dialog select".to_string() }
                Prose { text: "Because the files are now yours, adico refuses to silently overwrite edits you have made: if an installed file no longer matches what the registry shipped, `adico add` stops. Pass `--replace` to take the registry's version and discard yours.".to_string() }
            }

            GuideSection { title: "Two steps the CLI does not do yet".to_string(),
                Alert { variant: AlertVariant::Warning,
                    AlertTitle { "Styling is wired by hand, once per app" }
                    AlertDescription {
                        Prose { text: "`adico init` writes the theme tokens, but it does **not** create the root `tailwind.css` or link the compiled stylesheet. Without both, every installed component renders as unstyled semantic HTML — which is the single most common reason a first install looks broken.".to_string() }
                    }
                }
                Prose { text: "The Tailwind guide walks through both steps.".to_string() }
                div { class: "flex flex-wrap gap-3",
                    CtaLink {
                        href: "/docs/tailwind".to_string(),
                        variant: CtaLinkVariant::Primary,
                        "Wire up Tailwind →"
                    }
                    CtaLink {
                        href: "/docs".to_string(),
                        variant: CtaLinkVariant::Outline,
                        "Browse components"
                    }
                }
            }

            GuideSection { title: "components.json".to_string(),
                Prose { text: "The project's own configuration. Paths are yours to change — point `ui` wherever your project keeps components and adico will install there.".to_string() }
                CodeBlock {
                    code: r#"{
  "style": "default",
  "theme": { "tokens": "shadcn", "darkMode": "class" },
  "paths": {
    "components": "src/components",
    "ui": "src/components/ui",
    "lib": "src/adico_lib",
    "hooks": "src/hooks"
  },
  "css": { "entry": "tailwind.css" }
}"#
                        .to_string(),
                }
            }
        }
    }
}
