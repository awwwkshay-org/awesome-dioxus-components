//! Source-owned shadcn-style Card composition for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// The outer container for a Card composition.
#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    /// Corner radius of the card surface.
    #[props(default = Radius::Xl)]
    pub radius: Radius,
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
        // `@container`: `CardHeader`'s two-column-with-action layout responds
        // to the *card's own* rendered width, not the viewport (see
        // `CardHeader`'s doc comment for why `sm:` doesn't work here).
        "@container w-full border bg-card text-card-foreground shadow-sm",
        props.radius.class(),
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
        // The `has-[[data-slot=card-action]]:*` compound moves behind `@sm:`
        // (R6, container-query form): below a 384px-wide *card* (not
        // viewport), it stays `flex flex-col` (its own unconditional base)
        // -- title, description, and the action stack in document order --
        // rather than forcing a cramped two-column grid. At `@sm` and up,
        // this restores today's exact `grid grid-cols-[1fr_auto]` corner
        // layout. A viewport `sm:` breakpoint was tried first and rejected:
        // Tailwind v4.1.5 fails to compile `sm:has-[[data-slot=...]]:*`
        // entirely (verified: 0 occurrences in the compiled stylesheet, vs.
        // 6 for the unprefixed form) -- stacking a breakpoint variant in
        // front of a `has-[[...]]` double-bracket selector breaks its
        // parser. Container queries also don't have this problem AND are
        // more correct for a reusable `Card`: its layout should respond to
        // its own rendered width, not the viewport, since cards are commonly
        // embedded in narrow columns even on a wide desktop screen.
        "flex flex-col gap-1.5 p-6 @sm:has-[[data-slot=card-action]]:grid @sm:has-[[data-slot=card-action]]:grid-cols-[1fr_auto] @sm:has-[[data-slot=card-action]]:items-start",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        header { class, ..props.attributes, {props.children} }
    }
}

/// An action (for example a button or menu trigger) placed at the end of a
/// [`CardHeader`], alongside its title/description. Upstream shadcn's
/// `CardAction` was missing here entirely -- a real composition gap, not a
/// styling one: without it, a header action has nowhere to attach without
/// the caller inventing their own layout. `CardHeader` only switches to a
/// two-column grid (via the `has-[[data-slot=card-action]]:*` classes above)
/// when a `CardAction` is actually present, so every existing `CardHeader`
/// with no action renders exactly as before.
#[derive(Props, Clone, PartialEq)]
pub struct CardActionProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn CardAction(props: CardActionProps) -> Element {
    let class = cn(&[
        "col-start-2 row-span-2 row-start-1 self-start justify-self-end",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-slot": "card-action", ..props.attributes, {props.children} }
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
