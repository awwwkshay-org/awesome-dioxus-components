use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, TextControl};
use crate::components::demo::Demo;

/// A login-style card exercising every part the installed `card` item
/// exports — including `CardAction`, the header's top-right action slot.
#[component]
pub fn CardPage() -> Element {
    let show_footer = use_signal(|| true);
    let title = use_signal(|| "Sign in".to_string());
    let description =
        use_signal(|| "Enter your email below to sign in to your account.".to_string());
    rsx! {
        Demo {
            name: "Card",
            controls: rsx! {
                TextControl { label: "Title", value: title }
                TextControl { label: "Description", value: description }
                BoolControl { label: "Show actions", value: show_footer }
            },
            components::ui::Card { class: "max-w-md",
                components::ui::CardHeader {
                    components::ui::CardTitle { "{title}" }
                    components::ui::CardDescription { "{description}" }
                    components::ui::CardAction {
                        components::ui::Button {
                            variant: components::ui::ButtonVariant::Link,
                            class: "px-0",
                            "Sign up"
                        }
                    }
                }
                components::ui::CardContent {
                    div { class: "flex flex-col gap-4",
                        div { class: "flex flex-col gap-2",
                            components::ui::Label { html_for: "card-demo-email", "Email" }
                            components::ui::Input {
                                id: "card-demo-email",
                                r#type: "email",
                                placeholder: "m@example.com",
                            }
                        }
                        div { class: "flex flex-col gap-2",
                            div { class: "flex items-center justify-between",
                                components::ui::Label { html_for: "card-demo-password", "Password" }
                                span { class: "text-sm text-muted-foreground underline-offset-4 hover:underline",
                                    "Forgot your password?"
                                }
                            }
                            components::ui::Input { id: "card-demo-password", r#type: "password" }
                        }
                    }
                }
                if show_footer() {
                    components::ui::CardFooter { class: "flex-col gap-2",
                        components::ui::Button { class: "w-full", "Sign in" }
                        components::ui::Button {
                            variant: components::ui::ButtonVariant::Outline,
                            class: "w-full",
                            "Continue as guest"
                        }
                    }
                }
            }
        }
    }
}
