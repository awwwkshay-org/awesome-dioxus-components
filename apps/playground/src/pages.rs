//! One page per currently migrated registry item. Each renders the same
//! working demo composition already verified against the real `adico` CLI
//! install, just split into its own route instead of one long scroll.

use adico_primitives::ContentAlign;
use adico_primitives::icons::{ChevronLeft, ChevronRight};
use dioxus::prelude::*;
use time::{Date, Weekday};

use crate::components;
use crate::controls::{BoolControl, SelectControl, TextControl};
use crate::demo::Demo;

#[derive(Clone, Copy, PartialEq)]
enum ButtonContent {
    Text,
    Icon,
    IconAndText,
}

#[component]
pub fn ButtonPage() -> Element {
    let disabled = use_signal(|| false);
    let mut variant = use_signal(|| components::ui::ButtonVariant::Default);
    let mut size = use_signal(|| components::ui::ButtonSize::Default);
    let mut button_type = use_signal(|| "button".to_string());
    let mut content = use_signal(|| ButtonContent::Text);
    let label = use_signal(|| "Save changes".to_string());
    rsx! {
        Demo {
            name: "Button",
            controls: rsx! {
                SelectControl {
                    label: "Variant",
                    value: variant(),
                    options: vec![
                        ("Default", components::ui::ButtonVariant::Default),
                        ("Destructive", components::ui::ButtonVariant::Destructive),
                        ("Outline", components::ui::ButtonVariant::Outline),
                        ("Secondary", components::ui::ButtonVariant::Secondary),
                        ("Ghost", components::ui::ButtonVariant::Ghost),
                        ("Link", components::ui::ButtonVariant::Link),
                    ],
                    on_change: move |value| variant.set(value),
                }
                SelectControl {
                    label: "Size",
                    value: size(),
                    options: vec![
                        ("Default", components::ui::ButtonSize::Default),
                        ("Extra small", components::ui::ButtonSize::Xs),
                        ("Small", components::ui::ButtonSize::Sm),
                        ("Large", components::ui::ButtonSize::Lg),
                        ("Icon", components::ui::ButtonSize::Icon),
                        ("Icon extra small", components::ui::ButtonSize::IconXs),
                        ("Icon small", components::ui::ButtonSize::IconSm),
                        ("Icon large", components::ui::ButtonSize::IconLg),
                    ],
                    on_change: move |value| size.set(value),
                }
                BoolControl { label: "Disabled", value: disabled }
                SelectControl {
                    label: "Native type",
                    value: button_type(),
                    options: vec![
                        ("Button", "button".to_string()),
                        ("Submit", "submit".to_string()),
                        ("Reset", "reset".to_string()),
                    ],
                    on_change: move |value| button_type.set(value),
                }
                SelectControl {
                    label: "Children",
                    value: content(),
                    options: vec![
                        ("Text", ButtonContent::Text),
                        ("Icon only", ButtonContent::Icon),
                        ("Icon and text", ButtonContent::IconAndText),
                    ],
                    on_change: move |value| content.set(value),
                }
                TextControl { label: "Text", value: label }
            },
            components::ui::Button {
                variant: variant(),
                size: size(),
                disabled: disabled(),
                r#type: button_type(),
                aria_label: (content() == ButtonContent::Icon).then_some("Save changes"),
                if content() != ButtonContent::Text {
                    span { "aria-hidden": "true", "↗" }
                }
                if content() != ButtonContent::Icon {
                    "{label}"
                }
            }
        }
    }
}

#[component]
pub fn BadgePage() -> Element {
    let mut variant = use_signal(|| components::ui::BadgeVariant::Default);
    let label = use_signal(|| "New".to_string());
    rsx! {
        Demo {
            name: "Badge",
            controls: rsx! {
                SelectControl {
                    label: "Variant",
                    value: variant(),
                    options: vec![
                        ("Default", components::ui::BadgeVariant::Default),
                        ("Secondary", components::ui::BadgeVariant::Secondary),
                        ("Destructive", components::ui::BadgeVariant::Destructive),
                        ("Outline", components::ui::BadgeVariant::Outline),
                        ("Verified", components::ui::BadgeVariant::Verified),
                    ],
                    on_change: move |value| variant.set(value),
                }
                TextControl { label: "Content", value: label }
            },
            components::ui::Badge { variant: variant(), "{label}" }
        }
    }
}

