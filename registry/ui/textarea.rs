//! Source-owned shadcn-style Textarea for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// Props for [`Textarea`].
#[derive(Props, Clone, PartialEq)]
pub struct TextareaProps {
    /// Controlled text value.
    #[props(default)]
    pub value: Option<String>,
    /// Hint shown when the value is empty.
    #[props(default)]
    pub placeholder: Option<String>,
    /// Disables interaction and exposes native disabled semantics.
    #[props(default)]
    pub disabled: Option<bool>,
    /// Prevents edits while allowing focus and selection.
    #[props(default)]
    pub readonly: Option<bool>,
    /// Marks this field as required for native form validation.
    #[props(default)]
    pub required: Option<bool>,
    /// Applies the semantic invalid presentation alongside native `aria-invalid`.
    #[props(default)]
    pub invalid: bool,
    /// Initial visible text rows.
    #[props(default)]
    pub rows: Option<u32>,
    /// Input event handler.
    #[props(default)]
    pub oninput: EventHandler<FormEvent>,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native textarea/global attributes and events.
    #[props(extends = GlobalAttributes)]
    #[props(extends = textarea)]
    pub attributes: Vec<Attribute>,
}

/// A styled multi-line text input with the default adico/shadcn visual language.
#[component]
pub fn Textarea(props: TextareaProps) -> Element {
    let class = cn(&[
        "flex min-h-[60px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 read-only:cursor-default read-only:bg-muted aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        textarea {
            class,
            value: props.value,
            placeholder: props.placeholder,
            disabled: props.disabled,
            readonly: props.readonly,
            required: props.required,
            rows: props.rows,
            aria_invalid: props.invalid,
            oninput: move |event| props.oninput.call(event),
            ..props.attributes,
        }
    }
}
