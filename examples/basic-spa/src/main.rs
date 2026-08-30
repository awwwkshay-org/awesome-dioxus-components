use adico_primitives::ContentAlign;
use adico_primitives::popover::{
    PopoverContent as PrimitivePopoverContent, PopoverTrigger as PrimitivePopoverTrigger,
};
use dioxus::prelude::*;
use palette::{IntoColor, encoding};
use time::Date;

fn main() {
    dioxus::launch(App);
}

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        main { class: "min-h-screen space-y-6 bg-background p-6 text-foreground",
            h1 { "adico basic-spa example" }

            components::ui::Button { "Source-owned Button" }
            components::ui::Dialog {
                components::ui::DialogTrigger { "Open dialog" }
                components::ui::DialogOverlay {}
                components::ui::DialogContent {
                    components::ui::DialogHeader {
                        components::ui::DialogTitle { "Installed through adico" }
                        components::ui::DialogDescription {
                            "This Dialog source belongs to this example."
                        }
                    }
                }
            }
            components::ui::Select::<String> {
                components::ui::SelectTrigger {
                    aria_label: "Choose a fruit",
                    components::ui::SelectValue { placeholder: "Choose a fruit" }
                }
                components::ui::SelectList { aria_label: "Fruit options",
                    components::ui::SelectOption::<String> {
                        index: 0usize,
                        value: "apple",
                        text_value: "Apple",
                        "Apple"
                    }
                    components::ui::SelectOption::<String> {
                        index: 1usize,
                        value: "banana",
                        text_value: "Banana",
                        "Banana"
                    }
                }
            }

            // The rest of the current migrated set, installed through the
            // real `adico` binary (no direct registry workspace imports).
            // This is a representative render, not one route per component --
            // every installed item still compiles (and is exercised at the
            // type level) whether or not it is instantiated here.
            h2 { "Theme" }
            div { class: "flex items-center gap-4",
                div { id: "mode-toggle-demo", components::ui::ModeToggle {} }
                div { id: "theme-switcher-demo", components::ui::ThemeSwitcher {} }
            }

            h2 { "Layout and content" }
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
                    SheetDemo {}
                }
            }

            components::ui::AspectRatio { ratio: 16.0 / 9.0,
                div { style: "background-color: lightblue; width: 100%; height: 100%;",
                    "16:9"
                }
            }
            div { class: "grid w-full max-w-sm gap-1.5",
                components::ui::Label { html_for: "name", "Name" }
                components::ui::Input { id: "name", placeholder: "Enter your name" }
            }
            components::ui::Progress { value: 50.0 }

            h2 { "States and selection" }
            StateRow {}
            components::ui::Accordion { allow_multiple_open: false,
                components::ui::AccordionItem { index: 0usize,
                    components::ui::AccordionTrigger { "Section one" }
                    components::ui::AccordionContent { "Section one content." }
                }
                components::ui::AccordionItem { index: 1usize,
                    components::ui::AccordionTrigger { "Section two" }
                    components::ui::AccordionContent { "Section two content." }
                }
            }
            RovingFocusRow {}

            h2 { "Overlays" }
            components::ui::Tooltip {
                components::ui::TooltipTrigger { "Hover me" }
                components::ui::TooltipContent { "Tooltip content" }
            }
            PopoverDemo {}
            components::ui::HoverCard {
                components::ui::HoverCardTrigger { "Dioxus" }
                components::ui::HoverCardContent { "Hover card content" }
            }
            components::ui::DropdownMenu {
                components::ui::DropdownMenuTrigger { "Open menu" }
                components::ui::DropdownMenuContent {
                    components::ui::DropdownMenuItem::<String> {
                        value: "edit".to_string(),
                        index: 0usize,
                        on_select: move |_value| {},
                        "Edit"
                    }
                }
            }
            components::ui::ContextMenu {
                components::ui::ContextMenuTrigger { "Right click here" }
                components::ui::ContextMenuContent {
                    components::ui::ContextMenuItem {
                        value: "edit".to_string(),
                        index: 0usize,
                        on_select: move |_value| {},
                        "Edit"
                    }
                }
            }
            components::ui::Menubar {
                components::ui::MenubarMenu { index: 0usize,
                    components::ui::MenubarTrigger { "File" }
                    components::ui::MenubarContent {
                        components::ui::MenubarItem {
                            index: 0usize,
                            value: "new".to_string(),
                            on_select: move |_value| {},
                            "New"
                        }
                    }
                }
            }
            AlertDialogDemo {}

            h2 { "Feedback" }
            components::ui::ScrollArea {
                style: "height: 6em; width: 12em; border: 1px solid black;",
                div {
                    for i in 1..=20 {
                        p { "Scrollable item {i}" }
                    }
                }
            }
            components::ui::ToastProvider {
                ToastButton {}
            }
            components::ui::Slider { label: "Volume", default_value: 50.0,
                components::ui::SliderTrack {
                    components::ui::SliderRange {}
                    components::ui::SliderThumb {}
                }
            }

            h2 { "Dates, navigation, and extras" }
            components::ui::Combobox::<String> {
                components::ui::ComboboxInput { placeholder: "Search fruit" }
                components::ui::ComboboxList {
                    components::ui::ComboboxOption::<String> { value: "Apple".to_string(), index: 0usize, "Apple" }
                    components::ui::ComboboxOption::<String> { value: "Banana".to_string(), index: 1usize, "Banana" }
                    components::ui::ComboboxEmpty { "No results" }
                }
            }
            CalendarDemo {}
            DatePickerDemo {}
            SidebarDemo {}
            components::ui::Toolbar { aria_label: "Text formatting",
                components::ui::ToolbarButton { index: 0usize, "Bold" }
                components::ui::ToolbarSeparator {}
                components::ui::ToolbarButton { index: 1usize, "Italic" }
            }
            components::ui::VirtualList {
                count: 200usize,
                estimate_size: |_idx| 32,
                style: "height: 300px; overflow-y: auto; border: 1px solid #ccc;",
                render_item: move |idx: usize| rsx! {
                    div { key: "{idx}", "Row {idx}" }
                },
            }
            DragAndDropDemo {}
            ColorPickerDemo {}
        }
    }
}

