//! Source-owned shadcn-style Accordion for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::accordion::Accordion;
use adico_primitives::accordion::{
    AccordionContent as AccordionContentPrimitive, AccordionItem as AccordionItemPrimitive,
    AccordionTrigger as AccordionTriggerPrimitive,
};
use adico_primitives::icons::ChevronDown;

use crate::adico_lib::cn::cn;

/// A single collapsible section within an [`Accordion`].
#[component]
pub fn AccordionItem(
    index: usize,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] default_open: bool,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&["border-b", class.as_deref().unwrap_or_default()]);
    rsx! {
        AccordionItemPrimitive { index, disabled, default_open, class, {children} }
    }
}

/// The clickable header that toggles the enclosing [`AccordionItem`].
#[component]
pub fn AccordionTrigger(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "flex flex-1 items-center justify-between gap-4 py-4 text-sm font-medium transition-all hover:underline [&[data-state=open]>svg]:rotate-180",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AccordionTriggerPrimitive { class,
            {children}
            ChevronDown { class: "size-4 shrink-0 text-muted-foreground transition-transform duration-200" }
        }
    }
}

/// The panel shown while the enclosing [`AccordionItem`] is open.
#[component]
pub fn AccordionContent(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "overflow-hidden text-sm",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        AccordionContentPrimitive { class,
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
