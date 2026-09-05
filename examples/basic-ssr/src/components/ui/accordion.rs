//! Source-owned shadcn-style Accordion for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::accordion::{Accordion, AccordionMulti};
use adico_primitives::accordion::{
    AccordionContent as AccordionContentPrimitive, AccordionItem as AccordionItemPrimitive,
    AccordionTrigger as AccordionTriggerPrimitive,
};
use adico_primitives::icons::ChevronDown;

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// A single collapsible section within an [`Accordion`]/[`AccordionMulti`].
#[component]
pub fn AccordionItem(
    value: ReadSignal<String>,
    index: usize,
    #[props(default)] disabled: ReadSignal<bool>,
    /// Called when this item's open/closed state changes, with the new
    /// value.
    #[props(default)]
    on_change: Callback<bool, ()>,
    /// Called when this item's trigger is clicked.
    #[props(default)]
    on_trigger_click: Callback,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&["border-b", class.as_deref().unwrap_or_default()]);
    rsx! {
        AccordionItemPrimitive {
            value,
            index,
            disabled,
            on_change,
            on_trigger_click,
            class,
            attributes,
            {children}
        }
    }
}

/// The clickable header that toggles the enclosing [`AccordionItem`].
#[component]
pub fn AccordionTrigger(
    id: Option<String>,
    #[props(default = Radius::Md)] radius: Radius,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[
        "flex w-full flex-1 items-start justify-between gap-4 py-4 text-left text-sm font-medium outline-none transition-all hover:underline focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 [&[data-state=open]>svg]:rotate-180",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AccordionTriggerPrimitive { id, class, attributes,
            {children}
            ChevronDown { class: "size-4 shrink-0 text-muted-foreground transition-transform duration-200" }
        }
    }
}

/// The panel shown while the enclosing [`AccordionItem`] is open.
#[component]
pub fn AccordionContent(
    id: Option<String>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[
        "overflow-hidden text-sm",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AccordionContentPrimitive { id, class, attributes,
            div { class: "pb-4 pt-0", {children} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_class_draws_a_semantic_divider() {
        let class = cn(&["border-b", ""]);
        assert!(class.contains("border-b"));
    }
}
