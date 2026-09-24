use dioxus::prelude::*;

use crate::components;

/// Shell-free harness fixture rendering exactly one pre-opened overlay,
/// selected by the `case` query parameter (`/responsive/overlay?case=dialog`).
/// Overlays cannot share a single page like `ResponsiveFlowPage`'s in-flow
/// cases do -- they are all `fixed`/portalled and would pile at identical
/// viewport coordinates -- so the harness spec navigates to this route once
/// per case instead. See `responsive_flow.rs`'s module doc for the shared
/// rationale (shell-free, richer-than-demo content).
///
/// Add a new `match` arm here as each wave hardens its overlay components;
/// this file grows alongside the sweep rather than needing every case
/// authored up front.
#[component]
pub fn ResponsiveOverlayPage(case: String) -> Element {
    rsx! {
        div { "data-responsive-case": "{case}",
            match case.as_str() {
                "dialog" => rsx! { DialogCase {} },
                "alert-dialog" => rsx! { AlertDialogCase {} },
                "sheet" => rsx! { SheetCase {} },
                "drawer" => rsx! { DrawerCase {} },
                "toast" => rsx! { ToastCase {} },
                "popover" => rsx! { PopoverCase {} },
                "hover-card" => rsx! { HoverCardCase {} },
                "command" => rsx! { CommandCase {} },
                "time-picker" => rsx! { TimePickerCase {} },
                other => rsx! {
                    p { class: "p-4 text-sm text-destructive", "Unknown responsive overlay case: {other}" }
                },
            }
        }
    }
}

#[component]
fn DialogCase() -> Element {
    rsx! {
        components::ui::Dialog { open: true,
            components::ui::DialogTrigger { "Edit profile" }
            components::ui::DialogOverlay {}
            components::ui::DialogContent {
                components::ui::DialogHeader {
                    components::ui::DialogTitle { "Edit profile" }
                    components::ui::DialogDescription {
                        "Make changes to your profile here. Click save when you're done."
                    }
                }
                components::ui::DialogFooter {
                    components::ui::DialogClose { "Cancel" }
                    components::ui::Button { "Save changes" }
                }
            }
        }
    }
}

#[component]
fn AlertDialogCase() -> Element {
    rsx! {
        components::ui::AlertDialog { open: true,
            components::ui::AlertDialogTrigger { "Delete item" }
            components::ui::AlertDialogOverlay {}
            components::ui::AlertDialogContent {
                components::ui::AlertDialogHeader {
                    components::ui::AlertDialogTitle { "Delete item" }
                    components::ui::AlertDialogDescription { "Are you sure? This cannot be undone." }
                }
                components::ui::AlertDialogActions {
                    components::ui::AlertDialogCancel { "Cancel" }
                    components::ui::AlertDialogAction { "Delete" }
                }
            }
        }
    }
}

#[component]
fn SheetCase() -> Element {
    rsx! {
        components::ui::Sheet { open: true,
            components::ui::SheetTrigger { "Open sheet" }
            components::ui::SheetOverlay {}
            components::ui::SheetContent {
                components::ui::SheetHeader {
                    components::ui::SheetTitle { "Settings" }
                    components::ui::SheetDescription { "Adjust your preferences." }
                }
                components::ui::SheetFooter {
                    components::ui::SheetClose { "Close" }
                    components::ui::Button { "Save" }
                }
            }
        }
    }
}

#[component]
fn DrawerCase() -> Element {
    rsx! {
        components::ui::Drawer { open: true,
            components::ui::DrawerTrigger { "Open drawer" }
            components::ui::DrawerOverlay {}
            components::ui::DrawerContent {
                components::ui::DrawerHeader {
                    components::ui::DrawerTitle { "Move goal" }
                    components::ui::DrawerDescription { "Set your daily activity goal." }
                }
                components::ui::DrawerFooter {
                    components::ui::Button { "Submit" }
                    components::ui::DrawerClose { "Cancel" }
                }
            }
        }
    }
}

#[component]
fn ToastCase() -> Element {
    rsx! {
        components::ui::ToastProvider { ToastAutoFire {} }
    }
}

#[component]
fn ToastAutoFire() -> Element {
    let toast_api = components::ui::toast::use_toast();
    use_effect(move || {
        toast_api.info(
            "Saved".to_string(),
            components::ui::toast::ToastOptions::new(),
        );
    });
    rsx! {}
}

#[component]
fn PopoverCase() -> Element {
    rsx! {
        components::ui::Popover { open: true,
            components::ui::PopoverTrigger { "Open popover" }
            components::ui::PopoverContent { "Popover content" }
        }
    }
}

#[component]
fn HoverCardCase() -> Element {
    rsx! {
        components::ui::HoverCard { open: Signal::new(Some(true)),
            components::ui::HoverCardTrigger { class: "font-medium underline underline-offset-4",
                "@dioxus"
            }
            components::ui::HoverCardContent {
                div { class: "flex gap-4",
                    components::ui::Avatar {
                        components::ui::AvatarFallback { "DX" }
                    }
                    div { class: "space-y-1",
                        h4 { class: "text-sm font-semibold", "@dioxus" }
                        p { class: "text-sm", "Fullstack app framework for web, desktop, and mobile — written in Rust." }
                    }
                }
            }
        }
    }
}

#[component]
fn TimePickerCase() -> Element {
    rsx! {
        components::ui::TimePicker {
            selected_time: None,
            view: components::ui::TimePickerView::Analog,
            components::ui::TimePickerPopover { open: ReadSignal::from(Signal::new(Some(true))),
                components::ui::TimePickerInput {
                    components::ui::TimePickerInputValue {}
                    components::ui::TimePickerTrigger { compact: true }
                    components::ui::TimePickerContent { components::ui::TimePickerBody {} }
                }
            }
        }
    }
}

#[component]
fn CommandCase() -> Element {
    rsx! {
        components::ui::CommandDialog {
            open: true,
            title: "Command Palette".to_string(),
            description: "Search for a command to run...".to_string(),
            components::ui::CommandInput { placeholder: "Search...".to_string() }
            components::ui::CommandList {
                components::ui::CommandEmpty { "No results found." }
                components::ui::CommandGroup { heading: "Suggestions".to_string(),
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
