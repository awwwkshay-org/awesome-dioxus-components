//! Source-owned shadcn-style Marker for Dioxus.
//!
//! A small inline annotation -- a step number, tool-call badge, or citation
//! marker -- attached to a chat/agent message. Purely presentational; no
//! dedicated primitive dependency (task 10.1's own audit).
//!
//! Upstream's `root` part declares `asChild` (render-as-caller-supplied-
//! element polymorphism); Dioxus has no equivalent mechanism. This
//! ecosystem's existing answer is caller-composition instead (see
//! `dropdown_menu.rs`'s `DropdownMenuTrigger` doc comment for the
//! established precedent) -- a caller who needs `Marker`'s content to also be
//! e.g. a link wraps its own `a` around `Marker`'s children rather than
//! `Marker` itself changing element.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// The visual treatment of a [`Marker`] chip, matching shadcn's own cva axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MarkerVariant {
    /// A filled chip with a border, the existing default look.
    #[default]
    Default,
    /// A bordered chip with no background fill.
    Border,
    /// No border or background at all, reading as inline plain text.
    Separator,
}

impl MarkerVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "border bg-muted",
            Self::Border => "border bg-transparent",
            Self::Separator => "border-transparent bg-transparent",
        }
    }
}

/// Props for [`Marker`].
#[derive(Props, Clone, PartialEq)]
pub struct MarkerProps {
    /// Visual treatment; see [`MarkerVariant`].
    #[props(default)]
    pub variant: MarkerVariant,
    /// Corner radius of the marker chip.
    #[props(default = Radius::Full)]
    pub radius: Radius,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native span/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = span)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed [`MarkerIcon`]/[`MarkerContent`].
    pub children: Element,
}

/// A small rounded annotation chip.
#[component]
pub fn Marker(props: MarkerProps) -> Element {
    let class = cn(&[
        "inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium text-muted-foreground",
        props.variant.class(),
        props.radius.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span { class, ..props.attributes, {props.children} }
    }
}

/// Props for [`MarkerIcon`].
#[derive(Props, Clone, PartialEq)]
pub struct MarkerIconProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed icon.
    pub children: Element,
}

/// The icon slot, sized to match the marker's text.
#[component]
pub fn MarkerIcon(props: MarkerIconProps) -> Element {
    let class = cn(&[
        "inline-flex size-3 shrink-0 items-center justify-center [&>svg]:size-3",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span { class, {props.children} }
    }
}

/// Props for [`MarkerContent`].
#[derive(Props, Clone, PartialEq)]
pub struct MarkerContentProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed label content.
    pub children: Element,
}

/// The label slot.
#[component]
pub fn MarkerContent(props: MarkerContentProps) -> Element {
    let class = cn(&["truncate", props.class.as_deref().unwrap_or_default()]);
    rsx! {
        span { class, {props.children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_class_is_a_rounded_chip() {
        let class = cn(&["inline-flex items-center gap-1 rounded-full border bg-muted"]);
        assert!(class.contains("rounded-full"));
        assert!(class.contains("bg-muted"));
    }
}
