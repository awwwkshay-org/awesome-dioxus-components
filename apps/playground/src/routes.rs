//! The router: route declarations, the sidebar navigation list, and the
//! routing shell. dioxus-router has no file-system route generation, so
//! this enum still declares every path explicitly even though each page's
//! body now lives under `pages/` (see `pages/mod.rs`).

use dioxus::prelude::*;

use crate::components;
use crate::components::theme_builder_launcher::ThemeBuilderLauncher;
use crate::pages::{
    AccordionPage, AlertDialogPage, AlertPage, AspectRatioPage, AvatarPage, BadgePage,
    BreadcrumbPage, ButtonGroupPage, ButtonPage, CalendarPage, CardPage, CheckboxPage,
    CollapsiblePage, ColorPickerPage, ComboboxPage, ContextMenuPage, DatePickerPage, DialogPage,
    DragAndDropListPage, DropdownMenuPage, EmptyPage, Home, HoverCardPage, InputGroupPage,
    InputPage, ItemPage, KbdPage, LabelPage, MenubarPage, ModeTogglePage, NativeSelectPage,
    PaginationPage, PopoverPage, ProgressPage, RadioGroupPage, ScrollAreaPage, SelectPage,
    SheetPage, SidebarPage, SkeletonPage, SliderPage, SpinnerPage, SwitchPage, TablePage, TabsPage,
    TagGroupPage, TextareaPage, ThemeSwitcherPage, ToastPage, ToggleGroupPage, TogglePage,
    ToolbarPage, TooltipPage, VirtualListPage,
};

const PLAYGROUND_LOGO: Asset = asset!("/assets/web/android-chrome-192x192.png");

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/button")]
    ButtonPage {},
    #[route("/badge")]
    BadgePage {},
    #[route("/card")]
    CardPage {},
    #[route("/input")]
    InputPage {},
    #[route("/textarea")]
    TextareaPage {},
    #[route("/skeleton")]
    SkeletonPage {},
    #[route("/item")]
    ItemPage {},
    #[route("/pagination")]
    PaginationPage {},
    #[route("/dialog")]
    DialogPage {},
    #[route("/sheet")]
    SheetPage {},
    #[route("/select")]
    SelectPage {},
    #[route("/combobox")]
    ComboboxPage {},
    #[route("/tooltip")]
    TooltipPage {},
    #[route("/popover")]
    PopoverPage {},
    #[route("/hover-card")]
    HoverCardPage {},
    #[route("/dropdown-menu")]
    DropdownMenuPage {},
    #[route("/context-menu")]
    ContextMenuPage {},
    #[route("/menubar")]
    MenubarPage {},
    #[route("/calendar")]
    CalendarPage {},
    #[route("/date-picker")]
    DatePickerPage {},
    #[route("/sidebar")]
    SidebarPage {},
    #[route("/accordion")]
    AccordionPage {},
    #[route("/alert-dialog")]
    AlertDialogPage {},
    #[route("/aspect-ratio")]
    AspectRatioPage {},
    #[route("/avatar")]
    AvatarPage {},
    #[route("/checkbox")]
    CheckboxPage {},
    #[route("/collapsible")]
    CollapsiblePage {},
    #[route("/color-picker")]
    ColorPickerPage {},
    #[route("/drag-and-drop-list")]
    DragAndDropListPage {},
    #[route("/label")]
    LabelPage {},
    #[route("/mode-toggle")]
    ModeTogglePage {},
    #[route("/progress")]
    ProgressPage {},
    #[route("/radio-group")]
    RadioGroupPage {},
    #[route("/scroll-area")]
    ScrollAreaPage {},
    #[route("/slider")]
    SliderPage {},
    #[route("/switch")]
    SwitchPage {},
    #[route("/tabs")]
    TabsPage {},
    #[route("/tag-group")]
    TagGroupPage {},
    #[route("/theme-switcher")]
    ThemeSwitcherPage {},
    #[route("/toast")]
    ToastPage {},
    #[route("/toggle")]
    TogglePage {},
    #[route("/toggle-group")]
    ToggleGroupPage {},
    #[route("/toolbar")]
    ToolbarPage {},
    #[route("/virtual-list")]
    VirtualListPage {},
    #[route("/alert")]
    AlertPage {},
    #[route("/empty")]
    EmptyPage {},
    #[route("/kbd")]
    KbdPage {},
    #[route("/spinner")]
    SpinnerPage {},
    #[route("/breadcrumb")]
    BreadcrumbPage {},
    #[route("/table")]
    TablePage {},
    #[route("/button-group")]
    ButtonGroupPage {},
    #[route("/input-group")]
    InputGroupPage {},
    #[route("/native-select")]
    NativeSelectPage {},
}

