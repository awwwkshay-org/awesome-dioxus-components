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
                            div { class: "grid w-full max-w-sm gap-1.5",
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

            h2 { class: "text-xl font-semibold", "M7 complex components" }
            section { class: "grid gap-6 md:grid-cols-2",
                CommandDemo {}
                NavigationMenuDemo {}
                DrawerDemo {}
                CarouselDemo {}
                InputOTPDemo {}
                ResizableDemo {}
            }

            h2 { class: "text-xl font-semibold", "M8 Data Table" }
            DataTableDemo {}

            h2 { class: "text-xl font-semibold", "M9 Chat/agent components" }
            ChatDemo {}
        }
    }
}

#[component]
fn DataTableDemo() -> Element {
    #[derive(Clone, PartialEq)]
    struct Person {
        id: String,
        name: String,
        status: String,
    }

    let people = vec![
        Person {
            id: "1".to_string(),
            name: "Ada Lovelace".to_string(),
            status: "active".to_string(),
        },
        Person {
            id: "2".to_string(),
            name: "Grace Hopper".to_string(),
            status: "active".to_string(),
        },
    ];

    rsx! {
        components::ui::DataTable {
            columns: vec![
                components::ui::DataTableColumn::new(
                        "name",
                        "Name",
                        Callback::new(|row: Person| rsx! { "{row.name}" }),
                    )
                    .sortable(Callback::new(|row: Person| row.name.clone())),
                components::ui::DataTableColumn::new(
                    "status",
                    "Status",
                    Callback::new(|row: Person| rsx! { "{row.status}" }),
                ),
            ],
            rows: people,
            row_id: Callback::new(|row: Person| row.id.clone()),
        }
    }
}

