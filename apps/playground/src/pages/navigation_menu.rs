use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn NavigationMenuPage() -> Element {
    rsx! {
        Demo {
            name: "NavigationMenu",
            controls: rsx! {
                p { class: "self-end pb-2 text-sm text-muted-foreground", "Hover or focus a trigger to open its content." }
            },
            components::ui::NavigationMenu {
                components::ui::NavigationMenuList {
                    components::ui::NavigationMenuItem { index: 0usize,
                        components::ui::NavigationMenuTrigger { "Products" }
                        components::ui::NavigationMenuContent {
                            div { class: "grid w-64 gap-2",
                                components::ui::NavigationMenuLink { href: "#widgets", "Widgets" }
                                components::ui::NavigationMenuLink { href: "#gadgets", "Gadgets" }
                            }
                        }
                    }
                    components::ui::NavigationMenuItem { index: 1usize,
                        components::ui::NavigationMenuTrigger { "Docs" }
                        components::ui::NavigationMenuContent {
                            div { class: "grid w-64 gap-2",
                                components::ui::NavigationMenuLink { href: "#getting-started", "Getting Started" }
                                components::ui::NavigationMenuLink { href: "#api", "API Reference" }
                            }
                        }
                    }
                    components::ui::NavigationMenuItem { index: 2usize,
                        components::ui::NavigationMenuLink { href: "#pricing", "Pricing" }
                    }
                }
            }
        }
    }
}
