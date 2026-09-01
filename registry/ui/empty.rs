//! Source-owned shadcn-style Empty state for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// The presentation of an [`EmptyMedia`] slot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EmptyMediaVariant {
    /// Bare media (an illustration or raw icon), no surrounding chrome.
    #[default]
    Default,
    /// A muted rounded-square icon surface.
    Icon,
}

impl EmptyMediaVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "bg-transparent",
            Self::Icon => {
                "flex size-10 shrink-0 items-center justify-center rounded-lg bg-muted text-foreground [&_svg:not([class*='size-'])]:size-6"
            }
        }
    }
}

/// A dashed-border placeholder region for empty/zero-data states.
#[component]
pub fn Empty(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "flex min-w-0 flex-1 flex-col items-center justify-center gap-6 rounded-lg border-dashed p-6 text-center text-balance md:p-12",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// Groups an [`Empty`]'s media, title, and description.
#[component]
pub fn EmptyHeader(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "flex max-w-sm flex-col items-center gap-2 text-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// Props for [`EmptyMedia`].
#[derive(Props, Clone, PartialEq)]
pub struct EmptyMediaProps {
    /// Whether to render bare media or a muted icon surface.
    #[props(default)]
    pub variant: EmptyMediaVariant,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native div/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    /// An icon or illustration.
    pub children: Element,
}

/// An [`Empty`] state's leading icon or illustration slot.
#[component]
pub fn EmptyMedia(props: EmptyMediaProps) -> Element {
    let class = cn(&[
        "mb-2 flex shrink-0 items-center justify-center [&_svg]:pointer-events-none [&_svg]:shrink-0",
        props.variant.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, ..props.attributes, {props.children} }
    }
}

/// An [`Empty`] state's heading.
#[component]
pub fn EmptyTitle(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "text-lg font-medium tracking-tight",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// An [`Empty`] state's supporting body copy.
#[component]
pub fn EmptyDescription(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "text-sm/relaxed text-muted-foreground [&>a]:underline [&>a]:underline-offset-4 [&>a:hover]:text-primary",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// An [`Empty`] state's trailing action area (typically buttons).
#[component]
pub fn EmptyContent(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "flex w-full max-w-sm min-w-0 flex-col items-center gap-4 text-sm text-balance",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_media_variant_has_intentional_chrome() {
        assert_eq!(EmptyMediaVariant::Default.class(), "bg-transparent");
        assert!(EmptyMediaVariant::Icon.class().contains("bg-muted"));
    }
}
