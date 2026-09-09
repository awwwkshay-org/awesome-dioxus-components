//! The router: route declarations, the sidebar navigation list, and the
//! routing shell. dioxus-router has no file-system route generation, so
//! this enum still declares every path explicitly even though each page's
//! body now lives under `pages/` (see `pages/mod.rs`).

use dioxus::prelude::*;

use crate::components;
use crate::components::theme_builder_launcher::ThemeBuilderLauncher;
use crate::pages::{
    AccordionPage, AlertDialogPage, AlertPage, AspectRatioPage, AttachmentPage, AvatarPage,
    BadgePage, BreadcrumbPage, BubblePage, ButtonGroupPage, ButtonPage, CalendarPage, CardPage,
    CarouselPage, CheckboxPage, CollapsiblePage, ColorPickerPage, ComboboxPage, CommandPage,
    ContextMenuPage, DataTablePage, DatePickerPage, DateTimePickerPage, DialogPage,
    DragAndDropListPage, DrawerPage, DropdownMenuPage, EmptyPage, Home, HoverCardPage,
    InputGroupPage, InputOTPPage, InputPage, ItemPage, KbdPage, LabelPage, MarkerPage, MenubarPage,
    MessagePage, MessageScrollerPage, ModeTogglePage, NativeSelectPage, NavigationMenuPage,
    PaginationPage, PopoverPage, ProgressPage, RadioGroupPage, ResizablePage, ResponsiveFlowPage,
    ResponsiveOverlayPage, ScrollAreaPage, SelectPage, SheetPage, SidebarPage, SkeletonPage,
    SliderPage, SpinnerPage, SwitchPage, TablePage, TabsPage, TagGroupPage, TextareaPage,
    ThemeBuilderPage, ThemeSwitcherPage, TimePickerPage, ToastPage, ToggleGroupPage, TogglePage,
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
    #[route("/command")]
    CommandPage {},
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
    #[route("/time-picker")]
    TimePickerPage {},
    #[route("/date-time-picker")]
    DateTimePickerPage {},
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
    #[route("/navigation-menu")]
    NavigationMenuPage {},
    #[route("/drawer")]
    DrawerPage {},
    #[route("/carousel")]
    CarouselPage {},
    #[route("/input-otp")]
    InputOTPPage {},
    #[route("/resizable")]
    ResizablePage {},
    #[route("/attachment")]
    AttachmentPage {},
    #[route("/bubble")]
    BubblePage {},
    #[route("/data-table")]
    DataTablePage {},
    #[route("/marker")]
    MarkerPage {},
    #[route("/message")]
    MessagePage {},
    #[route("/message-scroller")]
    MessageScrollerPage {},
    #[route("/theme-builder")]
    ThemeBuilderPage {},
    // Shell-free viewport-harness routes: intentionally NOT nested under
    // `Layout` (see `pages/responsive_flow.rs`'s module doc) so the
    // automated responsive test suite measures real browser-viewport
    // geometry, not the shell's own fixed-percentage panels. Not part of
    // `nav_items()` -- these are harness fixtures, not browsable pages.
    #[end_layout]
    #[route("/responsive/flow")]
    ResponsiveFlowPage {},
    #[route("/responsive/overlay?:case")]
    ResponsiveOverlayPage { case: String },
}

