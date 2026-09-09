use adico_primitives::LocalDateExt as _;
use adico_primitives::icons::{ChevronLeft, ChevronRight, House};
use dioxus::prelude::*;
use time::Weekday;

use crate::components;

/// Shell-free harness fixture for the automated viewport test suite
/// (`tests/playwright/responsive.spec.ts` / `responsive-desktop.spec.ts`).
/// Renders in-flow (non-overlay) registry components directly, full-bleed,
/// outside `Layout` and outside `Demo`'s `overflow-visible` panning canvas —
/// see `openspec/changes/make-registry-components-mobile-first/design.md`'s
/// D7 for why the harness cannot reuse the normal demo pages or shell.
///
/// Each case gets richer, more realistic content than the corresponding demo
/// page uses (e.g. 5 real tab labels instead of the demo's 2), specifically
/// because the demo fixtures are too minimal to reproduce the overflow
/// failures this change exists to fix. This page is a harness fixture, not a
/// user-facing surface — it is not linked from the playground's navigation.
#[component]
pub fn ResponsiveFlowPage() -> Element {
    rsx! {
        div { id: "responsive-flow-root", class: "flex flex-col gap-8 p-4",
            section { "data-responsive-case": "tabs",
                components::ui::Tabs {
                    value: Some("overview".to_string()),
                    components::ui::TabList {
                        components::ui::TabTrigger { value: "overview".to_string(), index: 0usize, "Overview" }
                        components::ui::TabTrigger { value: "analytics".to_string(), index: 1usize, "Analytics" }
                        components::ui::TabTrigger { value: "notifications".to_string(), index: 2usize, "Notifications" }
                        components::ui::TabTrigger { value: "team".to_string(), index: 3usize, "Team Members" }
                        components::ui::TabTrigger { value: "billing".to_string(), index: 4usize, "Billing & Invoices" }
                    }
                    components::ui::TabContent { value: "overview".to_string(), index: 0usize, "Overview content" }
                    components::ui::TabContent { value: "analytics".to_string(), index: 1usize, "Analytics content" }
                    components::ui::TabContent { value: "notifications".to_string(), index: 2usize, "Notifications content" }
                    components::ui::TabContent { value: "team".to_string(), index: 3usize, "Team content" }
                    components::ui::TabContent { value: "billing".to_string(), index: 4usize, "Billing content" }
                }
            }
            section { "data-responsive-case": "menubar",
                components::ui::Menubar {
                    components::ui::MenubarMenu { index: 0usize,
                        components::ui::MenubarTrigger { "File" }
                        components::ui::MenubarContent {
                            components::ui::MenubarItem { index: 0usize, value: "new".to_string(), on_select: move |_| {}, "New" }
                        }
                    }
                    components::ui::MenubarMenu { index: 1usize,
                        components::ui::MenubarTrigger { "Edit" }
                        components::ui::MenubarContent {
                            components::ui::MenubarItem { index: 0usize, value: "undo".to_string(), on_select: move |_| {}, "Undo" }
                        }
                    }
                    components::ui::MenubarMenu { index: 2usize,
                        components::ui::MenubarTrigger { "View" }
                        components::ui::MenubarContent {
                            components::ui::MenubarItem { index: 0usize, value: "zoom-in".to_string(), on_select: move |_| {}, "Zoom In" }
                        }
                    }
                    components::ui::MenubarMenu { index: 3usize,
                        components::ui::MenubarTrigger { "Window" }
                        components::ui::MenubarContent {
                            components::ui::MenubarItem { index: 0usize, value: "minimize".to_string(), on_select: move |_| {}, "Minimize" }
                        }
                    }
                    components::ui::MenubarMenu { index: 4usize,
                        components::ui::MenubarTrigger { "Help" }
                        components::ui::MenubarContent {
                            components::ui::MenubarItem { index: 0usize, value: "docs".to_string(), on_select: move |_| {}, "Documentation" }
                        }
                    }
                }
            }
            section { "data-responsive-case": "breadcrumb",
                components::ui::Breadcrumb {
                    components::ui::BreadcrumbList {
                        components::ui::BreadcrumbItem {
                            components::ui::BreadcrumbLink { href: "/", "Home" }
                        }
                        components::ui::BreadcrumbSeparator {}
                        components::ui::BreadcrumbItem {
                            components::ui::BreadcrumbLink { href: "/components", "Components" }
                        }
                        components::ui::BreadcrumbSeparator {}
                        components::ui::BreadcrumbItem {
                            components::ui::BreadcrumbLink { href: "/components/registry", "Registry" }
                        }
                        components::ui::BreadcrumbSeparator {}
                        components::ui::BreadcrumbItem {
                            components::ui::BreadcrumbPage { "Data Table" }
                        }
                    }
                }
            }
            section { "data-responsive-case": "card",
                components::ui::Card { class: "max-w-md",
                    components::ui::CardHeader {
                        components::ui::CardTitle { "Sign in" }
                        components::ui::CardDescription { "Enter your email below to sign in to your account." }
                        components::ui::CardAction {
                            components::ui::Button {
                                variant: components::ui::ButtonVariant::Link,
                                class: "px-0",
                                "Sign up"
                            }
                        }
                    }
                    components::ui::CardContent {
                        div { class: "flex flex-col gap-2",
                            components::ui::Label { html_for: "responsive-card-email", "Email" }
                            components::ui::Input { id: "responsive-card-email", r#type: "email" }
                        }
                    }
                }
            }
            section { "data-responsive-case": "button-group",
                components::ui::ButtonGroup { class: "w-fit",
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Left" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Middle" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Right" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Extra" }
                    components::ui::Button { variant: components::ui::ButtonVariant::Outline, "More" }
                }
            }
            section { "data-responsive-case": "data-table", DataTableFixture {} }
            section { "data-responsive-case": "calendar", CalendarFixture {} }
            section { "data-responsive-case": "carousel",
                components::ui::Carousel {
                    orientation: components::ui::CarouselOrientation::Vertical,
                    class: "w-full max-w-xs",
                    components::ui::CarouselContent {
                        components::ui::CarouselItem {
                            div { class: "flex aspect-square items-center justify-center rounded-md bg-muted",
                                "Slide 1"
                            }
                        }
                        components::ui::CarouselItem {
                            div { class: "flex aspect-square items-center justify-center rounded-md bg-muted",
                                "Slide 2"
                            }
                        }
                    }
                }
            }
            section { "data-responsive-case": "sidebar",
                div { class: "h-[28rem] w-full overflow-hidden rounded-lg border",
                    components::ui::SidebarProvider { class: "h-full", default_open: true,
                        components::ui::Sidebar {
                            components::ui::SidebarHeader {
                                div { class: "px-2 py-1 text-sm font-semibold", "Acme Inc." }
                            }
                            components::ui::SidebarContent {
                                components::ui::SidebarGroup {
                                    components::ui::SidebarGroupLabel { "Platform" }
                                    components::ui::SidebarGroupContent {
                                        components::ui::SidebarMenu {
                                            components::ui::SidebarMenuItem {
                                                components::ui::SidebarMenuButton {
                                                    House { class: "size-4" }
                                                    "Dashboard"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A separate component so the calendar's own hook calls (`use_signal` for
/// the view date, `use_effect`) don't need threading through the page body
/// above -- the flat, always-visible composition from `pages/calendar.rs`
/// (not the popover-wrapped one), since that's the one that renders
/// `CalendarView`'s `w-[18rem]` directly in flow rather than behind a
/// trigger.
#[component]
fn CalendarFixture() -> Element {
    let today = time::OffsetDateTime::now_local_date();
    let mut view_date = use_signal(move || today);
    rsx! {
        components::ui::Calendar {
            selected_date: None,
            on_date_change: move |_| {},
            view_date: view_date(),
            today,
            on_view_change: move |new_view: time::Date| view_date.set(new_view),
            disabled: false,
            first_day_of_week: Weekday::Sunday,
            components::ui::CalendarView {
                components::ui::CalendarHeader {
                    components::ui::CalendarNavigation {
                        components::ui::CalendarPreviousMonthButton {
                            ChevronLeft { class: "size-4", size: 16 }
                        }
                        div { class: "flex flex-1 items-center gap-1",
                            components::ui::CalendarSelectMonth {
                                components::ui::CalendarSelectMonthSelect {}
                                components::ui::CalendarSelectMonthValue {}
                            }
                            components::ui::CalendarSelectYear {
                                components::ui::CalendarSelectYearSelect {}
                                components::ui::CalendarSelectYearValue {}
                            }
                        }
                        components::ui::CalendarNextMonthButton {
                            ChevronRight { class: "size-4", size: 16 }
                        }
                    }
                }
                components::ui::CalendarGrid {}
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct ResponsivePerson {
    id: String,
    name: String,
    status: String,
}

/// A separate component (not inline in the page body) so `people` and
/// `columns` can be built once per render without cluttering the page's
/// section list above -- the data table needs enough rows to actually
/// exercise its pagination footer, unlike the 3-row demo page fixture.
#[component]
fn DataTableFixture() -> Element {
    let people: Vec<ResponsivePerson> = [
        ("Ada Lovelace", "Active"),
        ("Grace Hopper", "Active"),
        ("Margaret Hamilton", "Invited"),
        ("Katherine Johnson", "Active"),
        ("Radia Perlman", "Invited"),
        ("Barbara Liskov", "Active"),
        ("Frances Allen", "Active"),
    ]
    .iter()
    .enumerate()
    .map(|(index, (name, status))| ResponsivePerson {
        id: index.to_string(),
        name: name.to_string(),
        status: status.to_string(),
    })
    .collect();
    rsx! {
        components::ui::DataTable {
            page_size: 5usize,
            loading: false,
            columns: vec![
                components::ui::DataTableColumn::new(
                        "name",
                        "Name",
                        Callback::new(|row: ResponsivePerson| rsx! { "{row.name}" }),
                    )
                    .sortable(Callback::new(|row: ResponsivePerson| row.name.clone())),
                components::ui::DataTableColumn::new(
                    "status",
                    "Status",
                    Callback::new(|row: ResponsivePerson| rsx! { "{row.status}" }),
                ),
            ],
            rows: people,
            row_id: Callback::new(|row: ResponsivePerson| row.id.clone()),
        }
    }
}
