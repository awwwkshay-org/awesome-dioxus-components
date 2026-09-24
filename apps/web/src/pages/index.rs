use dioxus::prelude::*;

use crate::components::cta_link::{CtaLink, CtaLinkVariant};
use crate::components::prose::Prose;
use crate::components::ui::badge::Badge;
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::copy_button::CopyButton;
use crate::pages::docs::data::ui_components;

const BREW_INSTALL: &str = "brew install awwwkshay-org/tap/adico";
const CARGO_INSTALL: &str = "cargo install --git https://github.com/awwwkshay-org/awesome-dioxus-components --locked --package adico-cli";

/// Headless primitives shipped by `adico-primitives`.
///
/// Unlike the component count below, this has no compile-time manifest to read:
/// primitives are Rust modules, not registry items, so nothing enumerates them
/// the way `registry.json` enumerates components. Kept as a literal until a
/// generator exists; update it alongside `packages/adico-primitives`.
const PRIMITIVE_COUNT: usize = 67;

#[component]
pub fn Home() -> Element {
    // Read from the same embedded manifest the docs route tree uses, so the
    // headline number cannot drift from what `adico add` can actually install.
    let component_count = ui_components().count();

    rsx! {
        div { class: "mx-auto flex w-full max-w-5xl flex-col gap-20 px-6 py-16",

            // Centered column rather than left-aligned: the hero's paragraph is
            // capped well below the container width, so a left-aligned block
            // left the right half of the section visibly empty.
            section { class: "flex flex-col items-center gap-6 text-center",
                Badge { "v0.1.0" }
                h1 { class: "text-display max-w-4xl",
                    "Source-owned components for Dioxus"
                }
                Prose {
                    text: format!(
                        "adico is a shadcn-style component registry for Dioxus. `adico add` copies a \
                         component's real Rust source into your project — you read it, own it, and can \
                         change it, backed by {PRIMITIVE_COUNT} headless primitives for behavior.",
                    ),
                    class: "text-lead max-w-2xl text-muted-foreground",
                }
                InstallCommand { command: BREW_INSTALL.to_string() }
                div { class: "flex flex-wrap justify-center gap-3",
                    CtaLink { href: "/playground".to_string(), variant: CtaLinkVariant::Primary, "Browse components" }
                    CtaLink { href: "/docs".to_string(), variant: CtaLinkVariant::Outline, "Read the docs" }
                    CtaLink {
                        href: "https://github.com/awwwkshay-org/awesome-dioxus-components".to_string(),
                        variant: CtaLinkVariant::Ghost,
                        "View on GitHub"
                    }
                }
            }

            section {
                h2 { class: "text-h2", "Why adico" }
                // `items-stretch` is the grid default; the `h-full` on each
                // card's own wrapper is what actually makes a short card match
                // a tall one. Passed as a wrapper layout class, not as a
                // competing utility -- `cn()` is a plain join with no
                // last-wins conflict resolution, and `Card`'s base classes set
                // no height, so there is nothing to collide with.
                div { class: "mt-6 grid items-stretch gap-4 sm:grid-cols-2 lg:grid-cols-4",
                    FeatureCard {
                        title: format!("{component_count} components"),
                        description: "shadcn-style, source-owned UI components you install and keep.".to_string(),
                    }
                    FeatureCard {
                        title: format!("{PRIMITIVE_COUNT} primitives"),
                        description: "Headless, accessible behavior powering every styled component.".to_string(),
                    }
                    FeatureCard {
                        title: "You own the source".to_string(),
                        description: "Installed files live in your project's src/ — read, change, no black box.".to_string(),
                    }
                    FeatureCard {
                        title: "MIT OR Apache-2.0".to_string(),
                        description: "Dual-licensed, the same terms as the rest of the Rust ecosystem.".to_string(),
                    }
                }
            }

            section {
                h2 { class: "text-h2", "Install" }
                div { class: "mt-6 grid items-stretch gap-4 sm:grid-cols-3",
                    InstallChannelCard {
                        title: "Homebrew".to_string(),
                        command: BREW_INSTALL.to_string(),
                    }
                    Card { class: "flex h-full flex-col",
                        CardHeader {
                            CardTitle { "GitHub releases" }
                            CardDescription { "Prebuilt binaries for macOS, Linux, and Windows." }
                        }
                        CardContent { class: "mt-auto",
                            CtaLink {
                                href: "https://github.com/awwwkshay-org/awesome-dioxus-components/releases/latest"
                                    .to_string(),
                                variant: CtaLinkVariant::Outline,
                                "View latest release"
                            }
                        }
                    }
                    InstallChannelCard {
                        title: "From source".to_string(),
                        command: CARGO_INSTALL.to_string(),
                    }
                }
            }
        }
    }
}

#[component]
fn InstallCommand(command: String) -> Element {
    rsx! {
        Card { class: "w-full max-w-xl",
            CardContent { class: "flex items-center gap-4 p-4 text-left font-mono text-sm",
                // `min-w-0` lets the command shrink inside the flex row instead
                // of forcing the card wider; `break-words` then wraps it rather
                // than overflowing. `break-words` and not `break-all`: it takes
                // the spaces and slashes first and only splits a token when the
                // token alone cannot fit, so `brew install …/tap/adico` breaks
                // where a reader expects instead of mid-word.
                span { class: "min-w-0 flex-1 break-words", "{command}" }
                CopyButton { value: ReadSignal::new(Signal::new(command.clone())) }
            }
        }
    }
}

#[component]
fn FeatureCard(title: String, description: String) -> Element {
    rsx! {
        Card { class: "h-full",
            CardHeader {
                CardTitle { "{title}" }
                CardDescription { class: "min-w-0 text-pretty", "{description}" }
            }
        }
    }
}

#[component]
fn InstallChannelCard(title: String, command: String) -> Element {
    rsx! {
        Card { class: "flex h-full flex-col",
            CardHeader {
                CardTitle { "{title}" }
            }
            // The full command is the thing a visitor came to copy, so it wraps
            // rather than truncating -- the previous `truncate` hid the tail of
            // every command behind an ellipsis.
            CardContent { class: "mt-auto flex items-start gap-2 font-mono text-xs",
                span { class: "min-w-0 flex-1 break-words", "{command}" }
                CopyButton { value: ReadSignal::new(Signal::new(command.clone())) }
            }
        }
    }
}
