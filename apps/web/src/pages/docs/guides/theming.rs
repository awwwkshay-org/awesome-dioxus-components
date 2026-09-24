//! `/docs/theming`.

use dioxus::prelude::*;

use super::super::tokens::theme_tokens;
use super::{GuidePage, GuideSection};
use crate::adico_lib::variants::{Radius, Tone};
use crate::components::code_block::CodeBlock;
use crate::components::prose::Prose;
use crate::components::ui::button::Button;
use crate::components::ui::card::Card;
use crate::components::ui::theme_builder::ThemeBuilder;

#[component]
pub fn ThemingGuide() -> Element {
    rsx! {
        GuidePage {
            title: "Theming".to_string(),
            lead: "Every colour in every component resolves through a semantic token, never a literal. Change the token and the whole system moves with it — including the components already copied into your project.".to_string(),

            GuideSection { title: "How a colour reaches a component".to_string(),
                Prose { text: "There are two layers. A raw custom property holds HSL components (`--primary: 222.2 47.4% 11.2%`), and a Tailwind token wraps it (`--color-primary: hsl(var(--primary))`). Utilities like `bg-primary` resolve through the second to the first.".to_string() }
                Prose { text: "That indirection is what makes runtime theming work: setting `--primary` on `<html>` re-colours every component instantly, with no rebuild. It is also why you should set the **raw** property and never the `--color-*` alias — writing the alias inline freezes it and it stops tracking.".to_string() }
                CodeBlock {
                    code: r#"/* the raw value — set this at runtime */
:root { --primary: 222.2 47.4% 11.2%; }

/* the Tailwind token — generated; don't set this inline */
@theme { --color-primary: hsl(var(--primary)); }"#
                        .to_string(),
                }
            }

            GuideSection { title: "Installed tokens".to_string(),
                Prose { text: "This table is read from this project's own `tailwind.css` at compile time, so it lists exactly what `adico add` installed here — not a copy that can drift. Swatches show both appearances side by side.".to_string() }
                TokenTable {}
            }

            GuideSection { title: "Tone".to_string(),
                Prose { text: "`Tone` is a *colour* axis that composes with each component's own *shape* axis. That is why there is no `Destructive` button variant: destructive is `color: Tone::Error` on whichever shape you want.".to_string() }
                div { class: "flex flex-wrap items-center gap-3",
                    Button { color: Tone::Default, "Default" }
                    Button { color: Tone::Success, "Success" }
                    Button { color: Tone::Warning, "Warning" }
                    Button { color: Tone::Error, "Error" }
                    Button { color: Tone::Info, "Info" }
                }
                CodeBlock {
                    code: "Button { variant: ButtonVariant::Outline, color: Tone::Error, \"Delete\" }"
                        .to_string(),
                }
            }

            GuideSection { title: "Radius".to_string(),
                Prose { text: "One number drives the whole corner scale. `--radius` is the base, and every step is derived from it, so changing it rescales the entire system proportionally.".to_string() }
                CodeBlock {
                    code: r#"--radius-sm:  calc(var(--radius) * 0.6);
--radius-md:  calc(var(--radius) * 0.8);
--radius-lg:  var(--radius);
--radius-xl:  calc(var(--radius) * 1.4);
--radius-2xl: calc(var(--radius) * 1.8);"#
                        .to_string(),
                }
                RadiusScale {}
            }

            GuideSection { title: "Try it".to_string(),
                Prose { text: "The editor below writes the raw custom properties onto this document, so the page around it responds as you change values. Edits are deliberately **transient** — they are cleared when you navigate away, so nothing here can strand you in a broken theme.".to_string() }
                Card { class: "p-4",
                    ThemeBuilder { on_theme_change: move |_| {} }
                }
            }
        }
    }
}

/// The installed token set, derived from the project's stylesheet.
#[component]
fn TokenTable() -> Element {
    let tokens = theme_tokens();
    rsx! {
        div { class: "flex flex-col gap-2",
            p { class: "text-sm text-muted-foreground",
                "{tokens.len()} tokens installed in this project."
            }
            div { class: "overflow-x-auto rounded-lg border border-border",
                table { class: "w-full min-w-[32rem] border-collapse text-sm",
                    thead {
                        tr { class: "border-b border-border bg-muted/40 text-left",
                            th { class: "p-3 font-medium", "Token" }
                            th { class: "p-3 font-medium", "Light" }
                            th { class: "p-3 font-medium", "Dark" }
                        }
                    }
                    tbody {
                        for token in tokens {
                            tr { key: "{token.name}", class: "border-b border-border/60 last:border-0",
                                td { class: "p-3 font-mono text-xs", "--{token.name}" }
                                td { class: "p-3",
                                    Swatch { value: token.light.clone(), color: token.swatch(false) }
                                }
                                td { class: "p-3",
                                    Swatch { value: token.dark.clone(), color: token.swatch(true) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Swatch(value: String, color: Option<String>) -> Element {
    rsx! {
        div { class: "flex min-w-0 items-center gap-2",
            if let Some(color) = color {
                span {
                    class: "size-4 shrink-0 rounded border border-border/60",
                    style: "background-color: {color}",
                }
            }
            span { class: "min-w-0 break-words font-mono text-xs text-muted-foreground", "{value}" }
        }
    }
}

/// Live specimens of the shared `Radius` scale.
#[component]
fn RadiusScale() -> Element {
    const STEPS: &[(&str, Radius)] = &[
        ("None", Radius::None),
        ("Sm", Radius::Sm),
        ("Md", Radius::Md),
        ("Default", Radius::Default),
        ("Lg", Radius::Lg),
        ("Xl", Radius::Xl),
        ("Full", Radius::Full),
    ];
    rsx! {
        div { class: "flex flex-wrap items-end gap-4",
            for (label , radius) in STEPS {
                div { key: "{label}", class: "flex flex-col items-center gap-2",
                    div { class: "size-14 border border-border bg-muted {radius.class()}" }
                    span { class: "font-mono text-xs text-muted-foreground", "{label}" }
                }
            }
        }
    }
}
