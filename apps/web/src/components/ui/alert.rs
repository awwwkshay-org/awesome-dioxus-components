//! Source-owned shadcn-style Alert for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// The semantic presentation of an [`Alert`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AlertVariant {
    /// Neutral informational alert.
    #[default]
    Default,
    /// Destructive or error alert.
    Destructive,
    /// Positive/success alert.
    Success,
    /// Cautionary alert.
    Warning,
    /// Informational alert.
    Info,
}

impl AlertVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "bg-card text-card-foreground",
            Self::Destructive => "bg-card text-destructive [&>svg]:text-current",
            Self::Success => "bg-card text-success [&>svg]:text-current",
            Self::Warning => "bg-card text-warning [&>svg]:text-current",
            Self::Info => "bg-card text-info [&>svg]:text-current",
        }
    }
}

/// Props for [`Alert`].
#[derive(Props, Clone, PartialEq)]
pub struct AlertProps {
    /// Semantic presentation variant.
    #[props(default)]
    pub variant: AlertVariant,
    /// Corner radius of the alert surface.
    #[props(default)]
    pub radius: Radius,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native div/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed alert content, typically an icon plus
    /// [`AlertTitle`]/[`AlertDescription`].
    pub children: Element,
}

/// A short, non-modal message calling attention to information, backed by a
/// native `role="alert"` region.
#[component]
pub fn Alert(props: AlertProps) -> Element {
    let class = cn(&[
        "relative grid w-full grid-cols-[0_1fr] items-start gap-y-0.5 border px-4 py-3 text-sm has-[>svg]:grid-cols-[calc(var(--spacing)*4)_1fr] has-[>svg]:gap-x-3 [&>svg]:size-4 [&>svg]:translate-y-0.5 [&>svg]:text-current",
        props.variant.class(),
        props.radius.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, role: "alert", ..props.attributes, {props.children} }
    }
}

/// The bold, single-line heading of an [`Alert`].
#[component]
pub fn AlertTitle(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "col-start-2 line-clamp-1 min-h-4 font-medium tracking-tight",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// The supporting body copy of an [`Alert`].
#[component]
pub fn AlertDescription(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "col-start-2 grid justify-items-start gap-1 text-sm text-muted-foreground",
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
    fn every_variant_uses_a_semantic_surface() {
        assert!(AlertVariant::Default.class().contains("bg-card"));
        assert!(
            AlertVariant::Destructive
                .class()
                .contains("text-destructive")
        );
    }

    #[test]
    fn every_new_tone_variant_uses_its_matching_theme_token() {
        assert!(AlertVariant::Success.class().contains("text-success"));
        assert!(AlertVariant::Warning.class().contains("text-warning"));
        assert!(AlertVariant::Info.class().contains("text-info"));
    }
}
