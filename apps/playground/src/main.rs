use dioxus::prelude::*;

mod demo;
mod pages;

use pages::{
    BadgePage, ButtonPage, CalendarPage, CardPage, ComboboxPage, ContextMenuPage, DatePickerPage,
    DialogPage, DropdownMenuPage, HoverCardPage, InputPage, ItemPage, MenubarPage, PaginationPage,
    PopoverPage, SelectPage, SheetPage, SidebarPage, SkeletonPage, TextareaPage, TooltipPage,
};

// Compiled by `dx serve`/`dx build` from the project-root `tailwind.css`
// (which declares `@import "tailwindcss"` + `@source` + the adico theme
// tokens) into this generated asset. Any app installing components through
// `adico add` needs both: the root `tailwind.css` input file, and this
// `document::Stylesheet` link plus the `document` Dioxus feature enabled in
// Cargo.toml — `adico add`'s CSS step does not wire the link or the feature
// automatically yet.
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

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
    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        Router::<Route> {}
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
    rsx! {
        div { class: "mx-auto flex max-w-5xl gap-8 p-8",
            nav { class: "w-48 shrink-0 space-y-1",
                Link { class: "block text-lg font-bold", to: Route::Home {}, "adico playground" }
                ul { class: "mt-4 space-y-1 text-sm",
                    for (label , route) in nav_items() {
                        li {
                            Link { class: "block rounded px-2 py-1 hover:bg-accent", to: route, "{label}" }
                        }
                    }
                }
            }
            main { class: "min-w-0 flex-1 space-y-2",
                Outlet::<Route> {}
            }
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