#[component]
fn SheetDemo() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
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

#[component]
fn StateRow() -> Element {
    let mut checked = use_signal(|| components::ui::CheckboxState::Unchecked);
    let mut switched = use_signal(|| false);
    let mut pressed = use_signal(|| false);
    let mut collapsible_open = use_signal(|| true);
    rsx! {
        components::ui::Avatar {
            components::ui::AvatarFallback { "AB" }
        }
        components::ui::Checkbox {
            checked: checked(),
            on_checked_change: move |value| checked.set(value),
            aria_label: "Accept terms",
        }
        components::ui::Switch {
            checked: switched(),
            on_checked_change: move |value| switched.set(value),
            aria_label: "Enable notifications",
        }
        components::ui::Toggle {
            pressed: pressed(),
            on_pressed_change: move |value| pressed.set(value),
            "Bold"
        }
        components::ui::Collapsible {
            open: collapsible_open(),
            on_open_change: move |value| collapsible_open.set(value),
            components::ui::CollapsibleTrigger { "Toggle section" }
            components::ui::CollapsibleContent { "Collapsible content" }
        }
    }
}

#[component]
fn RovingFocusRow() -> Element {
    let mut radio_value = use_signal(|| "blue".to_string());
    let mut tab_value = use_signal(|| "tab1".to_string());
    rsx! {
        components::ui::RadioGroup {
            value: Some(radio_value()),
            on_value_change: move |value| radio_value.set(value),
            components::ui::RadioItem { value: "blue".to_string(), index: 0usize, "Blue" }
            components::ui::RadioItem { value: "red".to_string(), index: 1usize, "Red" }
        }
        components::ui::Tabs {
            value: Some(tab_value()),
            on_value_change: move |value| tab_value.set(value),
            components::ui::TabList {
                components::ui::TabTrigger { value: "tab1".to_string(), index: 0usize, "Tab 1" }
                components::ui::TabTrigger { value: "tab2".to_string(), index: 1usize, "Tab 2" }
            }
            components::ui::TabContent { value: "tab1".to_string(), index: 0usize, "Tab 1 content" }
            components::ui::TabContent { value: "tab2".to_string(), index: 1usize, "Tab 2 content" }
        }
        components::ui::ToggleGroup { horizontal: true,
            components::ui::ToggleItem { index: 0usize, "Bold" }
            components::ui::ToggleItem { index: 1usize, "Italic" }
        }
    }
}

