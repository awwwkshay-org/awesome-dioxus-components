//! Source-owned shadcn-style Collapsible for Dioxus, backed by the owned
//! adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::collapsible::Collapsible;
use adico_primitives::collapsible::{
    CollapsibleContent as PrimitiveCollapsibleContent,
    CollapsibleTrigger as PrimitiveCollapsibleTrigger,
};
use adico_primitives::icons::ChevronDown;

use crate::adico_lib::cn::cn;

/// A trigger button with a chevron affordance that flips based on the
/// primitive's own `data-open` attribute.
#[component]
pub fn CollapsibleTrigger(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex w-full items-center justify-between gap-4 rounded-md border border-input bg-background px-3 py-2 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 [&[data-open=true]>svg]:rotate-180",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCollapsibleTrigger { class, attributes,
            {children}
            ChevronDown { class: "size-4 shrink-0 text-muted-foreground transition-transform duration-200" }
        }
    }
}

/// The collapsible panel, indented and top-padded to read as nested content
/// beneath its trigger.
#[component]
pub fn CollapsibleContent(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "mt-2 space-y-2 text-sm",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCollapsibleContent { id: ReadSignal::default(), class, attributes, {children} }
    }
}