/// One flat list, alphabetical ascending by displayed label — no thematic
/// batches. A new component page is inserted at its alphabetical position.
pub fn nav_items() -> Vec<(&'static str, Route)> {
    vec![
        ("Accordion", Route::AccordionPage {}),
        ("Alert", Route::AlertPage {}),
        ("Alert Dialog", Route::AlertDialogPage {}),
        ("Aspect Ratio", Route::AspectRatioPage {}),
        ("Attachment", Route::AttachmentPage {}),
        ("Avatar", Route::AvatarPage {}),
        ("Badge", Route::BadgePage {}),
        ("Breadcrumb", Route::BreadcrumbPage {}),
        ("Bubble", Route::BubblePage {}),
        ("Button", Route::ButtonPage {}),
        ("Button Group", Route::ButtonGroupPage {}),
        ("Calendar", Route::CalendarPage {}),
        ("Card", Route::CardPage {}),
        ("Carousel", Route::CarouselPage {}),
        ("Checkbox", Route::CheckboxPage {}),
        ("Collapsible", Route::CollapsiblePage {}),
        ("Color Picker", Route::ColorPickerPage {}),
        ("Combobox", Route::ComboboxPage {}),
        ("Command", Route::CommandPage {}),
        ("Context Menu", Route::ContextMenuPage {}),
        ("Data Table", Route::DataTablePage {}),
        ("Date Picker", Route::DatePickerPage {}),
        ("Date Time Picker", Route::DateTimePickerPage {}),
        ("Dialog", Route::DialogPage {}),
        ("Drag And Drop List", Route::DragAndDropListPage {}),
        ("Drawer", Route::DrawerPage {}),
        ("Dropdown Menu", Route::DropdownMenuPage {}),
        ("Empty", Route::EmptyPage {}),
        ("Hover Card", Route::HoverCardPage {}),
        ("Input", Route::InputPage {}),
        ("Input Group", Route::InputGroupPage {}),
        ("Input OTP", Route::InputOTPPage {}),
        ("Item", Route::ItemPage {}),
        ("Kbd", Route::KbdPage {}),
        ("Label", Route::LabelPage {}),
        ("Marker", Route::MarkerPage {}),
        ("Menubar", Route::MenubarPage {}),
        ("Message", Route::MessagePage {}),
        ("Message Scroller", Route::MessageScrollerPage {}),
        ("Mode Toggle", Route::ModeTogglePage {}),
        ("Native Select", Route::NativeSelectPage {}),
        ("Navigation Menu", Route::NavigationMenuPage {}),
        ("Pagination", Route::PaginationPage {}),
        ("Popover", Route::PopoverPage {}),
        ("Progress", Route::ProgressPage {}),
        ("Radio Group", Route::RadioGroupPage {}),
        ("Resizable", Route::ResizablePage {}),
        ("Scroll Area", Route::ScrollAreaPage {}),
        ("Select", Route::SelectPage {}),
        ("Sheet", Route::SheetPage {}),
        ("Sidebar", Route::SidebarPage {}),
        ("Skeleton", Route::SkeletonPage {}),
        ("Slider", Route::SliderPage {}),
        ("Spinner", Route::SpinnerPage {}),
        ("Switch", Route::SwitchPage {}),
        ("Table", Route::TablePage {}),
        ("Tabs", Route::TabsPage {}),
        ("Tag Group", Route::TagGroupPage {}),
        ("Textarea", Route::TextareaPage {}),
        ("Theme Builder", Route::ThemeBuilderPage {}),
        ("Theme Switcher", Route::ThemeSwitcherPage {}),
        ("Time Picker", Route::TimePickerPage {}),
        ("Toast", Route::ToastPage {}),
        ("Toggle", Route::TogglePage {}),
        ("Toggle Group", Route::ToggleGroupPage {}),
        ("Toolbar", Route::ToolbarPage {}),
        ("Tooltip", Route::TooltipPage {}),
        ("Virtual List", Route::VirtualListPage {}),
    ]
}

#[component]
pub fn Layout() -> Element {
    let navigator = use_navigator();
    let current_route = use_route::<Route>();

    rsx! {
        components::ui::ResizablePanelGroup {
            direction: components::ui::ResizableDirection::Horizontal,
            class: "h-full w-full",
            components::ui::ResizablePanel {
                index: 0usize,
                default_size: 18.0,
                min_size: 12.0,
                max_size: 30.0,
                class: "flex h-full min-w-0 flex-col",
                components::ui::SidebarHeader {
                    // `Link`'s own `shrink-0` (harmless under the old fixed
                    // 16rem `Sidebar`, which never got narrow enough for it
                    // to matter) actively fights a resizable nav column: it
                    // stops this row from shrinking at all, so "adico
                    // playground" is forced to wrap instead of truncating
                    // once the column is dragged narrow. `min-w-0` +
                    // wrapping the text in its own `truncate` span lets the
                    // row shrink and elide instead.
                    Link { class: "flex min-w-0 items-center gap-2 text-lg font-bold", to: Route::Home {},
                        img { class: "size-8 shrink-0 rounded-md", src: PLAYGROUND_LOGO, alt: "adico logo" }
                        span { class: "min-w-0 truncate", "Adico Playground" }
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
                                                // `SidebarMenuButton` passes
                                                // its children straight
                                                // through with no
                                                // truncation handling of its
                                                // own (its own root button
                                                // has `overflow-hidden` but
                                                // not `whitespace-nowrap`,
                                                // so a bare text child still
                                                // wraps rather than eliding)
                                                // -- wrapping the label in
                                                // its own `min-w-0 truncate`
                                                // span here is this
                                                // playground's own
                                                // composition choice, not a
                                                // registry change.
                                                span { class: "min-w-0 flex-1 truncate", "{label}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                components::ui::SidebarFooter {
                    div { class: "flex items-end gap-2",
                        components::ui::ModeToggle {}
                        components::ui::ThemeSwitcher { class: "flex-1", show_label: false }
                    }
                    ThemeBuilderLauncher {}
                }
            }
            components::ui::ResizableHandle { handle_index: 0usize, with_handle: true }
            components::ui::ResizablePanel {
                index: 1usize,
                default_size: 82.0,
                min_size: 70.0,
                max_size: 88.0,
                class: "flex h-full min-h-0 flex-col",
                div { class: "min-h-0 flex-1 overflow-y-auto p-3 lg:p-6",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
