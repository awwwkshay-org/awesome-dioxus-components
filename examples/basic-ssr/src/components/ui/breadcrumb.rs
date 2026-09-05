//! Source-owned shadcn-style Breadcrumb for Dioxus.

use dioxus::prelude::*;

use adico_primitives::icons::{ChevronRight, Ellipsis};

use crate::adico_lib::cn::cn;

/// The `<nav aria-label="breadcrumb">` root.
#[component]
pub fn Breadcrumb(class: Option<String>, children: Element) -> Element {
    rsx! {
        nav { "aria-label": "breadcrumb", class: class.unwrap_or_default(), {children} }
    }
}

/// The ordered list of [`BreadcrumbItem`]s.
#[component]
pub fn BreadcrumbList(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "flex flex-wrap items-center gap-1.5 text-sm text-muted-foreground break-words sm:gap-2.5",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ol { class, {children} }
    }
}

/// A single breadcrumb entry, wrapping a [`BreadcrumbLink`] or
/// [`BreadcrumbPage`].
#[component]
pub fn BreadcrumbItem(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "inline-flex items-center gap-1.5",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        li { class, {children} }
    }
}

/// Props for [`BreadcrumbLink`].
#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbLinkProps {
    /// The link target.
    #[props(default)]
    pub href: Option<String>,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native anchor/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = a)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// A navigable breadcrumb entry.
#[component]
pub fn BreadcrumbLink(props: BreadcrumbLinkProps) -> Element {
    let class = cn(&[
        "transition-colors hover:text-foreground",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        a { class, href: props.href, ..props.attributes, {props.children} }
    }
}

/// The current (non-navigable) breadcrumb entry.
#[component]
pub fn BreadcrumbPage(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "font-normal text-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span {
            class,
            role: "link",
            "aria-disabled": "true",
            "aria-current": "page",
            {children}
        }
    }
}

/// A separator between breadcrumb entries. Defaults to a chevron when no
/// children are supplied.
#[component]
pub fn BreadcrumbSeparator(class: Option<String>, children: Option<Element>) -> Element {
    let class = cn(&["[&>svg]:size-3.5", class.as_deref().unwrap_or_default()]);
    rsx! {
        li {
            role: "presentation",
            "aria-hidden": "true",
            class,
            {children.unwrap_or_else(|| rsx! { ChevronRight {} })}
        }
    }
}

/// A collapsed-run indicator (`...`) for long breadcrumb trails.
///
/// Renders the Lucide `Ellipsis` icon (this icon set's current name for
/// upstream shadcn's `MoreHorizontalIcon`, which Lucide itself renamed).
#[component]
pub fn BreadcrumbEllipsis(class: Option<String>) -> Element {
    let class = cn(&[
        "flex size-9 items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span { role: "presentation", "aria-hidden": "true", class,
            Ellipsis { class: "size-4" }
            span { class: "sr-only", "More" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_uses_a_muted_semantic_surface() {
        let class = cn(&[
            "flex flex-wrap items-center gap-1.5 text-sm text-muted-foreground break-words sm:gap-2.5",
        ]);
        assert!(class.contains("text-muted-foreground"));
    }

    #[test]
    fn current_page_uses_the_foreground_token() {
        let class = cn(&["font-normal text-foreground"]);
        assert!(class.contains("text-foreground"));
    }
}
