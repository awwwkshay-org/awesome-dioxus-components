//! Source-owned shadcn-style Textarea for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// A styled multi-line text input with the default adico/shadcn visual language.
#[component]
pub fn Textarea(
    value: Option<String>,
    placeholder: Option<String>,
    disabled: Option<bool>,
    #[props(default)] oninput: EventHandler<FormEvent>,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "flex min-h-[60px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        textarea {
            class,
            value,
            placeholder,
            disabled,
            oninput: move |event| oninput.call(event),
        }
    }
}
