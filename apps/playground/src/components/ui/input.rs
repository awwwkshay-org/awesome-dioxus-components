//! Source-owned shadcn-style Input for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// A styled single-line text input with the default adico/shadcn visual language.
#[component]
pub fn Input(
    #[props(default = "text".to_string())] r#type: String,
    value: Option<String>,
    placeholder: Option<String>,
    disabled: Option<bool>,
    #[props(default)] oninput: EventHandler<FormEvent>,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        input {
            class,
            r#type,
            value,
            placeholder,
            disabled,
            oninput: move |event| oninput.call(event),
        }
    }
}
