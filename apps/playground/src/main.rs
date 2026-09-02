use dioxus::prelude::*;

mod generated;
mod pages;
mod routes;

use routes::Route;

// Compiled by `dx serve`/`dx build` from the project-root `tailwind.css`
// (which declares `@import "tailwindcss"` + `@source` + the adico theme
// tokens) into this generated asset. Any app installing components through
// `adico add` needs both: the root `tailwind.css` input file, and this
// `document::Stylesheet` link plus the `document` Dioxus feature in
// Cargo.toml — `adico add`'s CSS step does not wire the link or the feature
// automatically yet.
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const WEB_MANIFEST: Asset = asset!("/assets/web/site.webmanifest");
const FAVICON: Asset = asset!("/assets/web/favicon.ico");
const FAVICON_16: Asset = asset!("/assets/web/favicon-16x16.png");
const FAVICON_32: Asset = asset!("/assets/web/favicon-32x32.png");
const APPLE_TOUCH_ICON: Asset = asset!("/assets/web/apple-touch-icon.png");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Title { "adico playground" }
        document::Stylesheet { href: TAILWIND_CSS }
        document::Link { rel: "manifest", href: WEB_MANIFEST }
        document::Link { rel: "shortcut icon", r#type: "image/x-icon", href: FAVICON }
        document::Link { rel: "icon", r#type: "image/png", sizes: "16x16", href: FAVICON_16 }
        document::Link { rel: "icon", r#type: "image/png", sizes: "32x32", href: FAVICON_32 }
        document::Link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON }
        div { class: "h-screen w-screen overflow-hidden bg-background text-foreground", Router::<Route> {} }
    }
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
