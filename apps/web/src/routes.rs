//! The router: route declarations, the site shell, the playground's
//! sidebar navigation list, and the playground routing shell.
//! dioxus-router has no file-system route generation, so this enum still
//! declares every path explicitly even though each page's body lives under
//! `pages/` (see `pages/mod.rs`).

use adico_primitives::icons;
use dioxus::prelude::*;

use crate::components;
use crate::components::nav::NavList;
use crate::components::theme_builder_launcher::ThemeBuilderLauncher;
use crate::components::ui::mode_toggle::ModeToggle;
use crate::components::ui::navigation_menu::{
    NavigationMenu, NavigationMenuItem, NavigationMenuLink, NavigationMenuList,
};
use crate::components::ui::theme_switcher::ThemeSwitcher;
use crate::pages::docs::{DocsComponent, DocsIndex};
use crate::pages::playground::{
    AccordionPage, AlertDialogPage, AlertPage, AspectRatioPage, AttachmentPage, AvatarPage,
    BadgePage, BreadcrumbPage, BubblePage, ButtonGroupPage, ButtonPage, CalendarPage, CardPage,
    CarouselPage, CheckboxPage, CollapsiblePage, ColorPickerPage, ComboboxPage, CommandPage,
    ContextMenuPage, CopyButtonPage, DataTablePage, DatePickerPage, DateTimePickerPage, DialogPage,
    DragAndDropListPage, DrawerPage, DropdownMenuPage, EmptyPage, HoverCardPage, InputGroupPage,
    InputOTPPage, InputPage, ItemPage, KbdPage, LabelPage, MarkerPage, MenubarPage, MessagePage,
    MessageScrollerPage, ModeTogglePage, NativeSelectPage, NavigationMenuPage, PaginationPage,
    PlaygroundIndex, PopoverPage, ProgressPage, RadioGroupPage, ResizablePage, ScrollAreaPage,
    SelectPage, SheetPage, SidebarPage, SkeletonPage, SliderPage, SpinnerPage, SwitchPage,
    TablePage, TabsPage, TagGroupPage, TextareaPage, ThemeBuilderPage, ThemeSwitcherPage,
    TimePickerPage, ToastPage, ToggleGroupPage, TogglePage, ToolbarPage, TooltipPage,
    VirtualListPage,
};
use crate::pages::{Home, ResponsiveFlowPage, ResponsiveOverlayPage};

const SITE_LOGO: Asset = asset!("/assets/web/android-chrome-192x192.png");

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(SiteLayout)]
    #[route("/")]
    Home {},
    #[route("/docs")]
    DocsIndex {},
    #[route("/docs/components/:name")]
    DocsComponent { name: String },

    #[layout(PlaygroundLayout)]
    #[route("/playground")]
    PlaygroundIndex {},
    #[route("/playground/button")]
    ButtonPage {},
    #[route("/playground/badge")]
    BadgePage {},
    #[route("/playground/card")]
    CardPage {},
    #[route("/playground/input")]
    InputPage {},
    #[route("/playground/textarea")]
    TextareaPage {},
    #[route("/playground/skeleton")]
    SkeletonPage {},
    #[route("/playground/item")]
    ItemPage {},
    #[route("/playground/pagination")]
    PaginationPage {},
    #[route("/playground/dialog")]
    DialogPage {},
    #[route("/playground/sheet")]
    SheetPage {},
    #[route("/playground/select")]
    SelectPage {},
    #[route("/playground/combobox")]
    ComboboxPage {},
    #[route("/playground/command")]
    CommandPage {},
    #[route("/playground/tooltip")]
    TooltipPage {},
    #[route("/playground/popover")]
    PopoverPage {},
    #[route("/playground/hover-card")]
    HoverCardPage {},
    #[route("/playground/dropdown-menu")]
    DropdownMenuPage {},
    #[route("/playground/context-menu")]
    ContextMenuPage {},
    #[route("/playground/menubar")]
    MenubarPage {},
    #[route("/playground/calendar")]
    CalendarPage {},
    #[route("/playground/date-picker")]
    DatePickerPage {},
    #[route("/playground/time-picker")]
    TimePickerPage {},
    #[route("/playground/date-time-picker")]
    DateTimePickerPage {},
    #[route("/playground/sidebar")]
    SidebarPage {},
    #[route("/playground/accordion")]
    AccordionPage {},
    #[route("/playground/alert-dialog")]
    AlertDialogPage {},
    #[route("/playground/aspect-ratio")]
    AspectRatioPage {},
    #[route("/playground/avatar")]
    AvatarPage {},
    #[route("/playground/checkbox")]
    CheckboxPage {},
    #[route("/playground/collapsible")]
    CollapsiblePage {},
    #[route("/playground/color-picker")]
    ColorPickerPage {},
    #[route("/playground/drag-and-drop-list")]
    DragAndDropListPage {},
    #[route("/playground/label")]
    LabelPage {},
    #[route("/playground/mode-toggle")]
    ModeTogglePage {},
    #[route("/playground/progress")]
    ProgressPage {},
    #[route("/playground/radio-group")]
    RadioGroupPage {},
    #[route("/playground/scroll-area")]
    ScrollAreaPage {},
    #[route("/playground/slider")]
    SliderPage {},
    #[route("/playground/switch")]
    SwitchPage {},
    #[route("/playground/tabs")]
    TabsPage {},
    #[route("/playground/tag-group")]
    TagGroupPage {},
    #[route("/playground/theme-switcher")]
    ThemeSwitcherPage {},
    #[route("/playground/toast")]
    ToastPage {},
    #[route("/playground/toggle")]
    TogglePage {},
    #[route("/playground/toggle-group")]
    ToggleGroupPage {},
    #[route("/playground/toolbar")]
    ToolbarPage {},
    #[route("/playground/virtual-list")]
    VirtualListPage {},
    #[route("/playground/alert")]
    AlertPage {},
    #[route("/playground/empty")]
    EmptyPage {},
    #[route("/playground/kbd")]
    KbdPage {},
    #[route("/playground/spinner")]
    SpinnerPage {},
    #[route("/playground/breadcrumb")]
    BreadcrumbPage {},
    #[route("/playground/table")]
    TablePage {},
    #[route("/playground/button-group")]
    ButtonGroupPage {},
    #[route("/playground/input-group")]
    InputGroupPage {},
    #[route("/playground/native-select")]
    NativeSelectPage {},
    #[route("/playground/navigation-menu")]
    NavigationMenuPage {},
    #[route("/playground/drawer")]
    DrawerPage {},
    #[route("/playground/carousel")]
    CarouselPage {},
    #[route("/playground/input-otp")]
    InputOTPPage {},
    #[route("/playground/resizable")]
    ResizablePage {},
    #[route("/playground/attachment")]
    AttachmentPage {},
    #[route("/playground/bubble")]
    BubblePage {},
    #[route("/playground/copy-button")]
    CopyButtonPage {},
    #[route("/playground/data-table")]
    DataTablePage {},
    #[route("/playground/marker")]
    MarkerPage {},
    #[route("/playground/message")]
    MessagePage {},
    #[route("/playground/message-scroller")]
    MessageScrollerPage {},
    #[route("/playground/theme-builder")]
    ThemeBuilderPage {},
    #[end_layout]
    // End of the `/playground/*` subtree.
    #[end_layout]
    // End of the site-wide shell. Shell-free viewport-harness routes below
    // are intentionally NOT nested under `SiteLayout` or `PlaygroundLayout`
    // (see `pages/responsive_flow.rs`'s module doc) so the automated
    // responsive test suite measures real browser-viewport geometry, not
    // any shell's own fixed-percentage panels. Not part of `nav_items()` --
    // these are harness fixtures, not browsable pages.
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

