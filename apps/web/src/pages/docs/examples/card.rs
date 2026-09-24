//! Examples for `card`.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;

pub const SRC: &str = include_str!("card.rs");

pub const METAS: &[DocExampleMeta] = &[
    DocExampleMeta {
        id: "composition",
        title: "Full composition",
        description: "Every part the registry item exports, wired as a sign-in card.",
    },
    DocExampleMeta {
        id: "action",
        title: "Header action",
        description: "`CardAction` is the header's top-right slot. `CardHeader` switches to a two-column grid when it is present — via a *container* query, so it responds to the card's own width, not the viewport's.",
    },
];

pub fn render(id: &str) -> Element {
    match id {
        "composition" => rsx! { Composition {} },
        "action" => rsx! { Action {} },
        _ => rsx! {},
    }
}

#[component]
fn Composition() -> Element {
    rsx! {
        // doc-example:start composition
        Card { class: "w-full max-w-sm",
            CardHeader {
                CardTitle { "Sign in" }
                CardDescription { "Enter your email below to sign in to your account." }
            }
            CardContent {
                div { class: "flex flex-col gap-4",
                    div { class: "flex flex-col gap-2",
                        Label { html_for: "card-example-email", "Email" }
                        Input {
                            id: "card-example-email",
                            r#type: "email",
                            placeholder: "m@example.com",
                        }
                    }
                    div { class: "flex flex-col gap-2",
                        Label { html_for: "card-example-password", "Password" }
                        Input { id: "card-example-password", r#type: "password" }
                    }
                }
            }
            CardFooter { class: "flex-col gap-2",
                Button { class: "w-full", "Sign in" }
                Button { variant: ButtonVariant::Outline, class: "w-full", "Continue as guest" }
            }
        }
        // doc-example:end
    }
}

#[component]
fn Action() -> Element {
    rsx! {
        // doc-example:start action
        Card { class: "w-full max-w-sm",
            CardHeader {
                CardTitle { "Team plan" }
                CardDescription { "Billed annually." }
                CardAction {
                    Button { variant: ButtonVariant::Link, class: "px-0", "Upgrade" }
                }
            }
            CardContent { "Unlimited members and projects." }
        }
        // doc-example:end
    }
}