pub fn nav_items() -> Vec<(&'static str, Route)> {
    vec![
        ("Button", Route::ButtonPage {}),
        ("Badge", Route::BadgePage {}),
        ("Card", Route::CardPage {}),
        ("Input", Route::InputPage {}),
        ("Textarea", Route::TextareaPage {}),
        ("Skeleton", Route::SkeletonPage {}),
        ("Item", Route::ItemPage {}),
        ("Pagination", Route::PaginationPage {}),
        ("Dialog", Route::DialogPage {}),
        ("Sheet", Route::SheetPage {}),
        ("Select", Route::SelectPage {}),
        ("Combobox", Route::ComboboxPage {}),
        ("Tooltip", Route::TooltipPage {}),
        ("Popover", Route::PopoverPage {}),
        ("HoverCard", Route::HoverCardPage {}),
        ("DropdownMenu", Route::DropdownMenuPage {}),
        ("ContextMenu", Route::ContextMenuPage {}),
        ("Menubar", Route::MenubarPage {}),
        ("Calendar", Route::CalendarPage {}),
        ("DatePicker", Route::DatePickerPage {}),
        ("Sidebar", Route::SidebarPage {}),
        ("Accordion", Route::AccordionPage {}),
        ("AlertDialog", Route::AlertDialogPage {}),
        ("AspectRatio", Route::AspectRatioPage {}),
        ("Avatar", Route::AvatarPage {}),
        ("Checkbox", Route::CheckboxPage {}),
        ("Collapsible", Route::CollapsiblePage {}),
        ("ColorPicker", Route::ColorPickerPage {}),
        ("DragAndDropList", Route::DragAndDropListPage {}),
        ("Label", Route::LabelPage {}),
        ("ModeToggle", Route::ModeTogglePage {}),
        ("Progress", Route::ProgressPage {}),
        ("RadioGroup", Route::RadioGroupPage {}),
        ("ScrollArea", Route::ScrollAreaPage {}),
        ("Slider", Route::SliderPage {}),
        ("Switch", Route::SwitchPage {}),
        ("Tabs", Route::TabsPage {}),
        ("TagGroup", Route::TagGroupPage {}),
        ("ThemeSwitcher", Route::ThemeSwitcherPage {}),
        ("Toast", Route::ToastPage {}),
        ("Toggle", Route::TogglePage {}),
        ("ToggleGroup", Route::ToggleGroupPage {}),
        ("Toolbar", Route::ToolbarPage {}),
        ("VirtualList", Route::VirtualListPage {}),
        ("Alert", Route::AlertPage {}),
        ("Empty", Route::EmptyPage {}),
        ("Kbd", Route::KbdPage {}),
        ("Spinner", Route::SpinnerPage {}),
        ("Breadcrumb", Route::BreadcrumbPage {}),
        ("Table", Route::TablePage {}),
        ("ButtonGroup", Route::ButtonGroupPage {}),
        ("InputGroup", Route::InputGroupPage {}),
        ("NativeSelect", Route::NativeSelectPage {}),
    ]
}

#[component]
pub fn Layout() -> Element {
    let navigator = use_navigator();
    let current_route = use_route::<Route>();

    rsx! {
        components::ui::SidebarProvider {
            components::ui::Sidebar {
                components::ui::SidebarHeader {
                    Link { class: "flex shrink-0 items-center gap-2 text-lg font-bold", to: Route::Home {},
                        img { class: "size-8 rounded-md", src: PLAYGROUND_LOGO, alt: "adico logo" }
                        span { "adico playground" }
                    }
                }
                components::ui::SidebarContent {
                    components::ui::SidebarGroup {
                        components::ui::SidebarGroupContent {
                            components::ui::SidebarMenu {
                                for (label , route) in nav_items() {
                                    components::ui::SidebarMenuItem {
                                        div {
                                            onclick: move |_| { navigator.push(route.clone()); },
                                            components::ui::SidebarMenuButton {
                                                is_active: current_route == route,
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                components::ui::SidebarFooter {
                    components::ui::ModeToggle {}
                    components::ui::ThemeSwitcher {}
                    ThemeBuilderLauncher {}
                }
                components::ui::SidebarRail {}
            }
            components::ui::SidebarInset {
                div { class: "flex items-center gap-2 border-b border-border p-3",
                    components::ui::SidebarTrigger { "☰" }
                }
                div { class: "min-h-0 flex-1 overflow-y-auto p-3 lg:p-6",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
