//! Source-owned shadcn-style Pagination composition for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// The outer landmark wrapping a page-link list.
#[component]
pub fn Pagination(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "mx-auto flex w-full justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        nav {
            class,
            role: "navigation",
            "aria-label": "pagination",
            {children}
        }
    }
}

/// The row of [`PaginationItem`]s.
#[component]
pub fn PaginationContent(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-row items-center gap-1",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ul { class, {children} }
    }
}

/// A single entry in a [`PaginationContent`] list.
#[component]
pub fn PaginationItem(children: Element, class: Option<String>) -> Element {
    let class = cn(&[class.as_deref().unwrap_or_default()]);
    rsx! {
        li { class, {children} }
    }
}

/// A clickable page link, styled as active when it represents the current page.
#[component]
pub fn PaginationLink(
    children: Element,
    #[props(default)] is_active: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    aria_label: Option<String>,
    class: Option<String>,
) -> Element {
    let state_class = if is_active {
        "border border-input bg-background"
    } else {
        "hover:bg-accent hover:text-accent-foreground"
    };
    let class = cn(&[
        "inline-flex h-9 w-9 items-center justify-center rounded-md text-sm font-medium transition-colors",
        state_class,
        class.as_deref().unwrap_or_default(),
    ]);
    let aria_current = is_active.then_some("page");
    rsx! {
        a {
            class,
            href: "#",
            aria_current,
            aria_label,
            onclick: move |event| {
                event.prevent_default();
                onclick.call(event);
            },
            {children}
        }
    }
}

/// A [`PaginationLink`] preset for moving to the previous page.
#[component]
pub fn PaginationPrevious(
    #[props(default)] onclick: EventHandler<MouseEvent>,
    class: Option<String>,
) -> Element {
    let class = cn(&["gap-1 pl-2.5", class.as_deref().unwrap_or_default()]);
    rsx! {
        PaginationLink {
            class,
            onclick: move |event| onclick.call(event),
            aria_label: "Go to previous page",
            span { "aria-hidden": "true", "‹" }
            span { "Previous" }
        }
    }
}

/// A [`PaginationLink`] preset for moving to the next page.
#[component]
pub fn PaginationNext(
    #[props(default)] onclick: EventHandler<MouseEvent>,
    class: Option<String>,
) -> Element {
    let class = cn(&["gap-1 pr-2.5", class.as_deref().unwrap_or_default()]);
    rsx! {
        PaginationLink {
            class,
            onclick: move |event| onclick.call(event),
            aria_label: "Go to next page",
            span { "Next" }
            span { "aria-hidden": "true", "›" }
        }
    }
}

/// A non-interactive marker for skipped pages between [`PaginationLink`]s.
#[component]
pub fn PaginationEllipsis(class: Option<String>) -> Element {
    let class = cn(&[
        "flex h-9 w-9 items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span {
            class,
            "aria-hidden": "true",
            "…"
            span { class: "sr-only", "More pages" }
        }
    }
}
