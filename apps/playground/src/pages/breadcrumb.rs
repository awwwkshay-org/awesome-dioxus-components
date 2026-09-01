use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn BreadcrumbPage() -> Element {
    rsx! {
        Demo { name: "Breadcrumb",
            components::ui::Breadcrumb {
                components::ui::BreadcrumbList {
                    components::ui::BreadcrumbItem {
                        components::ui::BreadcrumbLink { href: "/", "Home" }
                    }
                    components::ui::BreadcrumbSeparator {}
                    components::ui::BreadcrumbItem {
                        components::ui::BreadcrumbEllipsis {}
                    }
                    components::ui::BreadcrumbSeparator {}
                    components::ui::BreadcrumbItem {
                        components::ui::BreadcrumbLink { href: "/components", "Components" }
                    }
                    components::ui::BreadcrumbSeparator {}
                    components::ui::BreadcrumbItem {
                        components::ui::BreadcrumbPage { "Breadcrumb" }
                    }
                }
            }
        }
    }
}
