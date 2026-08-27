use dioxus::prelude::*;

mod controls;
mod demo;
mod pages;
mod theme;

use pages::{
    BadgePage, ButtonPage, CalendarPage, CardPage, ComboboxPage, ContextMenuPage, DatePickerPage,
    DialogPage, DropdownMenuPage, HoverCardPage, InputPage, ItemPage, MenubarPage, PaginationPage,
    PopoverPage, SelectPage, SheetPage, SidebarPage, SkeletonPage, TextareaPage, TooltipPage,
};
use theme::{ThemeLauncher, ThemeModal, ThemeSelection};

// Compiled by `dx serve`/`dx build` from the project-root `tailwind.css`
// (which declares `@import "tailwindcss"` + `@source` + the adico theme
// tokens) into this generated asset. Any app installing components through
// `adico add` needs both: the root `tailwind.css` input file, and this
// `document::Stylesheet` link plus the `document` Dioxus feature enabled in
// Cargo.toml — `adico add`'s CSS step does not wire the link or the feature
// automatically yet.
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const PLAYGROUND_LOGO: Asset = asset!("/assets/web/android-chrome-192x192.png");
const WEB_MANIFEST: Asset = asset!("/assets/web/site.webmanifest");
const FAVICON: Asset = asset!("/assets/web/favicon.ico");
const FAVICON_16: Asset = asset!("/assets/web/favicon-16x16.png");
const FAVICON_32: Asset = asset!("/assets/web/favicon-32x32.png");
const APPLE_TOUCH_ICON: Asset = asset!("/assets/web/apple-touch-icon.png");

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
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
}

#[component]
fn App() -> Element {
    let theme = use_signal(ThemeSelection::default);
    use_context_provider(|| theme);
    let selection = theme();

    rsx! {
        document::Title { "adico playground" }
        document::Stylesheet { href: TAILWIND_CSS }
        document::Link { rel: "manifest", href: WEB_MANIFEST }
        document::Link { rel: "shortcut icon", r#type: "image/x-icon", href: FAVICON }
        document::Link { rel: "icon", r#type: "image/png", sizes: "16x16", href: FAVICON_16 }
        document::Link { rel: "icon", r#type: "image/png", sizes: "32x32", href: FAVICON_32 }
        document::Link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON }
        div { class: "{selection.shell_class()}", style: "{selection.variables()}",
            Router::<Route> {}
        }
    }
}

fn nav_items() -> Vec<(&'static str, Route)> {
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
    ]
}

#[component]
fn Layout() -> Element {
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

#[component]
fn Home() -> Element {
    rsx! {
        h1 { class: "text-2xl font-bold", "adico playground" }
        p { class: "text-sm text-muted-foreground",
            "Every currently migrated component, installed through the real `adico` CLI. Pick one from the list to see its live demo."
        }
    }
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
