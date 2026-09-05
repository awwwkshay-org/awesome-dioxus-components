//! Source-owned shadcn-style Pagination composition for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;
use crate::components::ui::spinner::Spinner;

/// The outer landmark wrapping a page-link list.
#[component]
pub fn Pagination(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "mx-auto flex w-full justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        nav {
            class,
            role: "navigation",
            "aria-label": "pagination",
            ..attributes,
            {children}
        }
    }
}

/// The row of [`PaginationItem`]s.
#[component]
pub fn PaginationContent(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex flex-row items-center gap-1",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        ul { class, ..attributes, {children} }
    }
}

/// A single entry in a [`PaginationContent`] list.
#[component]
pub fn PaginationItem(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[class.as_deref().unwrap_or_default()]);
    rsx! {
        li { class, ..attributes, {children} }
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
    /// Corner radius of the link surface.
    #[props(default = Radius::Md)]
    pub radius: Radius,
    /// Shows a [`Spinner`] and marks the link busy; also disables its
    /// click behavior, since a native anchor has no `disabled` attribute
    /// to defer to. An adico extension — shadcn's own convention is
    /// composing `<Button disabled><Spinner /></Button>` by hand.
    #[props(default)]
    pub loading: bool,
    /// Replaces the link's visible content while `loading` is true.
    #[props(default)]
    pub loading_text: Option<String>,
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
        "inline-flex h-9 w-9 items-center justify-center text-sm font-medium outline-none transition-colors focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50",
        state_class,
        props.radius.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    let aria_current = props.is_active.then_some("page");
    let action_only = props.href.is_none();
    let loading = props.loading;
    rsx! {
        a {
            class,
            href: props.href.unwrap_or_else(|| "#".to_string()),
            aria_current,
            aria_busy: loading,
            "aria-disabled": loading,
            aria_label: props.aria_label,
            onclick: move |event| {
                if action_only || loading {
                    event.prevent_default();
                }
                if !loading {
                    props.onclick.call(event);
                }
            },
            ..props.attributes,
            if loading {
                Spinner {}
                if let Some(text) = props.loading_text {
                    "{text}"
                } else {
                    {props.children}
                }
            } else {
                {props.children}
            }
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
    /// Native anchor and global Dioxus attributes, forwarded to the
    /// underlying [`PaginationLink`].
    #[props(extends = GlobalAttributes)]
    #[props(extends = a)]
    attributes: Vec<Attribute>,
) -> Element {
    // `PaginationLink`'s base `w-9` fits a single-character page number, but
    // this preset's label ("Previous" by default) needs real width -- left
    // as a fixed square, the label overflowed the box and visually collided
    // with the next `PaginationItem` (found live: rendered as "Previous1").
    // `compact` legitimately wants the square (icon only), so only override
    // width when the label is showing.
    let width_class = if compact { "" } else { "w-auto" };
    let class = cn(&[
        "gap-1 pl-2.5 pr-3",
        width_class,
        class.as_deref().unwrap_or_default(),
    ]);
    let text = text.unwrap_or_else(|| "Previous".to_string());
    rsx! {
        PaginationLink {
            class,
            href,
            onclick: move |event| onclick.call(event),
            aria_label: "Go to previous page",
            attributes,
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
    /// Native anchor and global Dioxus attributes, forwarded to the
    /// underlying [`PaginationLink`].
    #[props(extends = GlobalAttributes)]
    #[props(extends = a)]
    attributes: Vec<Attribute>,
) -> Element {
    // See `PaginationPrevious`'s own comment for why this overrides width.
    let width_class = if compact { "" } else { "w-auto" };
    let class = cn(&[
        "gap-1 pl-3 pr-2.5",
        width_class,
        class.as_deref().unwrap_or_default(),
    ]);
    let text = text.unwrap_or_else(|| "Next".to_string());
    rsx! {
        PaginationLink {
            class,
            href,
            onclick: move |event| onclick.call(event),
            aria_label: "Go to next page",
            attributes,
            if !compact { span { "{text}" } }
            span { "aria-hidden": "true", "›" }
        }
    }
}

/// A non-interactive marker for skipped pages between [`PaginationLink`]s.
#[component]
pub fn PaginationEllipsis(
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex h-9 w-9 items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span {
            class,
            "aria-hidden": "true",
            ..attributes,
            "…"
            span { class: "sr-only", "More pages" }
        }
    }
}
