use dioxus::prelude::*;

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move { Ok(dioxus::server::router(App)) });

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
fn App() -> Element {
    let mut checked = use_signal(|| components::ui::CheckboxState::Unchecked);
    let mut switched = use_signal(|| true);
    let mut open = use_signal(|| false);
    let mut tab_value = use_signal(|| "account".to_string());
    let mut dialog_open = use_signal(|| false);

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        main { class: "min-h-screen space-y-8 bg-background p-8 text-foreground",
            header { class: "flex items-center justify-between border-b border-border pb-4",
                div {
                    h1 { class: "text-2xl font-semibold", "adico theme showcase" }
                    p { class: "text-sm text-muted-foreground",
                        "Switch the mode or the palette below -- every component on this page re-renders live."
                    }
                }
                div { class: "flex items-center gap-4",
                    components::ui::ThemeSwitcher {}
                    components::ui::ModeToggle {}
                }
            }

            // SSR/hydration smoke check -- exercised by
            // tests/playwright/fullstack.spec.ts. Keep this section's
            // accessible names/structure stable.
            components::ui::Button { "SSR/hydration smoke check" }
            components::ui::Dialog {
                open: dialog_open(),
                on_open_change: move |value| dialog_open.set(value),
                components::ui::DialogTrigger { "Open dialog" }
                components::ui::DialogOverlay {}
                components::ui::DialogContent {
                    components::ui::DialogHeader {
                        components::ui::DialogTitle { "Hydration check" }
                        components::ui::DialogDescription { "Renders on the server and hydrates on the client." }
                    }
                }
            }
            components::ui::Select::<String> {
                components::ui::SelectTrigger {
                    aria_label: "Choose a fruit",
                    components::ui::SelectValue { placeholder: "Choose a fruit" }
                }
                components::ui::SelectList { aria_label: "Fruit options",
                    components::ui::SelectOption::<String> {
                        index: 0usize,
                        value: "apple",
                        text_value: "Apple",
                        "Apple"
                    }
                    components::ui::SelectOption::<String> {
                        index: 1usize,
                        value: "banana",
                        text_value: "Banana",
                        "Banana"
                    }
                }
            }

            section { class: "flex flex-wrap items-center gap-3",
                components::ui::Button { variant: components::ui::ButtonVariant::Default, "Primary" }
                components::ui::Button { variant: components::ui::ButtonVariant::Secondary, "Secondary" }
                components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Outline" }
                components::ui::Button { variant: components::ui::ButtonVariant::Ghost, "Ghost" }
                components::ui::Button { variant: components::ui::ButtonVariant::Destructive, "Destructive" }
                components::ui::Badge { "New" }
                components::ui::Badge { class: "bg-secondary text-secondary-foreground", "Secondary" }
            }

            section { class: "grid gap-6 md:grid-cols-2",
                components::ui::Card {
                    components::ui::CardHeader {
                        components::ui::CardTitle { "Notifications" }
                        components::ui::CardDescription { "Primary-colored controls track the active palette." }
                    }
                    components::ui::CardContent { class: "space-y-4",
                        div { class: "flex items-center gap-3",
                            components::ui::Checkbox {
                                checked: checked(),
                                on_checked_change: move |value| checked.set(value),
                                aria_label: "Accept terms",
                            }
                            span { "Accept terms and conditions" }
                        }
                        div { class: "flex items-center gap-3",
                            components::ui::Switch {
                                checked: switched(),
                                on_checked_change: move |value| switched.set(value),
                                aria_label: "Enable notifications",
                            }
                            span { "Enable notifications" }
                        }
                        components::ui::Progress { value: 65.0 }
                    }
                    components::ui::CardFooter { class: "gap-2",
                        components::ui::AlertDialog {
                            open: open(),
                            on_open_change: move |value| open.set(value),
                            components::ui::AlertDialogTrigger { "Reset" }
                            components::ui::AlertDialogOverlay {}
                            components::ui::AlertDialogContent {
                                components::ui::AlertDialogHeader {
                                    components::ui::AlertDialogTitle { "Reset settings?" }
                                    components::ui::AlertDialogDescription {
                                        "This uses the destructive action color, which also follows the active palette."
                                    }
                                }
                                components::ui::AlertDialogActions {
                                    components::ui::AlertDialogCancel { "Cancel" }
                                    components::ui::AlertDialogAction { "Reset" }
                                }
                            }
                        }
                    }
                }

                components::ui::Card {
                    components::ui::CardHeader {
                        components::ui::CardTitle { "Account" }
                        components::ui::CardDescription { "Tabs, avatar, and form controls in the same palette." }
                    }
                    components::ui::CardContent { class: "space-y-4",
                        div { class: "flex items-center gap-3",
                            components::ui::Avatar {
                                components::ui::AvatarFallback { "AB" }
                            }
                            div {
                                components::ui::Label { html_for: "name", "Display name" }
                                components::ui::Input { id: "name", placeholder: "Ada Byron" }
                            }
                        }
                        components::ui::Tabs {
                            value: Some(tab_value()),
                            on_value_change: move |value| tab_value.set(value),
                            components::ui::TabList {
                                components::ui::TabTrigger { value: "account".to_string(), index: 0usize, "Account" }
                                components::ui::TabTrigger { value: "security".to_string(), index: 1usize, "Security" }
                            }
                            components::ui::TabContent { value: "account".to_string(), index: 0usize,
                                "Profile details go here."
                            }
                            components::ui::TabContent { value: "security".to_string(), index: 1usize,
                                "Password and two-factor settings go here."
                            }
                        }
                    }
                }
            }
        }
    }
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
