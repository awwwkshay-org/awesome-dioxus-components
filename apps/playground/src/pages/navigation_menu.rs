use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{
    NavigationMenuContentControls, NavigationMenuContentDemoState, NavigationMenuLinkControls,
    NavigationMenuLinkDemoState,
};

#[component]
pub fn NavigationMenuPage() -> Element {
    let content_state = use_signal(NavigationMenuContentDemoState::default);
    let link_state = use_signal(NavigationMenuLinkDemoState::default);
    rsx! {
        Demo {
            name: "NavigationMenu",
            controls: rsx! {
                NavigationMenuContentControls { state: content_state }
                NavigationMenuLinkControls { state: link_state }
                p { class: "self-end pb-2 text-sm text-muted-foreground", "Hover or focus a trigger to open its content." }
            },
            components::ui::NavigationMenu {
                components::ui::NavigationMenuList {
                    components::ui::NavigationMenuItem { index: 0usize,
                        components::ui::NavigationMenuTrigger { "Products" }
                        components::ui::NavigationMenuContent { force_mount: content_state().force_mount,
                            div { class: "grid w-64 gap-2",
                                components::ui::NavigationMenuLink {
                                    href: "#widgets",
                                    close_on_click: link_state().close_on_click,
                                    "Widgets"
                                }
                                components::ui::NavigationMenuLink {
                                    href: "#gadgets",
                                    close_on_click: link_state().close_on_click,
                                    "Gadgets"
                                }
                            }
                        }
                    }
                    components::ui::NavigationMenuItem { index: 1usize,
                        components::ui::NavigationMenuTrigger { "Docs" }
                        components::ui::NavigationMenuContent { force_mount: content_state().force_mount,
                            div { class: "grid w-64 gap-2",
                                components::ui::NavigationMenuLink {
                                    href: "#getting-started",
                                    close_on_click: link_state().close_on_click,
                                    "Getting Started"
                                }
                                components::ui::NavigationMenuLink {
                                    href: "#api",
                                    close_on_click: link_state().close_on_click,
                                    "API Reference"
                                }
                            }
                        }
                    }
                    components::ui::NavigationMenuItem { index: 2usize,
                        components::ui::NavigationMenuLink {
                            href: "#pricing",
                            close_on_click: link_state().close_on_click,
                            "Pricing"
                        }
                    }
                }
            }
        }
    }
}
