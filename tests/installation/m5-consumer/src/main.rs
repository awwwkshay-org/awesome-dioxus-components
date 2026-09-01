use dioxus::prelude::*;

fn app() -> Element {
    rsx! {
        components::ui::Alert {
            components::ui::AlertTitle { "Heads up" }
            components::ui::AlertDescription { "This is the M5 low-complexity batch fixture." }
        }
        components::ui::Breadcrumb {
            components::ui::BreadcrumbList {
                components::ui::BreadcrumbItem {
                    components::ui::BreadcrumbLink { href: "/", "Home" }
                }
                components::ui::BreadcrumbSeparator {}
                components::ui::BreadcrumbItem {
                    components::ui::BreadcrumbPage { "Fixture" }
                }
            }
        }
        components::ui::ButtonGroup {
            components::ui::Button { "Left" }
            components::ui::ButtonGroupSeparator {}
            components::ui::Button { "Right" }
        }
        components::ui::Empty {
            components::ui::EmptyHeader {
                components::ui::EmptyTitle { "No results" }
                components::ui::EmptyDescription { "Nothing to show yet." }
            }
        }
        components::ui::InputGroup {
            components::ui::InputGroupAddon { components::ui::InputGroupText { "$" } }
            components::ui::InputGroupInput { placeholder: "0.00" }
        }
        components::ui::KbdGroup {
            components::ui::Kbd { "Ctrl" }
            components::ui::Kbd { "K" }
        }
        components::ui::NativeSelect {
            components::ui::NativeSelectOption { value: "one", "One" }
            components::ui::NativeSelectOption { value: "two", "Two" }
        }
        components::ui::Spinner {}
        components::ui::Table {
            components::ui::TableBody {
                components::ui::TableRow {
                    components::ui::TableCell { "Row" }
                }
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
