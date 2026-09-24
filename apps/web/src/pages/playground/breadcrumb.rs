use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{
    BreadcrumbControls, BreadcrumbEllipsisControls, BreadcrumbItemControls, BreadcrumbLinkControls,
    BreadcrumbListControls, BreadcrumbPageControls, BreadcrumbSeparatorControls,
};

#[component]
pub fn BreadcrumbPage() -> Element {
    rsx! {
        Demo {
            name: "Breadcrumb",
            controls: rsx! {
                BreadcrumbControls {}
                BreadcrumbListControls {}
                BreadcrumbItemControls {}
                BreadcrumbLinkControls {}
                BreadcrumbSeparatorControls {}
                BreadcrumbEllipsisControls {}
                BreadcrumbPageControls {}
            },
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
                        components::ui::BreadcrumbLink { href: "#", "Components" }
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
