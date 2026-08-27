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

/// Props for [`PaginationLink`].
#[derive(Props, Clone, PartialEq)]
pub struct PaginationLinkProps {
    /// Whether this link represents the current page.
    #[props(default)]
    pub is_active: bool,
    /// Destination for the native anchor. Omit for an action-only link.
    #[props(default)]
    pub href: Option<String>,
    /// Optional page-change handler. Native navigation is preserved whenever
    /// `href` is supplied.
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    /// Accessible label for an icon-only or otherwise abbreviated link.
    #[props(default)]
    pub aria_label: Option<String>,
    /// Extra semantic classes appended to the default treatment.
    #[props(default)]
    pub class: Option<String>,
    /// Native anchor and global Dioxus attributes, including `target`, `rel`,
    /// `download`, ARIA properties, and event handlers.
    #[props(extends = GlobalAttributes)]
    #[props(extends = a)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed link content.
    pub children: Element,
}

/// A clickable page link, styled as active when it represents the current page.
#[component]
pub fn PaginationLink(props: PaginationLinkProps) -> Element {
    let state_class = if props.is_active {
        "border border-input bg-background"
    } else {
        "hover:bg-accent hover:text-accent-foreground"
    };
    let class = cn(&[
        "inline-flex h-9 w-9 items-center justify-center rounded-md text-sm font-medium transition-colors",
        state_class,
        props.class.as_deref().unwrap_or_default(),
    ]);
    let aria_current = props.is_active.then_some("page");
    let action_only = props.href.is_none();
    rsx! {
        a {
            class,
            href: props.href.unwrap_or_else(|| "#".to_string()),
            aria_current,
            aria_label: props.aria_label,
            onclick: move |event| {
                if action_only {
                    event.prevent_default();
                }
                props.onclick.call(event);
            },
            ..props.attributes,
            {props.children}
        }
    }
}

/// A [`PaginationLink`] preset for moving to the previous page.
#[component]
pub fn PaginationPrevious(
    #[props(default)] onclick: EventHandler<MouseEvent>,
    /// Native anchor destination.
    href: Option<String>,
    /// Visible label. Defaults to `Previous`.
    text: Option<String>,
    /// Shows only the direction icon while retaining the accessible label.
    #[props(default)]
    compact: bool,
    class: Option<String>,
) -> Element {
    let class = cn(&["gap-1 pl-2.5", class.as_deref().unwrap_or_default()]);
    let text = text.unwrap_or_else(|| "Previous".to_string());
    rsx! {
        PaginationLink {
            class,
            href,
            onclick: move |event| onclick.call(event),
            aria_label: "Go to previous page",
            span { "aria-hidden": "true", "‹" }
            if !compact { span { "{text}" } }
        }
    }
}

/// A [`PaginationLink`] preset for moving to the next page.
#[component]
pub fn PaginationNext(
    #[props(default)] onclick: EventHandler<MouseEvent>,
    /// Native anchor destination.
    href: Option<String>,
    /// Visible label. Defaults to `Next`.
    text: Option<String>,
    /// Shows only the direction icon while retaining the accessible label.
    #[props(default)]
    compact: bool,
    class: Option<String>,
) -> Element {
    let class = cn(&["gap-1 pr-2.5", class.as_deref().unwrap_or_default()]);
    let text = text.unwrap_or_else(|| "Next".to_string());
    rsx! {
        PaginationLink {
            class,
            href,
            onclick: move |event| onclick.call(event),
            aria_label: "Go to next page",
            if !compact { span { "{text}" } }
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