#[component]
fn ChatDemo() -> Element {
    let mut messages = use_signal(|| {
        vec![
            (
                "Assistant".to_string(),
                "Hi, how can I help?".to_string(),
                components::ui::MessageAlign::Start,
            ),
            (
                "You".to_string(),
                "Summarize this PDF.".to_string(),
                components::ui::MessageAlign::End,
            ),
        ]
    });
    let mut attachment_state = use_signal(|| components::ui::AttachmentState::Uploading);

    let add_message = move |_| {
        messages.write().push((
            "Assistant".to_string(),
            "On it -- one moment.".to_string(),
            components::ui::MessageAlign::Start,
        ));
    };

    rsx! {
        div { class: "flex flex-col gap-4",
            components::ui::Button { onclick: add_message, "Add message" }
            components::ui::MessageScroller { class: "rounded-md border",
                components::ui::MessageScrollerViewport { class: "h-56",
                    components::ui::MessageScrollerContent {
                        for (index , (sender , text , align)) in messages.read().iter().enumerate() {
                            components::ui::MessageScrollerItem { key: "{index}",
                                components::ui::Message {
                                    align: *align,
                                    avatar: rsx! {
                                        components::ui::MessageAvatar {
                                            components::ui::Avatar {
                                                components::ui::AvatarFallback { "{sender.chars().next().unwrap_or('?')}" }
                                            }
                                        }
                                    },
                                    components::ui::MessageHeader { "{sender}" }
                                    components::ui::MessageContent {
                                        components::ui::Bubble {
                                            align: match align {
                                                components::ui::MessageAlign::Start => components::ui::BubbleAlign::Start,
                                                components::ui::MessageAlign::End => components::ui::BubbleAlign::End,
                                            },
                                            components::ui::BubbleContent {
                                                align: match align {
                                                    components::ui::MessageAlign::Start => components::ui::BubbleAlign::Start,
                                                    components::ui::MessageAlign::End => components::ui::BubbleAlign::End,
                                                },
                                                "{text}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                components::ui::MessageScrollerButton { "Jump to latest" }
            }
            components::ui::Marker {
                components::ui::MarkerIcon { adico_primitives::icons::CircleCheck {} }
                components::ui::MarkerContent { "Step 1" }
            }
            components::ui::Attachment { state: attachment_state(),
                components::ui::AttachmentMedia {}
                components::ui::AttachmentContent {
                    components::ui::AttachmentTitle { "report.pdf" }
                    components::ui::AttachmentDescription { "2.4 MB" }
                }
                components::ui::AttachmentActions {
                    components::ui::AttachmentAction {
                        aria_label: "Mark done",
                        onclick: move |_| attachment_state.set(components::ui::AttachmentState::Done),
                        adico_primitives::icons::CircleCheck {}
                    }
                    components::ui::AttachmentAction {
                        aria_label: "Remove",
                        onclick: move |_| {},
                        adico_primitives::icons::X {}
                    }
                }
            }
        }
    }
}

#[component]
fn CommandDemo() -> Element {
    rsx! {
        components::ui::Command { class: "w-72 rounded-md border",
            components::ui::CommandInput { placeholder: "Search...".to_string() }
            components::ui::CommandList {
                components::ui::CommandEmpty { "No results." }
                components::ui::CommandGroup {
                    components::ui::CommandItem::<String> {
                        index: 0usize,
                        value: "profile".to_string(),
                        on_select: move |_| {},
                        "Profile"
                    }
                    components::ui::CommandItem::<String> {
                        index: 1usize,
                        value: "billing".to_string(),
                        on_select: move |_| {},
                        "Billing"
                    }
                }
            }
        }
    }
}

#[component]
fn NavigationMenuDemo() -> Element {
    rsx! {
        components::ui::NavigationMenu {
            components::ui::NavigationMenuList {
                components::ui::NavigationMenuItem { index: 0usize,
                    components::ui::NavigationMenuTrigger { "Products" }
                    components::ui::NavigationMenuContent {
                        components::ui::NavigationMenuLink { href: "#widgets", "Widgets" }
                    }
                }
            }
        }
    }
}

#[component]
fn DrawerDemo() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        components::ui::Drawer {
            open: open(),
            on_open_change: move |value| open.set(value),
            components::ui::DrawerTrigger { "Open drawer" }
            components::ui::DrawerOverlay {}
            components::ui::DrawerContent {
                components::ui::DrawerHeader {
                    components::ui::DrawerTitle { "Installed through adico" }
                    components::ui::DrawerDescription { "This Drawer source belongs to this example." }
                }
            }
        }
    }
}

#[component]
fn CarouselDemo() -> Element {
    rsx! {
        components::ui::Carousel { class: "w-full max-w-xs",
            components::ui::CarouselContent {
                for i in 1..=3 {
                    components::ui::CarouselItem {
                        div { class: "flex aspect-square items-center justify-center rounded-md border", "{i}" }
                    }
                }
            }
            components::ui::CarouselPrevious {}
            components::ui::CarouselNext {}
        }
    }
}

#[component]
fn InputOTPDemo() -> Element {
    let mut value = use_signal(String::new);
    rsx! {
        components::ui::InputOTP {
            length: 4usize,
            value: value(),
            on_value_change: move |v| value.set(v),
            components::ui::InputOTPGroup {
                components::ui::InputOTPSlot { index: 0usize }
                components::ui::InputOTPSlot { index: 1usize }
                components::ui::InputOTPSlot { index: 2usize }
                components::ui::InputOTPSlot { index: 3usize }
            }
        }
    }
}

#[component]
fn ResizableDemo() -> Element {
    rsx! {
        components::ui::ResizablePanelGroup { class: "h-32 rounded-md border",
            components::ui::ResizablePanel { index: 0usize, default_size: 50.0,
                div { class: "flex h-full items-center justify-center text-sm", "One" }
            }
            components::ui::ResizableHandle { handle_index: 0usize, with_handle: true }
            components::ui::ResizablePanel { index: 1usize, default_size: 50.0,
                div { class: "flex h-full items-center justify-center text-sm", "Two" }
            }
        }
    }
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
