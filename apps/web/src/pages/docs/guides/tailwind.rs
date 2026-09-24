//! `/docs/tailwind`.

use dioxus::prelude::*;

use super::{GuidePage, GuideSection};
use crate::components::code_block::CodeBlock;
use crate::components::prose::Prose;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};

#[component]
pub fn TailwindGuide() -> Element {
    rsx! {
        GuidePage {
            title: "Tailwind & Dioxus".to_string(),
            lead: "Tailwind only emits the classes it can see referenced in a project's own `src/`. That makes the stylesheet per-project by nature — adico cannot ship you a prebuilt one, so each app compiles its own.".to_string(),

            GuideSection { title: "The two files".to_string(),
                Prose { text: "A root `tailwind.css` is the compile **input**; `assets/tailwind.css` is the generated **output**. Edit the first, never the second.".to_string() }
                CodeBlock {
                    code: r#"@import "tailwindcss";
@source "./src";

/* adico:theme:start */
/* ... generated tokens, animations, scrollbar CSS ... */
/* adico:theme:end */"#
                        .to_string(),
                }
                Prose { text: "`@source \"./src\"` is what tells Tailwind where to scan for class names. If a component lives outside that path, its classes will not be emitted.".to_string() }
            }

            GuideSection { title: "Never write inside the markers".to_string(),
                Alert { variant: AlertVariant::Destructive,
                    AlertTitle { "The marker region is regenerated, not merged" }
                    AlertDescription {
                        Prose { text: "`adico add` rebuilds the file as *prefix + freshly generated region + suffix*. It keeps only the bytes **before** `adico:theme:start` and **after** `adico:theme:end`. Anything you author between them is deleted on the next install — silently, with no warning and no conflict.".to_string() }
                    }
                }
                Prose { text: "So your own CSS goes **above** the start marker. That is where fonts, font and type tokens, and custom variants belong. This site keeps exactly that there:".to_string() }
                CodeBlock {
                    code: r#"@import "tailwindcss";
@source "./src";

/* --- yours: survives every `adico add` --- */
@custom-variant dark (&:is(.dark *));

@font-face { font-family: "Geist Variable"; /* ... */ }

@theme {
  --font-sans: "Geist Variable", ui-sans-serif, system-ui, sans-serif;
  --font-mono: "Geist Mono Variable", ui-monospace, monospace;
}

/* adico:theme:start */"#
                        .to_string(),
                }
            }

            GuideSection { title: "Link the compiled stylesheet".to_string(),
                Prose { text: "Dioxus needs the `document` feature, and your root component needs the stylesheet link. `adico init` wires neither, so do both by hand once per app.".to_string() }
                CodeBlock {
                    code: r#"dioxus = { version = "0.7", features = ["document", "router", /* ... */] }"#
                        .to_string(),
                }
                CodeBlock {
                    code: r#"const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        // ... your router or root view
    }
}"#
                        .to_string(),
                }
            }

            GuideSection { title: "Compiling".to_string(),
                Prose { text: "`dx serve` and `dx build` compile the stylesheet for you, using Tailwind's cached standalone binary — there is no npm package and no Node in the loop. To compile or verify it directly:".to_string() }
                CodeBlock { code: "adico css build    # regenerate assets/tailwind.css\nadico css check    # fail if it is stale".to_string() }
                Prose { text: "`adico css check` belongs in CI: it catches a committed output that no longer matches its input.".to_string() }
            }

            GuideSection { title: "Shipping your own assets".to_string(),
                Prose { text: "`dx` only copies files reachable from an `asset!()`, and it fingerprints each one into a flat `/assets/<name>-<hash>.<ext>`. That is a problem for anything a stylesheet references by name — a hashed filename is one no `url()` can spell.".to_string() }
                Prose { text: "A **folder** asset is the exception: it is not hashed and keeps its internal structure, so relative `url()` references resolve. This is how the fonts on this site are served.".to_string() }
                CodeBlock {
                    code: r#"const FONTS: Asset = asset!("/assets/fonts", AssetOptions::folder());
// files stay at /assets/fonts/<original name>, so a stylesheet's
// url("./fonts/geist.woff2") resolves from /assets/tailwind-<hash>.css"#
                        .to_string(),
                }
            }
        }
    }
}
