//! Source-owned shadcn-style Input for Dioxus.

use adico_primitives::icons::{Eye, EyeOff};
use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// Props for [`Input`].
#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    /// Native input type. `"password"` renders a built-in reveal toggle
    /// alongside the field; every other value renders exactly as before.
    #[props(default = "text".to_string())]
    pub r#type: String,
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
    /// Input event handler.
    #[props(default)]
    pub oninput: EventHandler<FormEvent>,
    /// Corner radius of the input surface.
    #[props(default = Radius::Md)]
    pub radius: Radius,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native input/global attributes and events.
    #[props(extends = GlobalAttributes)]
    #[props(extends = input)]
    pub attributes: Vec<Attribute>,
}

/// A styled single-line text input with the default adico/shadcn visual
/// language. When `r#type` is `"password"`, renders a built-in eye/eye-off
/// reveal toggle rather than requiring the consumer to hand-compose one from
/// `InputGroup` -- the value and `oninput` callback behave identically
/// either way; the toggle only ever changes the field's rendered `type`
/// between `"password"` and `"text"`, never the value itself.
#[component]
pub fn Input(props: InputProps) -> Element {
    let is_password = props.r#type == "password";
    let mut reveal = use_signal(|| false);
    let effective_type = if is_password && reveal() {
        "text".to_string()
    } else {
        props.r#type.clone()
    };

    let class = cn(&[
        "flex h-9 w-full border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 read-only:cursor-default read-only:bg-muted aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40",
        props.radius.class(),
        if is_password { "pr-9!" } else { "" },
        props.class.as_deref().unwrap_or_default(),
    ]);

    let field = rsx! {
        input {
            class,
            r#type: effective_type,
            value: props.value,
            placeholder: props.placeholder,
            disabled: props.disabled,
            readonly: props.readonly,
            required: props.required,
            aria_invalid: props.invalid,
            oninput: move |event| props.oninput.call(event),
            ..props.attributes,
        }
    };

    if is_password {
        rsx! {
            div { class: "relative",
                {field}
                button {
                    r#type: "button",
                    class: "absolute inset-y-0 right-0 flex items-center px-2.5 text-muted-foreground hover:text-foreground focus-visible:outline-none",
                    onclick: move |_| reveal.toggle(),
                    if reveal() {
                        EyeOff { class: "size-4" }
                    } else {
                        Eye { class: "size-4" }
                    }
                    span { class: "sr-only", if reveal() { "Hide password" } else { "Show password" } }
                }
            }
        }
    } else {
        field
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_reveal_reserves_space_without_relying_on_class_order() {
        // `cn` is a plain space-join, not a tailwind-merge conflict
        // resolver -- `pr-9!` (an important override) is required here so
        // it deterministically wins over the base class's `px-3`,
        // regardless of Tailwind's compiled utility order.
        let class = cn(&["px-3", "pr-9!", ""]);
        assert!(class.contains("pr-9!"));
    }
}
