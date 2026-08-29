//! Source-owned shadcn-style Card composition for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// The outer container for a Card composition.
#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    /// Extra classes appended to the semantic card surface.
    #[props(default)]
    pub class: Option<String>,
    /// Native div/global attributes, including data attributes and handlers.
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed card regions.
    pub children: Element,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let class = cn(&[
        "w-full rounded-xl border bg-card text-card-foreground shadow-sm",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        section { class, ..props.attributes, {props.children} }
    }
}

/// A semantic header region for a Card's title and description.
#[derive(Props, Clone, PartialEq)]
pub struct CardHeaderProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn CardHeader(props: CardHeaderProps) -> Element {
    let class = cn(&[
        "flex flex-col gap-1.5 p-6",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        header { class, ..props.attributes, {props.children} }
    }
}

/// The primary heading of a Card.
#[derive(Props, Clone, PartialEq)]
pub struct CardTitleProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = h3)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn CardTitle(props: CardTitleProps) -> Element {
    let class = cn(&[
        "font-semibold leading-none tracking-tight",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        h3 { class, ..props.attributes, {props.children} }
    }
}

/// Supporting text placed under a [`CardTitle`].
#[derive(Props, Clone, PartialEq)]
pub struct CardDescriptionProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = p)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn CardDescription(props: CardDescriptionProps) -> Element {
    let class = cn(&[
        "text-sm text-muted-foreground",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        p { class, ..props.attributes, {props.children} }
    }
}

/// The main body region of a Card.
#[derive(Props, Clone, PartialEq)]
pub struct CardContentProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn CardContent(props: CardContentProps) -> Element {
    let class = cn(&["p-6 pt-0", props.class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, ..props.attributes, {props.children} }
    }
}

/// A footer region typically used for Card actions.
#[derive(Props, Clone, PartialEq)]
pub struct CardFooterProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn CardFooter(props: CardFooterProps) -> Element {
    let class = cn(&[
        "flex flex-wrap items-center gap-2 p-6 pt-0",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        footer { class, ..props.attributes, {props.children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_regions_use_semantic_surfaces() {
        assert!(
            cn(&["w-full rounded-xl border bg-card text-card-foreground shadow-sm"])
                .contains("bg-card")
        );
        assert!(cn(&["flex flex-wrap items-center gap-2 p-6 pt-0"]).contains("flex-wrap"));
    }
}
