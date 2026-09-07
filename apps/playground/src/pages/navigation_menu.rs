use adico_primitives::ContentAlign;
use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;
use crate::generated::controls::{
    NavigationMenuContentControls, NavigationMenuContentDemoState, NavigationMenuLinkControls,
    NavigationMenuLinkDemoState,
};

/// A marketing-site-style navigation menu: "Products" opens a two-column
/// panel with a featured tile plus described links, "Docs" opens a list of
/// title-and-description links, and "Pricing" is a plain link item.
#[component]
pub fn NavigationMenuPage() -> Element {
    let content_state = use_signal(NavigationMenuContentDemoState::default);
    let link_state = use_signal(NavigationMenuLinkDemoState::default);
    let close_on_click = move || link_state().close_on_click;
    let align = use_signal(|| ContentAlign::Start);
    rsx! {
        Demo {
            name: "Navigation Menu",
            controls: rsx! {
                NavigationMenuContentControls { state: content_state }
                NavigationMenuLinkControls { state: link_state }
                SelectControl {
                    label: "Align",
                    value: align,
                    options: &[
                        ("Start", ContentAlign::Start),
                        ("Center", ContentAlign::Center),
                        ("End", ContentAlign::End),
                    ],
                }
                p { class: "self-end pb-2 text-sm text-muted-foreground", "Hover or focus a trigger to open its content." }
            },
            components::ui::NavigationMenu {
                components::ui::NavigationMenuList {
                    components::ui::NavigationMenuItem { index: 0usize,
                        components::ui::NavigationMenuTrigger { "Products" }
                        components::ui::NavigationMenuContent {
                            force_mount: content_state().force_mount,
                            align: align(),
                            div { class: "grid w-[400px] grid-cols-2 gap-2 p-1",
                                components::ui::NavigationMenuLink {
                                    href: "#platform",
                                    close_on_click: close_on_click(),
                                    class: "row-span-3 flex h-full flex-col justify-end rounded-md bg-muted p-4",
                                    div { class: "mb-1 text-lg font-medium", "Platform" }
                                    p { class: "text-sm leading-tight text-muted-foreground",
                                        "One codebase for web, desktop, and mobile apps."
                                    }
                                }
                                components::ui::NavigationMenuLink {
                                    href: "#widgets",
                                    close_on_click: close_on_click(),
                                    div { class: "text-sm font-medium", "Widgets" }
                                    p { class: "text-xs leading-snug text-muted-foreground",
                                        "Composable building blocks for every screen."
                                    }
                                }
                                components::ui::NavigationMenuLink {
                                    href: "#gadgets",
                                    close_on_click: close_on_click(),
                                    div { class: "text-sm font-medium", "Gadgets" }
                                    p { class: "text-xs leading-snug text-muted-foreground",
                                        "Ready-made integrations and utilities."
                                    }
                                }
                                components::ui::NavigationMenuLink {
                                    href: "#themes",
                                    close_on_click: close_on_click(),
                                    div { class: "text-sm font-medium", "Themes" }
                                    p { class: "text-xs leading-snug text-muted-foreground",
                                        "Semantic tokens, dark mode, custom palettes."
                                    }
                                }
                            }
                        }
                    }
                    components::ui::NavigationMenuItem { index: 1usize,
                        components::ui::NavigationMenuTrigger { "Docs" }
                        components::ui::NavigationMenuContent {
                            force_mount: content_state().force_mount,
                            align: align(),
                            div { class: "grid w-72 gap-2 p-1",
                                components::ui::NavigationMenuLink {
                                    href: "#getting-started",
                                    close_on_click: close_on_click(),
                                    div { class: "text-sm font-medium", "Getting Started" }
                                    p { class: "text-xs leading-snug text-muted-foreground",
                                        "Install the CLI and add your first component."
                                    }
                                }
                                components::ui::NavigationMenuLink {
                                    href: "#api",
                                    close_on_click: close_on_click(),
                                    div { class: "text-sm font-medium", "API Reference" }
                                    p { class: "text-xs leading-snug text-muted-foreground",
                                        "Every prop, part, and primitive, documented."
                                    }
                                }
                                components::ui::NavigationMenuLink {
                                    href: "#accessibility",
                                    close_on_click: close_on_click(),
                                    div { class: "text-sm font-medium", "Accessibility" }
                                    p { class: "text-xs leading-snug text-muted-foreground",
                                        "Keyboard, focus, and ARIA behavior per component."
                                    }
                                }
                            }
                        }
                    }
                    components::ui::NavigationMenuItem { index: 2usize,
                        components::ui::NavigationMenuLink {
                            href: "#pricing",
                            close_on_click: close_on_click(),
                            "Pricing"
                        }
                    }
                }
            }
        }
    }
}
