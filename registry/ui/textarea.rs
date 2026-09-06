//! Source-owned shadcn-style Textarea for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

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
    /// Maximum number of characters the native textarea accepts.
    #[props(default)]
    pub max_length: Option<u32>,
    /// Input event handler.
    #[props(default)]
    pub oninput: EventHandler<FormEvent>,
    /// Corner radius of the textarea surface.
    #[props(default = Radius::Md)]
    pub radius: Radius,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native textarea/global attributes and events.
    #[props(extends = GlobalAttributes)]
    #[props(extends = textarea)]
    pub attributes: Vec<Attribute>,
}

/// The textarea's base classes, shared by both render branches so the
/// bare and counter-wrapped forms stay visually identical.
const TEXTAREA_BASE_CLASS: &str = "flex min-h-[60px] w-full border border-input bg-transparent px-3 py-2 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 read-only:cursor-default read-only:bg-muted aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40";

/// A styled multi-line text input with the default adico/shadcn visual language.
///
/// When `max_length` is set, a live `N / max` character counter renders in
/// the field's bottom-right corner, which requires wrapping the textarea in
/// a `relative` `<div>` — so a consumer styling by element position gains
/// one wrapper level in that mode. Without `max_length` the rendered output
/// is the bare `<textarea>`, exactly as before the counter existed.
#[component]
pub fn Textarea(props: TextareaProps) -> Element {
    // Tracks what the user has typed for the uncontrolled case; a
    // controlled `value` takes precedence below so the counter can never
    // drift from it.
    let mut typed_count = use_signal(|| {
        props
            .value
            .as_deref()
            .map(|v| v.chars().count())
            .unwrap_or(0)
    });
    let shown_count = props
        .value
        .as_deref()
        .map(|v| v.chars().count())
        .unwrap_or_else(|| typed_count());

    let class = cn(&[
        TEXTAREA_BASE_CLASS,
        // Reserve room under the text so the last line never underlaps the
        // counter; only present in the wrapped branch.
        if props.max_length.is_some() {
            "pb-6"
        } else {
            ""
        },
        props.radius.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    let value = props.value.clone();
    let placeholder = props.placeholder.clone();
    let attributes = props.attributes.clone();
    let oninput = move |event: FormEvent| {
        typed_count.set(event.value().chars().count());
        props.oninput.call(event);
    };
    // The native `value` attribute key is present only when this field is
    // controlled: dioxus-web's forms handling calls the DOM `value` setter
    // on every render that declares this attribute, controlled or not. The
    // `typed_count` signal above makes this component re-render on every
    // keystroke (to keep the counter live), so an unconditionally-present
    // `value: None` would reassert an empty value on every character typed,
    // wiping it immediately -- found live as "can't type in the textarea"
    // once `max_length` was set. Uncontrolled usage must omit the attribute
    // entirely and let the browser own the field's value.
    let textarea = match value {
        Some(value) => rsx! {
            textarea {
                class,
                value,
                placeholder,
                disabled: props.disabled,
                readonly: props.readonly,
                required: props.required,
                rows: props.rows,
                maxlength: props.max_length,
                aria_invalid: props.invalid,
                oninput,
                ..attributes,
            }
        },
        None => rsx! {
            textarea {
                class,
                placeholder,
                disabled: props.disabled,
                readonly: props.readonly,
                required: props.required,
                rows: props.rows,
                maxlength: props.max_length,
                aria_invalid: props.invalid,
                oninput,
                ..attributes,
            }
        },
    };
    match props.max_length {
        Some(max) => rsx! {
            div { class: "relative w-full",
                {textarea}
                span { class: "pointer-events-none absolute bottom-1.5 right-2.5 text-xs text-muted-foreground",
                    "{shown_count} / {max}"
                }
            }
        },
        None => textarea,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_without_max_length_is_unchanged_from_the_pre_counter_output() {
        // The no-counter branch must stay byte-identical to what this
        // component rendered before the counter existed: `cn` drops the
        // empty padding fragment entirely.
        let class = cn(&[TEXTAREA_BASE_CLASS, "", Radius::Md.class(), ""]);
        let pre_counter = cn(&[TEXTAREA_BASE_CLASS, Radius::Md.class(), ""]);
        assert_eq!(class, pre_counter);
        assert!(!class.contains("pb-6"));
    }

    #[test]
    fn class_with_max_length_reserves_counter_padding() {
        let class = cn(&[TEXTAREA_BASE_CLASS, "pb-6", Radius::Md.class(), ""]);
        assert!(class.contains("pb-6"));
        assert!(class.contains("min-h-[60px]"));
    }
}
