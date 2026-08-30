//! The router: route declarations, the sidebar navigation list, and the
//! routing shell. dioxus-router has no file-system route generation, so
//! this enum still declares every path explicitly even though each page's
//! body now lives under `pages/` (see `pages/mod.rs`).

use dioxus::prelude::*;

use crate::pages::{
    AccordionPage, AlertDialogPage, AspectRatioPage, AvatarPage, BadgePage, ButtonPage,
    CalendarPage, CardPage, CheckboxPage, CollapsiblePage, ColorPickerPage, ComboboxPage,
    ContextMenuPage, DatePickerPage, DialogPage, DragAndDropListPage, DropdownMenuPage, Home,
    HoverCardPage, InputPage, ItemPage, LabelPage, MenubarPage, ModeTogglePage, PaginationPage,
    PopoverPage, ProgressPage, RadioGroupPage, ScrollAreaPage, SelectPage, SheetPage, SidebarPage,
    SkeletonPage, SliderPage, SwitchPage, TabsPage, TagGroupPage, TextareaPage, ThemeSwitcherPage,
    ToastPage, ToggleGroupPage, TogglePage, ToolbarPage, TooltipPage, VirtualListPage,
};
use crate::theme::{ThemeLauncher, ThemeModal, ThemeSelection};

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
    ]
}

#[component]
pub fn Layout() -> Element {
    let theme = use_context::<Signal<ThemeSelection>>();
    let theme_open = use_signal(|| false);

    rsx! {
        div { class: "mx-auto flex h-full min-h-0 w-full max-w-none flex-col gap-4 p-4 lg:w-4/5 lg:flex-row lg:gap-6 lg:p-6",
            nav { class: "flex min-h-0 w-full flex-col rounded-lg border border-sidebar-border bg-sidebar p-3 text-sidebar-foreground lg:w-60 lg:shrink-0",
                Link { class: "flex shrink-0 items-center gap-2 text-lg font-bold", to: Route::Home {},
                    img { class: "size-8 rounded-md", src: PLAYGROUND_LOGO, alt: "adico logo" }
                    span { "adico playground" }
                }
                ul { class: "mt-4 min-h-0 flex-1 space-y-1 overflow-y-auto pr-1 text-sm",
                    for (label , route) in nav_items() {
                        li {
                            Link { class: "block rounded px-2 py-1 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground", to: route, "{label}" }
                        }
                    }
                }
                ThemeLauncher { open: theme_open }
            }
            main { class: "min-h-0 min-w-0 flex-1 overflow-hidden rounded-lg border border-border bg-muted/30 p-3 lg:p-6",
                Outlet::<Route> {}
            }
            ThemeModal { theme, open: theme_open }
        }
    }
}
