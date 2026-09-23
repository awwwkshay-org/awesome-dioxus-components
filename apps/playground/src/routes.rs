//! The router: route declarations, the sidebar navigation list, and the
//! routing shell. dioxus-router has no file-system route generation, so
//! this enum still declares every path explicitly even though each page's
//! body now lives under `pages/` (see `pages/mod.rs`).

use adico_primitives::icons;
use dioxus::prelude::*;

use crate::components;
use crate::components::nav::NavList;
use crate::components::theme_builder_launcher::ThemeBuilderLauncher;
use crate::pages::{
    AccordionPage, AlertDialogPage, AlertPage, AspectRatioPage, AttachmentPage, AvatarPage,
    BadgePage, BreadcrumbPage, BubblePage, ButtonGroupPage, ButtonPage, CalendarPage, CardPage,
    CarouselPage, CheckboxPage, CollapsiblePage, ColorPickerPage, ComboboxPage, CommandPage,
    ContextMenuPage, CopyButtonPage, DataTablePage, DatePickerPage, DateTimePickerPage, DialogPage,
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
    #[route("/copy-button")]
    CopyButtonPage {},
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
        ("Copy Button", Route::CopyButtonPage {}),
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
    let mut mobile_nav_open = use_signal(|| false);

    rsx! {
        div { class: "flex h-full w-full flex-col",
            // Mobile-only (`< md`) top bar + nav overlay. No JS viewport
            // detection: `flex md:hidden` on this bar and `hidden md:flex`
            // on the `>= md` nav column below are the only thing selecting
            // between them (registry/ui/sidebar.rs documents why a real
            // `document::eval`-based viewport check doesn't work in this
            // runtime). `mobile_nav_open` is only ever set by clicks (the
            // hamburger trigger, a nav selection, or Sheet's own dismissal),
            // never by measuring the viewport.
            div { class: "flex items-center justify-between gap-2 border-b border-border p-3 md:hidden",
                Link { class: "flex min-w-0 items-center gap-2 text-lg font-bold", to: Route::Home {},
                    img { class: "size-8 shrink-0 rounded-md", src: PLAYGROUND_LOGO, alt: "adico logo" }
                    span { class: "min-w-0 truncate", "Adico Playground" }
                }
                components::ui::Sheet {
                    open: mobile_nav_open(),
                    on_open_change: move |value| mobile_nav_open.set(value),
                    components::ui::SheetTrigger {
                        variant: components::ui::ButtonVariant::Ghost,
                        size: components::ui::ButtonSize::Icon,
                        aria_label: "Open navigation",
                        icons::Menu { class: "size-5" }
                    }
                    components::ui::SheetOverlay {}
                    components::ui::SheetContent {
                        side: components::ui::SheetSide::Left,
                        class: "flex w-3/4 max-w-xs flex-col gap-4 p-4",
                        div { class: "min-h-0 flex-1 overflow-y-auto",
                            NavList {
                                current_route: current_route.clone(),
                                onnavigate: move |route| {
                                    mobile_nav_open.set(false);
                                    navigator.push(route);
                                },
                            }
                        }
                        div { class: "flex shrink-0 flex-col gap-2 border-t border-border pt-4",
                            div { class: "flex items-end gap-2",
                                components::ui::ModeToggle {}
                                components::ui::ThemeSwitcher { class: "flex-1", show_label: false }
                            }
                            ThemeBuilderLauncher {}
                        }
                    }
                }
            }
            // The one and only `Outlet` in this shell -- mounted exactly
            // once so routed page state (and any positioner-portalled
            // content a page renders) never exists in two places at once.
            // Below `md` the nav column and its resize handle are
            // `display:none` (freeing their row space) and the content
            // panel's `max-md:flex-1!` overrides its own `flex: 0 0 {size}%`
            // inline style (the resizable drag mechanism's own state) to
            // fill the row -- at `>= md` neither `max-md:` class ever
            // applies, so this tree renders byte-identical to before.
            div { class: "min-h-0 flex-1",
                components::ui::ResizablePanelGroup {
                    direction: components::ui::ResizableDirection::Horizontal,
                    class: "h-full w-full",
                    components::ui::ResizablePanel {
                        index: 0usize,
                        default_size: 18.0,
                        min_size: 12.0,
                        max_size: 30.0,
                        class: "hidden h-full min-w-0 flex-col md:flex",
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
                                    NavList {
                                        current_route: current_route.clone(),
                                        onnavigate: move |route| {
                                            navigator.push(route);
                                        },
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
                    components::ui::ResizableHandle {
                        handle_index: 0usize,
                        with_handle: true,
                        class: "hidden md:flex",
                    }
                    components::ui::ResizablePanel {
                        index: 1usize,
                        default_size: 82.0,
                        min_size: 70.0,
                        max_size: 88.0,
                        class: "flex h-full min-h-0 flex-col max-md:flex-1!",
                        div { class: "min-h-0 flex-1 overflow-y-auto p-3 lg:p-6",
                            Outlet::<Route> {}
                        }
                    }
                }
            }
        }
    }
}
