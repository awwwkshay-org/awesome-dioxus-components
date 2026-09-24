//! Router definitions for `apps/home` — kept out of `main.rs` per the
//! `adico-home-structure` spec's "Router definitions live in routes.rs"
//! requirement.

use dioxus::prelude::*;

use crate::components::ui::mode_toggle::ModeToggle;
use crate::components::ui::navigation_menu::{
    NavigationMenu, NavigationMenuItem, NavigationMenuLink, NavigationMenuList,
};
use crate::components::ui::theme_switcher::ThemeSwitcher;
use crate::pages::Home;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
}

#[component]
pub fn Layout() -> Element {
    rsx! {
        div { class: "flex min-h-dvh flex-col bg-background text-foreground",
            header { class: "border-b border-border",
                div { class: "mx-auto flex w-full max-w-5xl items-center justify-between gap-4 px-6 py-4",
                    Link { to: Route::Home {}, class: "text-lg font-semibold", "adico" }
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
            main { class: "flex-1", Outlet::<Route> {} }
        }
    }
}
