use dioxus::prelude::*;

fn app() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        components::ui::Card {
            components::ui::CardHeader {
                components::ui::CardTitle { "Wave 1 components" }
                components::ui::CardDescription { "Badge, Input, Item, Pagination, Skeleton, Textarea, Sheet" }
            }
            components::ui::CardContent {
                components::ui::Badge { "New" }
                components::ui::Input { placeholder: "Type here" }
                components::ui::Textarea { placeholder: "Longer text" }
                components::ui::Skeleton { class: "h-4 w-32" }
                components::ui::ItemGroup {
                    components::ui::Item {
                        components::ui::ItemContent {
                            components::ui::ItemTitle { "Row title" }
                            components::ui::ItemDescription { "Row description" }
                        }
                        components::ui::ItemActions {
                            components::ui::Badge { "Active" }
                        }
                    }
                }
                components::ui::Pagination {
                    components::ui::PaginationContent {
                        components::ui::PaginationItem {
                            components::ui::PaginationPrevious {}
                        }
                        components::ui::PaginationItem {
                            components::ui::PaginationLink { is_active: true, "1" }
                        }
                        components::ui::PaginationItem {
                            components::ui::PaginationEllipsis {}
                        }
                        components::ui::PaginationItem {
                            components::ui::PaginationNext {}
                        }
                    }
                }
            }
            components::ui::CardFooter {
                components::ui::Sheet {
                    open: open(),
                    on_open_change: move |value| open.set(value),
                    components::ui::SheetTrigger { "Open sheet" }
                    components::ui::SheetOverlay {}
                    components::ui::SheetContent {
                        components::ui::SheetHeader {
                            components::ui::SheetTitle { "Settings" }
                            components::ui::SheetDescription { "Adjust your preferences." }
                        }
                        components::ui::SheetFooter { "Done" }
                    }
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
