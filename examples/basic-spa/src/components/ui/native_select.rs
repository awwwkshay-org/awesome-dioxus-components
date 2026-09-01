//! Source-owned shadcn-style NativeSelect for Dioxus.
//!
//! Wraps a native `<select>` for OS-native picker behavior (mobile wheel
//! pickers, native keyboard search-by-typing), unlike the owned
//! [`super::select::Select`] custom-listbox component. `NativeSelectOption`
//! and `NativeSelectOptGroup` deliberately use the CSS system-color keywords
//! `Canvas`/`CanvasText` (matching upstream exactly) rather than adico's
//! `background`/`foreground` semantic tokens: a native `<option>` popup is
//! painted by the OS outside the page's own CSS custom properties, so only
//! system colors (which the OS itself resolves per-theme) keep option text
//! legible in that popup -- this is not a token-compliance gap.

use dioxus::prelude::*;

use adico_primitives::icons::ChevronDown;

use crate::adico_lib::cn::cn;

/// The visual size for a [`NativeSelect`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NativeSelectSize {
    /// Standard control height.
    #[default]
    Default,
    /// Compact control height.
    Sm,
}

impl NativeSelectSize {
    fn class(self) -> &'static str {
        match self {
            Self::Default => "h-9 py-2",
            Self::Sm => "h-8 py-1",
        }
    }
}

/// Props for [`NativeSelect`].
#[derive(Props, Clone, PartialEq)]
pub struct NativeSelectProps {
    /// Visual size.
    #[props(default)]
    pub size: NativeSelectSize,
    /// Controlled selected value.
    #[props(default)]
    pub value: Option<String>,
    /// Disables interaction and exposes native disabled semantics.
    #[props(default)]
    pub disabled: Option<bool>,
    /// Marks this field as required for native form validation.
    #[props(default)]
    pub required: Option<bool>,
    /// Applies the semantic invalid presentation alongside native `aria-invalid`.
    #[props(default)]
    pub invalid: bool,
    /// Change event handler.
    #[props(default)]
    pub oninput: EventHandler<FormEvent>,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native select/global attributes and events.
    #[props(extends = GlobalAttributes)]
    #[props(extends = select)]
    pub attributes: Vec<Attribute>,
    /// [`NativeSelectOption`]/[`NativeSelectOptGroup`] children.
    pub children: Element,
}

/// A styled native `<select>` with a chevron affordance.
#[component]
pub fn NativeSelect(props: NativeSelectProps) -> Element {
    let class = cn(&[
        "w-full min-w-0 appearance-none rounded-md border border-input bg-transparent px-3 pr-9 text-sm shadow-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40",
        props.size.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class: "group/native-select relative w-fit has-[select:disabled]:opacity-50",
            select {
                class,
                value: props.value,
                disabled: props.disabled,
                required: props.required,
                aria_invalid: props.invalid,
                oninput: move |event| props.oninput.call(event),
                ..props.attributes,
                {props.children}
            }
            ChevronDown {
                class: "pointer-events-none absolute top-1/2 right-3.5 size-4 -translate-y-1/2 text-muted-foreground opacity-50",
                size: 16,
            }
        }
    }
}

/// A styled `<option>` for [`NativeSelect`]. See this module's doc comment
/// for why it uses the `Canvas`/`CanvasText` system colors.
#[component]
pub fn NativeSelectOption(
    value: Option<String>,
    disabled: Option<bool>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "bg-[Canvas] text-[CanvasText]",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        option { class, value, disabled, {children} }
    }
}

/// A styled `<optgroup>` for [`NativeSelect`]. See this module's doc comment
/// for why it uses the `Canvas`/`CanvasText` system colors.
#[component]
pub fn NativeSelectOptGroup(label: String, class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "bg-[Canvas] text-[CanvasText]",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        optgroup { class, label, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_stay_compact_relative_to_default() {
        assert!(NativeSelectSize::Default.class().contains("h-9"));
        assert!(NativeSelectSize::Sm.class().contains("h-8"));
    }

    #[test]
    fn options_use_os_native_system_colors_not_app_tokens() {
        let class = cn(&["bg-[Canvas] text-[CanvasText]"]);
        assert!(class.contains("Canvas"));
    }
}
