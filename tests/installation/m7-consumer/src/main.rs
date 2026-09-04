use dioxus::prelude::*;

fn app() -> Element {
    let mut open = use_signal(|| false);
    let mut otp_value = use_signal(String::new);

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
                }
            }
        }
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
        components::ui::Drawer {
            open: open(),
            on_open_change: move |value| open.set(value),
            components::ui::DrawerTrigger { "Open drawer" }
            components::ui::DrawerOverlay {}
            components::ui::DrawerContent {
                components::ui::DrawerHeader {
                    components::ui::DrawerTitle { "This is the M7 complex-component fixture." }
                }
            }
        }
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
        components::ui::InputOTP {
            length: 4usize,
            value: otp_value(),
            on_value_change: move |v| otp_value.set(v),
            components::ui::InputOTPGroup {
                components::ui::InputOTPSlot { index: 0usize }
                components::ui::InputOTPSlot { index: 1usize }
                components::ui::InputOTPSlot { index: 2usize }
                components::ui::InputOTPSlot { index: 3usize }
            }
        }
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

fn main() {
    launch(app);
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
