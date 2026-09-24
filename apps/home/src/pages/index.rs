use dioxus::prelude::*;

use crate::components::cta_link::{CtaLink, CtaLinkVariant};
use crate::components::ui::badge::Badge;
use crate::components::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::components::ui::copy_button::CopyButton;

const BREW_INSTALL: &str = "brew install awwwkshay-org/tap/adico";
const CARGO_INSTALL: &str = "cargo install --git https://github.com/awwwkshay-org/awesome-dioxus-components --locked --package adico-cli";

#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "mx-auto flex w-full max-w-5xl flex-col gap-20 px-6 py-16",

            section { class: "flex flex-col items-start gap-6",
                Badge { "v0.1.0" }
                h1 { class: "text-4xl font-bold tracking-tight sm:text-5xl",
                    "Source-owned components for Dioxus"
                }
                p { class: "max-w-2xl text-lg text-muted-foreground",
                    "adico is a shadcn-style component registry for Dioxus. `adico add` copies a "
                    "component's real Rust source into your project — you read it, own it, and can "
                    "change it, backed by 67 headless primitives for behavior."
                }
                InstallCommand { command: BREW_INSTALL.to_string() }
                div { class: "flex flex-wrap gap-3",
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
                h2 { class: "text-2xl font-semibold", "Why adico" }
                div { class: "mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4",
                    FeatureCard {
                        title: "69 components".to_string(),
                        description: "shadcn-style, source-owned UI components you install and keep.".to_string(),
                    }
                    FeatureCard {
                        title: "67 primitives".to_string(),
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
                h2 { class: "text-2xl font-semibold", "Install" }
                div { class: "mt-6 grid gap-4 sm:grid-cols-3",
                    InstallChannelCard {
                        title: "Homebrew".to_string(),
                        command: BREW_INSTALL.to_string(),
                    }
                    Card {
                        CardHeader {
                            CardTitle { "GitHub releases" }
                            CardDescription { "Prebuilt binaries for macOS, Linux, and Windows." }
                        }
                        CardContent {
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
            CardContent { class: "flex items-center justify-between gap-4 font-mono text-sm",
                span { "{command}" }
                CopyButton { value: ReadSignal::new(Signal::new(command.clone())) }
            }
        }
    }
}

#[component]
fn FeatureCard(title: String, description: String) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "{title}" }
                CardDescription { "{description}" }
            }
        }
    }
}

#[component]
fn InstallChannelCard(title: String, command: String) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "{title}" }
            }
            CardContent { class: "flex items-center justify-between gap-2 font-mono text-xs",
                span { class: "truncate", "{command}" }
                CopyButton { value: ReadSignal::new(Signal::new(command.clone())) }
            }
        }
    }
}
