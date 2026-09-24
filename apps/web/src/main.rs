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
// The self-hosted Geist / Geist Mono faces, declared as a *folder* asset.
//
// `dx` only copies files reachable from an `asset!()`, and it fingerprints
// each one into a flat `/assets/<name>-<hash>.<ext>` — so a plain per-file
// `asset!()` would give the fonts unpredictable names that the `@font-face`
// `url()` in `tailwind.css` could never spell. A folder asset is the
// exception: `FolderAssetOptions` sets `add_hash: false` and preserves the
// directory's internal structure, so these land at a stable `/assets/fonts/
// <original name>` — which is exactly what the stylesheet's relative
// `url("./fonts/…")` resolves to from `/assets/tailwind-<hash>.css`.
const FONTS: Asset = asset!("/assets/fonts", AssetOptions::folder());
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
        document::Title { "adico" }
        document::Stylesheet { href: TAILWIND_CSS }
        // Preloaded because the body face is on the critical render path:
        // without it the browser only discovers the font after parsing the
        // stylesheet's `@font-face`, which lengthens the `font-display: swap`
        // fallback flash. Also keeps `FONTS` referenced so the folder asset is
        // actually collected.
        document::Link {
            rel: "preload",
            href: format!("{FONTS}/geist-latin-wght-normal.woff2"),
            r#as: "font",
            r#type: "font/woff2",
            crossorigin: "anonymous",
        }
        document::Link { rel: "manifest", href: WEB_MANIFEST }
        document::Link { rel: "shortcut icon", r#type: "image/x-icon", href: FAVICON }
        document::Link { rel: "icon", r#type: "image/png", sizes: "16x16", href: FAVICON_16 }
        document::Link { rel: "icon", r#type: "image/png", sizes: "32x32", href: FAVICON_32 }
        document::Link { rel: "apple-touch-icon", href: APPLE_TOUCH_ICON }
        // `h-dvh` (an exact, definite height), not `min-h-dvh`: playground's
        // resizable preview/controls split (`Demo`) uses percentage
        // `flex-basis` inside a flex column, which only resolves against a
        // *definite* ancestor height -- a `min-height`-only container can
        // still report a definite used height when content is shorter than
        // the viewport, but the moment nested `min-h-0`/`flex-1` panels are
        // involved several layers deep, that chain is fragile. `overflow-hidden`
        // here plus `overflow-y-auto` on `SiteLayout`'s `main` keeps a docs/
        // home page's own long content scrolling internally rather than
        // requiring the whole document to grow past the viewport.
        div { class: "h-dvh w-full overflow-hidden bg-background text-foreground", Router::<Route> {} }
    }
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
