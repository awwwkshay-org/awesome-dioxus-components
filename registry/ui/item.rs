//! Source-owned shadcn-style Item composition for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// A vertical list of [`Item`] rows.
#[derive(Props, Clone, PartialEq)]
pub struct ItemGroupProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ItemGroup(props: ItemGroupProps) -> Element {
    let class = cn(&["flex flex-col", props.class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, role: "list", ..props.attributes, {props.children} }
    }
}

/// Visual and interaction treatment for an [`Item`] row.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ItemVariant {
    /// A bordered row suitable for independently actionable content.
    #[default]
    Default,
    /// A quiet row with no outer border.
    Muted,
    /// A row that advertises pointer and keyboard focus affordances.
    Interactive,
}

impl ItemVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "border bg-card",
            Self::Muted => "border-transparent bg-muted/50",
            Self::Interactive => "border bg-card cursor-pointer hover:bg-accent hover:text-accent-foreground",
        }
    }
}

/// A single row combining optional media, content, and actions.
#[derive(Props, Clone, PartialEq)]
pub struct ItemProps {
    #[props(default)]
    pub variant: ItemVariant,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn Item(props: ItemProps) -> Element {
    let class = cn(&[
        "flex items-center gap-4 rounded-md p-4 text-sm outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
        props.variant.class(),
        if props.disabled { "pointer-events-none opacity-50" } else { "" },
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div {
            class,
            role: "listitem",
            tabindex: if props.variant == ItemVariant::Interactive && !props.disabled { 0 } else { -1 },
            "aria-disabled": props.disabled,
            onclick: move |event| if !props.disabled { props.onclick.call(event) },
            ..props.attributes,
            {props.children}
        }
    }
}

/// A leading icon, avatar, or image slot for an [`Item`].
#[derive(Props, Clone, PartialEq)]
pub struct ItemMediaProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ItemMedia(props: ItemMediaProps) -> Element {
    let class = cn(&[
        "flex shrink-0 items-center justify-center",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, ..props.attributes, {props.children} }
    }
}

/// The flexible text region of an [`Item`], holding title/description.
#[derive(Props, Clone, PartialEq)]
pub struct ItemContentProps {
    #[props(default)] pub class: Option<String>,
    #[props(extends = GlobalAttributes)] #[props(extends = div)] pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ItemContent(props: ItemContentProps) -> Element {
    let class = cn(&[
        "flex flex-1 flex-col gap-1",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, ..props.attributes, {props.children} }
    }
}

/// The primary label of an [`Item`].
#[derive(Props, Clone, PartialEq)]
pub struct ItemTitleProps {
    #[props(default)] pub class: Option<String>,
    #[props(extends = GlobalAttributes)] #[props(extends = div)] pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ItemTitle(props: ItemTitleProps) -> Element {
    let class = cn(&[
        "text-sm font-medium leading-none",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, ..props.attributes, {props.children} }
    }
}

/// Supporting text placed under an [`ItemTitle`].
#[derive(Props, Clone, PartialEq)]
pub struct ItemDescriptionProps {
    #[props(default)] pub class: Option<String>,
    #[props(extends = GlobalAttributes)] #[props(extends = div)] pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ItemDescription(props: ItemDescriptionProps) -> Element {
    let class = cn(&[
        "text-sm text-muted-foreground",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, ..props.attributes, {props.children} }
    }
}

/// A trailing slot for buttons or other controls on an [`Item`].
#[derive(Props, Clone, PartialEq)]
pub struct ItemActionsProps {
    #[props(default)] pub class: Option<String>,
    #[props(extends = GlobalAttributes)] #[props(extends = div)] pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ItemActions(props: ItemActionsProps) -> Element {
    let class = cn(&[
        "flex items-center gap-2",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, ..props.attributes, {props.children} }
    }
}

/// A full-width row above an [`Item`]'s main content, e.g. for grouped headers.
#[derive(Props, Clone, PartialEq)]
pub struct ItemHeaderProps {
    #[props(default)] pub class: Option<String>,
    #[props(extends = GlobalAttributes)] #[props(extends = div)] pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ItemHeader(props: ItemHeaderProps) -> Element {
    let class = cn(&[
        "flex basis-full items-center justify-between gap-2",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, ..props.attributes, {props.children} }
    }
}

/// A full-width row below an [`Item`]'s main content.
#[derive(Props, Clone, PartialEq)]
pub struct ItemFooterProps {
    #[props(default)] pub class: Option<String>,
    #[props(extends = GlobalAttributes)] #[props(extends = div)] pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn ItemFooter(props: ItemFooterProps) -> Element {
    let class = cn(&[
        "flex basis-full items-center justify-between gap-2",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, ..props.attributes, {props.children} }
    }
}

/// A thin horizontal rule between items in an [`ItemGroup`].
#[derive(Props, Clone, PartialEq)]
pub struct ItemSeparatorProps {
    #[props(default)] pub class: Option<String>,
    #[props(extends = GlobalAttributes)] #[props(extends = hr)] pub attributes: Vec<Attribute>,
}

#[component]
pub fn ItemSeparator(props: ItemSeparatorProps) -> Element {
    let class = cn(&["my-0 border-t", props.class.as_deref().unwrap_or_default()]);
    rsx! {
        hr { class, role: "separator", ..props.attributes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_variants_have_distinct_states() {
        assert!(ItemVariant::Default.class().contains("border"));
        assert!(ItemVariant::Muted.class().contains("bg-muted"));
        assert!(ItemVariant::Interactive.class().contains("cursor-pointer"));
    }
}