#[component]
pub fn CardPage() -> Element {
    let show_footer = use_signal(|| true);
    let title = use_signal(|| "Card title".to_string());
    let description = use_signal(|| "Supporting description text.".to_string());
    rsx! {
        Demo {
            name: "Card",
            controls: rsx! {
                TextControl { label: "Title", value: title }
                TextControl { label: "Description", value: description }
                BoolControl { label: "Show actions", value: show_footer }
            },
            components::ui::Card { class: "max-w-md",
                components::ui::CardHeader {
                    components::ui::CardTitle { "{title}" }
                    components::ui::CardDescription { "{description}" }
                }
                components::ui::CardContent { "Card body content uses composed semantic regions." }
                if show_footer() {
                    components::ui::CardFooter {
                        components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Cancel" }
                        components::ui::Button { "Continue" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn InputPage() -> Element {
    let placeholder = use_signal(|| "Type here".to_string());
    let disabled = use_signal(|| false);
    let readonly = use_signal(|| false);
    let required = use_signal(|| false);
    let invalid = use_signal(|| false);
    rsx! {
        Demo {
            name: "Input",
            controls: rsx! {
                TextControl { label: "Placeholder", value: placeholder }
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Read only", value: readonly }
                BoolControl { label: "Required", value: required }
                BoolControl { label: "Invalid", value: invalid }
            },
            components::ui::Input {
                placeholder: placeholder(),
                disabled: disabled(),
                readonly: readonly(),
                required: required(),
                invalid: invalid(),
            }
        }
    }
}

#[component]
pub fn TextareaPage() -> Element {
    let placeholder = use_signal(|| "Longer text".to_string());
    let disabled = use_signal(|| false);
    let readonly = use_signal(|| false);
    let required = use_signal(|| false);
    let invalid = use_signal(|| false);
    rsx! {
        Demo {
            name: "Textarea",
            controls: rsx! {
                TextControl { label: "Placeholder", value: placeholder }
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Read only", value: readonly }
                BoolControl { label: "Required", value: required }
                BoolControl { label: "Invalid", value: invalid }
            },
            components::ui::Textarea {
                placeholder: placeholder(),
                disabled: disabled(),
                readonly: readonly(),
                required: required(),
                invalid: invalid(),
            }
        }
    }
}

#[component]
pub fn SkeletonPage() -> Element {
    let mut variant = use_signal(|| components::ui::SkeletonVariant::Default);
    let decorative = use_signal(|| true);
    rsx! {
        Demo {
            name: "Skeleton",
            controls: rsx! {
                SelectControl {
                    label: "Shape",
                    value: variant(),
                    options: vec![
                        ("Rectangle", components::ui::SkeletonVariant::Default),
                        ("Circle", components::ui::SkeletonVariant::Circle),
                    ],
                    on_change: move |value| variant.set(value),
                }
                BoolControl { label: "Decorative", value: decorative }
            },
            components::ui::Skeleton {
                variant: variant(),
                decorative: decorative(),
                class: if variant() == components::ui::SkeletonVariant::Circle { "size-16" } else { "h-4 w-40" },
            }
        }
    }
}

#[component]
pub fn ItemPage() -> Element {
    let mut variant = use_signal(|| components::ui::ItemVariant::Default);
    let disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "Item",
            controls: rsx! {
                SelectControl {
                    label: "Variant",
                    value: variant(),
                    options: vec![
                        ("Default", components::ui::ItemVariant::Default),
                        ("Muted", components::ui::ItemVariant::Muted),
                        ("Interactive", components::ui::ItemVariant::Interactive),
                    ],
                    on_change: move |value| variant.set(value),
                }
                BoolControl { label: "Disabled", value: disabled }
            },
            components::ui::ItemGroup {
                components::ui::Item { variant: variant(), disabled: disabled(), class: "w-full max-w-md",
                    components::ui::ItemContent {
                        components::ui::ItemTitle { "Row title" }
                        components::ui::ItemDescription { "Row description" }
                    }
                    components::ui::ItemActions {
                        components::ui::Badge { "Active" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PaginationPage() -> Element {
    let mut active_page = use_signal(|| 2usize);
    let compact = use_signal(|| false);
    let previous_text = use_signal(|| "Previous".to_string());
    let next_text = use_signal(|| "Next".to_string());
    rsx! {
        Demo {
            name: "Pagination",
            controls: rsx! {
                SelectControl {
                    label: "Active page",
                    value: active_page(),
                    options: vec![("Page 1", 1usize), ("Page 2", 2usize), ("Page 3", 3usize)],
                    on_change: move |value| active_page.set(value),
                }
                BoolControl { label: "Compact previous / next", value: compact }
                TextControl { label: "Previous text", value: previous_text }
                TextControl { label: "Next text", value: next_text }
            },
            components::ui::Pagination {
                components::ui::PaginationContent {
                    components::ui::PaginationItem {
                        components::ui::PaginationPrevious {
                            text: previous_text(),
                            compact: compact(),
                            onclick: move |_| active_page.set(active_page().saturating_sub(1).max(1)),
                        }
                    }
                    components::ui::PaginationItem {
                        components::ui::PaginationLink { is_active: active_page() == 1, onclick: move |_| active_page.set(1), "1" }
                    }
                    components::ui::PaginationItem {
                        components::ui::PaginationLink { is_active: active_page() == 2, onclick: move |_| active_page.set(2), "2" }
                    }
                    components::ui::PaginationItem {
                        components::ui::PaginationLink { is_active: active_page() == 3, onclick: move |_| active_page.set(3), "3" }
                    }
                    components::ui::PaginationItem { components::ui::PaginationEllipsis {} }
                    components::ui::PaginationItem {
                        components::ui::PaginationNext {
                            text: next_text(),
                            compact: compact(),
                            onclick: move |_| active_page.set((active_page() + 1).min(3)),
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn DialogPage() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        Demo {
            name: "Dialog",
            controls: rsx! {
                BoolControl { label: "Open", value: open }
            },
            components::ui::Dialog {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::DialogTrigger { "Open dialog" }
                components::ui::DialogOverlay {}
                components::ui::DialogContent {
                    components::ui::DialogHeader {
                        components::ui::DialogTitle { "Installed through adico" }
                        components::ui::DialogDescription { "This Dialog source belongs to this app." }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SheetPage() -> Element {
    let mut side = use_signal(|| components::ui::SheetSide::Right);
    rsx! {
        Demo {
            name: "Sheet",
            controls: rsx! {
                SelectControl {
                    label: "Side",
                    value: side(),
                    options: vec![
                        ("Right", components::ui::SheetSide::Right),
                        ("Left", components::ui::SheetSide::Left),
                        ("Top", components::ui::SheetSide::Top),
                        ("Bottom", components::ui::SheetSide::Bottom),
                    ],
                    on_change: move |value| side.set(value),
                }
            },
            components::ui::Sheet {
                components::ui::SheetTrigger { "Open sheet" }
                components::ui::SheetOverlay {}
                components::ui::SheetContent { side: side(),
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

#[component]
pub fn SelectPage() -> Element {
    let disabled = use_signal(|| false);
    let multiple = use_signal(|| false);
    let mut value = use_signal(|| None::<String>);
    let mut values = use_signal(|| Some(Vec::<String>::new()));
    let mut open = use_signal(|| None::<bool>);
    let invalid = use_signal(|| false);
    rsx! {
        Demo {
            name: "Select",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Multi-select", value: multiple }
                BoolControl { label: "Invalid presentation", value: invalid }
                if !multiple() {
                    SelectControl {
                        label: "Value",
                        value: value(),
                        options: vec![
                            ("None", None),
                            ("Apple", Some("apple".to_string())),
                            ("Banana", Some("banana".to_string())),
                        ],
                        on_change: move |next| value.set(next),
                    }
                } else {
                    p { class: "self-end pb-2 text-sm text-muted-foreground", "Choose one or more options in the preview." }
                }
                SelectControl {
                    label: "Open state",
                    value: open(),
                    options: vec![("Uncontrolled", None), ("Closed", Some(false)), ("Open", Some(true))],
                    on_change: move |next| open.set(next),
                }
            },
            if multiple() {
                components::ui::SelectMulti::<String> {
                    disabled: disabled(),
                    values: ReadSignal::from(values),
                    open: open,
                    on_values_change: move |next| values.set(Some(next)),
                    components::ui::SelectTrigger {
                        class: "w-48",
                        aria_label: "Choose one or more fruits",
                        aria_invalid: invalid(),
                        components::ui::SelectValue { placeholder: "Choose fruits" }
                    }
                    components::ui::SelectList { class: "w-48", aria_label: "Fruit options",
                        components::ui::SelectOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                        components::ui::SelectOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana" }
                    }
                }
            } else {
                components::ui::Select::<String> {
                    disabled: disabled(),
                    value: Some(ReadSignal::from(value)),
                    open: open,
                    on_value_change: move |next| value.set(next),
                    components::ui::SelectTrigger {
                        class: "w-48",
                        aria_label: "Choose a fruit",
                        aria_invalid: invalid(),
                        components::ui::SelectValue { placeholder: "Choose a fruit" }
                    }
                    components::ui::SelectList { class: "w-48", aria_label: "Fruit options",
                        components::ui::SelectOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                        components::ui::SelectOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ComboboxPage() -> Element {
    let disabled = use_signal(|| false);
    let multiple = use_signal(|| false);
    let mut value = use_signal(|| None::<String>);
    let mut values = use_signal(|| Some(Vec::<String>::new()));
    let mut open = use_signal(|| None::<bool>);
    rsx! {
        Demo {
            name: "Combobox",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Multi-select", value: multiple }
                if !multiple() {
                    SelectControl {
                        label: "Value",
                        value: value(),
                        options: vec![("None", None), ("Apple", Some("Apple".to_string())), ("Banana", Some("Banana".to_string()))],
                        on_change: move |next| value.set(next),
                    }
                } else {
                    p { class: "self-end pb-2 text-sm text-muted-foreground", "Choose one or more options in the preview." }
                }
                SelectControl {
                    label: "Open state",
                    value: open(),
                    options: vec![("Uncontrolled", None), ("Closed", Some(false)), ("Open", Some(true))],
                    on_change: move |next| open.set(next),
                }
            },
            if multiple() {
                components::ui::ComboboxMulti::<String> {
                    disabled: disabled(),
                    values: ReadSignal::from(values),
                    open: open,
                    on_values_change: move |next| values.set(Some(next)),
                    components::ui::ComboboxInput { class: "w-48", placeholder: "Search fruits" }
                    components::ui::ComboboxList { class: "w-48",
                        components::ui::ComboboxOption::<String> { value: "Apple".to_string(), index: 0usize, "Apple" }
                        components::ui::ComboboxOption::<String> { value: "Banana".to_string(), index: 1usize, "Banana" }
                        components::ui::ComboboxEmpty { "No results" }
                    }
                }
            } else {
                components::ui::Combobox::<String> {
                    disabled: disabled(),
                    value: Some(ReadSignal::from(value)),
                    open: open,
                    on_value_change: move |next| value.set(next),
                    components::ui::ComboboxInput { class: "w-48", placeholder: "Search fruit" }
                    components::ui::ComboboxList { class: "w-48",
                        components::ui::ComboboxOption::<String> { value: "Apple".to_string(), index: 0usize, "Apple" }
                        components::ui::ComboboxOption::<String> { value: "Banana".to_string(), index: 1usize, "Banana" }
                        components::ui::ComboboxEmpty { "No results" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn TooltipPage() -> Element {
    let mut open = use_signal(|| None::<bool>);
    let disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "Tooltip",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                SelectControl {
                    label: "Open state",
                    value: open(),
                    options: vec![("Uncontrolled", None), ("Closed", Some(false)), ("Open", Some(true))],
                    on_change: move |value| open.set(value),
                }
            },
            components::ui::Tooltip { open: open, disabled: disabled(),
                components::ui::TooltipTrigger { "Hover me" }
                components::ui::TooltipContent { "Tooltip content" }
            }
        }
    }
}

#[component]
pub fn PopoverPage() -> Element {
    let mut open = use_signal(|| false);
    let mut align = use_signal(|| ContentAlign::Center);
    rsx! {
        Demo {
            name: "Popover",
            controls: rsx! {
                BoolControl { label: "Open", value: open }
                SelectControl {
                    label: "Align",
                    value: align(),
                    options: vec![
                        ("Start", ContentAlign::Start),
                        ("Center", ContentAlign::Center),
                        ("End", ContentAlign::End),
                    ],
                    on_change: move |value| align.set(value),
                }
            },
            components::ui::Popover {
                open: open(),
                on_open_change: move |value| open.set(value),
                components::ui::PopoverTrigger { "Open popover" }
                components::ui::PopoverContent { align: align(), "Popover content" }
            }
        }
    }
}

#[component]
pub fn HoverCardPage() -> Element {
    let mut open = use_signal(|| None::<bool>);
    let disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "HoverCard",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                SelectControl {
                    label: "Open state",
                    value: open(),
                    options: vec![("Uncontrolled", None), ("Closed", Some(false)), ("Open", Some(true))],
                    on_change: move |value| open.set(value),
                }
            },
            components::ui::HoverCard { open: open, disabled: disabled(),
                components::ui::HoverCardTrigger { "Dioxus" }
                components::ui::HoverCardContent { "Hover card content" }
            }
        }
    }
}

#[component]
pub fn DropdownMenuPage() -> Element {
    let disabled = use_signal(|| false);
    let mut open = use_signal(|| None::<bool>);
    rsx! {
        Demo {
            name: "DropdownMenu",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                SelectControl {
                    label: "Open state", value: open(),
                    options: vec![("Uncontrolled", None), ("Closed", Some(false)), ("Open", Some(true))],
                    on_change: move |value| open.set(value),
                }
            },
            components::ui::DropdownMenu { disabled: disabled(), open: open,
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
        }
    }
}

#[component]
pub fn ContextMenuPage() -> Element {
    let disabled = use_signal(|| false);
    let mut open = use_signal(|| None::<bool>);
    rsx! {
        Demo {
            name: "ContextMenu",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                SelectControl {
                    label: "Open state", value: open(),
                    options: vec![("Uncontrolled", None), ("Closed", Some(false)), ("Open", Some(true))],
                    on_change: move |value| open.set(value),
                }
            },
            components::ui::ContextMenu { disabled: disabled(), open: open,
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
        }
    }
}

#[component]
pub fn MenubarPage() -> Element {
    let disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "Menubar",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
            },
            components::ui::Menubar { disabled: disabled(),
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
        }
    }
}

#[component]
pub fn CalendarPage() -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let today = time::OffsetDateTime::now_utc().date();
    let mut view_date = use_signal(move || today);
    let disabled = use_signal(|| false);
    let mut first_day_of_week = use_signal(|| Weekday::Sunday);
    // Jump the visible month to a pre-existing selected date (e.g. on first
    // open) rather than always starting on today's month; mirrors the same
    // effect `DatePickerCalendar` already runs internally.
    use_effect(move || {
        if let Some(date) = selected_date() {
            view_date.set(date);
        }
    });
    rsx! {
        Demo {
            name: "Calendar",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                SelectControl {
                    label: "First day of week",
                    value: first_day_of_week(),
                    options: vec![("Sunday", Weekday::Sunday), ("Monday", Weekday::Monday)],
                    on_change: move |next| first_day_of_week.set(next),
                }
            },
            div { class: "flex w-full justify-center",
                components::ui::Calendar {
                    selected_date: selected_date(),
                    on_date_change: move |date| selected_date.set(date),
                    view_date: view_date(),
                    today,
                    on_view_change: move |new_view: Date| view_date.set(new_view),
                    disabled: disabled(),
                    first_day_of_week: first_day_of_week(),
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
    }
}

#[component]
pub fn DatePickerPage() -> Element {
    let mut picked_date = use_signal(|| None::<Date>);
    let disabled = use_signal(|| false);
    let read_only = use_signal(|| false);
    let mut open = use_signal(|| false);
    rsx! {
        Demo {
            name: "DatePicker",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Read only", value: read_only }
            },
            components::ui::DatePicker {
                selected_date: picked_date(),
                on_value_change: move |date| picked_date.set(date),
                disabled: disabled(),
                read_only: read_only(),
                components::ui::DatePickerPopover {
                    class: "playground-date-picker-popover-root",
                    open: Some(open()),
                    on_open_change: move |value| open.set(value),
                    components::ui::DatePickerInput {
                        components::ui::DatePickerInputValue {}
                        components::ui::DatePickerTrigger {}
                        components::ui::DatePickerContent {
                            components::ui::DatePickerCalendar {
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
                }
            }
        }
    }
}

#[component]
pub fn SidebarPage() -> Element {
    let mut collapsible = use_signal(|| components::ui::SidebarCollapsible::Offcanvas);
    let mut side = use_signal(|| components::ui::SidebarSide::Left);
    let mut open = use_signal(|| Some(true));
    let active_settings = use_signal(|| true);
    let settings_disabled = use_signal(|| false);
    rsx! {
        Demo {
            name: "Sidebar",
            controls: rsx! {
                SelectControl {
                    label: "Collapsible",
                    value: collapsible(),
                    options: vec![
                        ("Offcanvas", components::ui::SidebarCollapsible::Offcanvas),
                        ("Icon", components::ui::SidebarCollapsible::Icon),
                        ("None", components::ui::SidebarCollapsible::None),
                    ],
                    on_change: move |value| collapsible.set(value),
                }
                SelectControl {
                    label: "Side",
                    value: side(),
                    options: vec![
                        ("Left", components::ui::SidebarSide::Left),
                        ("Right", components::ui::SidebarSide::Right),
                    ],
                    on_change: move |value| side.set(value),
                }
                SelectControl {
                    label: "Open state",
                    value: open(),
                    options: vec![("Uncontrolled", None), ("Open", Some(true)), ("Closed", Some(false))],
                    on_change: move |value| open.set(value),
                }
                BoolControl { label: "Settings active", value: active_settings }
                BoolControl { label: "Settings disabled", value: settings_disabled }
            },
            components::ui::SidebarProvider { class: "h-64 min-h-0 rounded-lg border",
                open: open,
                components::ui::Sidebar {
                    collapsible: collapsible(),
                    side: side(),
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
                                        components::ui::SidebarMenuButton { is_active: active_settings(), disabled: settings_disabled(), "Settings" }
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
                    " Main content"
                }
            }
        }
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn calendar_page_builds_its_primitive_tree() {
        let mut dom = VirtualDom::new(CalendarPage);
        dom.rebuild_in_place();
        let html = dioxus::ssr::render(&dom);
        assert!(html.contains("Calendar"));
        assert!(html.contains("role=\"grid\""));
    }

    #[test]
    fn date_picker_uses_segment_width_placeholders_once() {
        let mut dom = VirtualDom::new(DatePickerPage);
        dom.rebuild_in_place();
        let html = dioxus::ssr::render(&dom);

        assert!(html.contains("YYYY"));
        assert!(html.contains("MM"));
        assert!(html.contains("DD"));
        assert!(!html.contains("YYYYY"));
        assert!(!html.contains("MMM"));
        assert!(!html.contains("DDD"));
    }
}
