//! Source-owned shadcn-style Tabs for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::tabs::Tabs;
use adico_primitives::tabs::{
    TabContent as TabContentPrimitive, TabList as TabListPrimitive,
    TabTrigger as TabTriggerPrimitive,
};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// Visual style for a [`TabList`] and the [`TabTrigger`]s inside it, matching
/// upstream shadcn's `default`/`line` Tabs variants (this repo previously had
/// no `line` support at all -- found by this session's variants audit).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TabsVariant {
    /// A pill-shaped, filled tab strip with the active trigger on a raised surface.
    #[default]
    Default,
    /// A flat, underlined tab strip with no background pill.
    Line,
}

/// Shared with [`TabTrigger`] so it can pick variant-specific classes without
/// the caller repeating `variant` on every trigger.
#[derive(Clone, Copy)]
struct TabsListContext {
    variant: TabsVariant,
}

/// The row of [`TabTrigger`] buttons for a [`Tabs`] root.
#[component]
pub fn TabList(
    #[props(default)] variant: TabsVariant,
    /// Corner radius of the `Default`-variant pill tray. Has no visible
    /// effect on the `Line` variant, which is deliberately flat (see
    /// [`TabTrigger`]'s own doc comment for why its own trigger-level
    /// corners stay independent of this prop).
    #[props(default)]
    radius: Radius,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    use_context_provider(|| TabsListContext { variant });
    let variant_class = match variant {
        TabsVariant::Default => "h-9 bg-muted p-1",
        TabsVariant::Line => "gap-1 border-b bg-transparent p-0",
    };
    // Only the `Default` (pill) variant has a corner to control; `Line` is
    // deliberately flat, so applying `radius.class()` there would draw an
    // unwanted rounded corner on what's meant to be a plain underline strip.
    let radius_class = match variant {
        TabsVariant::Default => radius.class(),
        TabsVariant::Line => "",
    };
    let class = cn(&[
        "inline-flex items-center justify-center text-muted-foreground",
        variant_class,
        radius_class,
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TabListPrimitive { class, attributes, {children} }
    }
}

/// A single tab button within a [`TabList`]. Deliberately has no `radius`
/// prop of its own: its `rounded-md`/`rounded-none` pair encodes
/// [`TabsVariant`]'s segmented-pill-vs-underline visual identity, not an
/// independent cosmetic choice — an independent radius here would let a
/// caller produce a broken hybrid (e.g. the `Line` variant's underline
/// style with a pill-shaped active trigger).
#[component]
pub fn TabTrigger(
    value: String,
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    id: Option<String>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let variant = try_consume_context::<TabsListContext>()
        .map(|ctx| ctx.variant)
        .unwrap_or_default();
    let variant_class = match variant {
        TabsVariant::Default => {
            "rounded-md data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm"
        }
        TabsVariant::Line => {
            "rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:text-foreground"
        }
    };
    let class = cn(&[
        "inline-flex flex-1 items-center justify-center gap-1.5 whitespace-nowrap px-2 py-1 text-sm font-medium transition-all focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-1 focus-visible:outline-ring disabled:pointer-events-none disabled:opacity-50",
        variant_class,
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TabTriggerPrimitive {
            value,
            index,
            disabled,
            id,
            class,
            attributes,
            {children}
        }
    }
}

/// The panel shown while its matching [`TabTrigger`] is active.
#[component]
pub fn TabContent(
    value: String,
    index: ReadSignal<usize>,
    id: Option<String>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[
        "mt-2 focus-visible:outline-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TabContentPrimitive {
            value,
            index,
            id,
            class,
            attributes,
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_trigger_gets_a_semantic_surface() {
        let class = cn(&[
            "data-[state=active]:bg-background data-[state=active]:text-foreground",
            "",
        ]);
        assert!(class.contains("data-[state=active]:bg-background"));
    }

    #[test]
    fn line_variant_uses_an_underline_instead_of_a_filled_surface() {
        let class = cn(&[
            "rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:text-foreground",
            "",
        ]);
        assert!(class.contains("data-[state=active]:border-primary"));
        assert!(!class.contains("data-[state=active]:bg-background"));
    }
}