#[component]
fn PopoverDemo() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        components::ui::Popover {
            open: open(),
            on_open_change: move |value| open.set(value),
            components::ui::PopoverTrigger { "Open popover" }
            components::ui::PopoverContent { "Popover content" }
        }
    }
}

#[component]
fn AlertDialogDemo() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        components::ui::AlertDialog {
            open: open(),
            on_open_change: move |value| open.set(value),
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
fn ToastButton() -> Element {
    let toast_api = components::ui::toast::use_toast();
    rsx! {
        button {
            onclick: move |_| {
                toast_api.info("Saved".to_string(), components::ui::toast::ToastOptions::new());
            },
            "Show toast"
        }
    }
}

#[component]
fn CalendarDemo() -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let today = time::OffsetDateTime::now_utc().date();
    let mut view_date = use_signal(move || today);
    rsx! {
        components::ui::Calendar {
            selected_date: selected_date(),
            on_date_change: move |date| selected_date.set(date),
            view_date: view_date(),
            today,
            on_view_change: move |new_view: Date| view_date.set(new_view),
            components::ui::CalendarView {
                components::ui::CalendarHeader {
                    components::ui::CalendarNavigation {
                        components::ui::CalendarPreviousMonthButton { "<" }
                        components::ui::CalendarMonthTitle {}
                        components::ui::CalendarNextMonthButton { ">" }
                    }
                }
                components::ui::CalendarGrid {}
            }
        }
    }
}

#[component]
fn DatePickerDemo() -> Element {
    let mut picked_date = use_signal(|| None::<Date>);
    rsx! {
        components::ui::DatePicker {
            selected_date: picked_date(),
            on_value_change: move |date| picked_date.set(date),
            components::ui::DatePickerPopover {
                components::ui::DatePickerInput {
                    PrimitivePopoverTrigger { "Select date" }
                    PrimitivePopoverContent {
                        align: ContentAlign::End,
                        components::ui::DatePickerCalendar {
                            calendar: components::ui::Calendar,
                            components::ui::CalendarView {
                                components::ui::CalendarHeader {
                                    components::ui::CalendarNavigation {
                                        components::ui::CalendarPreviousMonthButton { "<" }
                                        components::ui::CalendarMonthTitle {}
                                        components::ui::CalendarNextMonthButton { ">" }
                                    }
                                }
                                components::ui::CalendarGrid {}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SidebarDemo() -> Element {
    rsx! {
        components::ui::SidebarProvider {
            components::ui::Sidebar {
                components::ui::SidebarHeader { "My App" }
                components::ui::SidebarContent {
                    components::ui::SidebarGroup {
                        components::ui::SidebarGroupLabel { "Section" }
                        components::ui::SidebarGroupContent {
                            components::ui::SidebarMenu {
                                components::ui::SidebarMenuItem {
                                    components::ui::SidebarMenuButton { "Overview" }
                                }
                                components::ui::SidebarMenuItem {
                                    components::ui::SidebarMenuButton { is_active: true, "Settings" }
                                }
                            }
                        }
                    }
                    components::ui::SidebarSeparator {}
                }
                components::ui::SidebarFooter { "v1.0" }
                components::ui::SidebarRail {}
            }
            components::ui::SidebarInset {
                components::ui::SidebarTrigger { "☰" }
                "Main content"
            }
        }
    }
}

#[component]
fn DragAndDropDemo() -> Element {
    let items = ["Alpha", "Bravo", "Charlie"].map(|t| rsx! { {t} }).to_vec();
    rsx! {
        components::ui::DragAndDropList { items, aria_label: "Reorderable items" }
    }
}

#[component]
fn ColorPickerDemo() -> Element {
    let mut color = use_signal(|| -> palette::Hsv<encoding::Srgb, f64> {
        components::ui::color_picker::Color::new(155, 128, 255)
            .into_format::<f64>()
            .into_color()
    });
    rsx! {
        components::ui::ColorPicker {
            color: color(),
            on_color_change: move |c| color.set(c),
            components::ui::ColorArea {
                components::ui::AreaTrack {
                    components::ui::AreaThumb {
                        components::ui::AreaThumbSaturationInput {}
                        components::ui::AreaThumbValueInput {}
                    }
                }
            }
        }
    }
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