/// The site-wide shell: header (logo, docs/playground/GitHub nav, theme
/// controls) shared by every route except the shell-free viewport-harness
/// routes. Composed from installed registry components, not an
/// app-specific reimplementation — see `adico-web-structure`'s "The site
/// shell composes real registry components, not app-specific
/// reimplementations" requirement.
#[component]
pub fn SiteLayout() -> Element {
    rsx! {
        div { class: "flex h-full min-h-0 flex-col",
            header { class: "border-b border-border",
                div { class: "mx-auto flex w-full max-w-5xl items-center justify-between gap-4 px-6 py-4",
                    Link { to: Route::Home {}, class: "flex items-center gap-2 text-lg font-semibold",
                        img { class: "size-6 rounded-md", src: SITE_LOGO, alt: "adico logo" }
                        "adico"
                    }
                    NavigationMenu {
                        NavigationMenuList {
                            NavigationMenuItem { index: 0usize,
                                NavigationMenuLink { href: "/docs", "Docs" }
                            }
                            NavigationMenuItem { index: 1usize,
                                NavigationMenuLink { href: "/playground", "Playground" }
                            }
                            NavigationMenuItem { index: 2usize,
                                NavigationMenuLink {
                                    href: "https://github.com/awwwkshay-org/awesome-dioxus-components",
                                    "GitHub"
                                }
                            }
                        }
                    }
                    div { class: "flex items-center gap-2",
                        ThemeSwitcher { show_label: false }
                        ModeToggle {}
                    }
                }
            }
            // `min-h-0` lets this shrink below its content's natural size so
            // `flex-1` can give it a definite height (required for
            // playground's percentage-based resizable splits, several
            // layers deeper, to resolve against); `overflow-y-auto` lets a
            // long docs/home page scroll internally within that fixed
            // height instead of needing the document itself to grow past
            // the viewport (blocked by `overflow-hidden` on the App root).
            main { class: "flex min-h-0 flex-1 flex-col overflow-y-auto", Outlet::<Route> {} }
        }
    }
}

/// The playground's own navigation shell (sidebar/nav-column, mobile
/// hamburger sheet, theme controls), nested inside `SiteLayout` under
/// `/playground/*`.
#[component]
pub fn PlaygroundLayout() -> Element {
    let navigator = use_navigator();
    let current_route = use_route::<Route>();
    let mut mobile_nav_open = use_signal(|| false);

    rsx! {
        div { class: "flex min-h-0 w-full flex-1 flex-col",
            // Mobile-only (`< md`) top bar + nav overlay. No JS viewport
            // detection: `flex md:hidden` on this bar and `hidden md:flex`
            // on the `>= md` nav column below are the only thing selecting
            // between them (registry/ui/sidebar.rs documents why a real
            // `document::eval`-based viewport check doesn't work in this
            // runtime). `mobile_nav_open` is only ever set by clicks (the
            // hamburger trigger, a nav selection, or Sheet's own dismissal),
            // never by measuring the viewport.
            div { class: "flex items-center justify-between gap-2 border-b border-border p-3 md:hidden",
                Link {
                    class: "flex min-w-0 items-center gap-2 text-lg font-bold",
                    to: Route::PlaygroundIndex {},
                    img { class: "size-8 shrink-0 rounded-md", src: SITE_LOGO, alt: "adico logo" }
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
                            Link {
                                class: "flex min-w-0 items-center gap-2 text-lg font-bold",
                                to: Route::PlaygroundIndex {},
                                img { class: "size-8 shrink-0 rounded-md", src: SITE_LOGO, alt: "adico logo" }
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
