// Implements the WAI-ARIA APG "Spin Button" pattern (https://www.w3.org/WAI/ARIA/apg/patterns/spinbutton/)
// on `NumberFieldInput`: `role="spinbutton"` with `aria-valuemin`/`aria-valuemax`/`aria-valuenow`,
// moved by ArrowUp/ArrowDown (by `step`), PageUp/PageDown (by `large_step`, defaulting to 10x
// `step`), and Home/End (jump to `min`/`max` when bounded). This matches Base UI's own generated
// markup for `NumberField.Input` (a `role="spinbutton"` text input, not a native `<input
// type="number">`, so the displayed text can diverge from the committed numeric value while
// typing without the browser's own number-input parsing fighting it).
//
// The committed value (`NumberFieldRootProps::value`) is always a real `f64`, not Base UI's
// `number | null` -- this crate's shared `use_controlled` helper is generic over `T:
// Clone + PartialEq`, and modeling a nullable controlled value would need `T = Option<f64>`,
// i.e. a `ReadSignal<Option<Option<f64>>>` prop (controlled-or-not, wrapping has-a-value-or-not).
// That signature is real but the ergonomics cost (`Some(Some(5.0))` to set 5, `Some(None)` for
// "controlled and empty") isn't worth paying for this pass; `slider.rs`, the closest existing
// precedent, has the same always-a-number simplification. What's preserved instead is
// text-while-typing not fighting the committed value: `NumberFieldInput` keeps its own local
// `draft` text signal, only parsing/clamping/committing on blur, Enter, the step buttons, or an
// arrow/paging key -- not on every keystroke -- so an in-progress `"1."` isn't reformatted back
// to `"1"` mid-type.
//
// Deliberately NOT built (documented, not silently dropped, matching `field.rs`'s precedent for
// this task): `ScrubArea`/`ScrubAreaCursor` (pointer-lock-based drag-to-change, needs
// `document.requestPointerLock()` -- a genuinely separate browser-interop feature, real follow-on
// scope rather than one part of this primitive's own file); `smallStep` (Alt-held fine-adjustment
// modifier); `locale`/`format` (`Intl.NumberFormat`-driven display formatting -- no equivalent
// Rust-side formatting is composed here, so `NumberFieldInput` always displays/parses plain
// `f64::to_string`/`str::parse`); `allowWheelScrub` (mouse-wheel-while-hovering stepping).

//! Defines the [`NumberFieldRoot`] component and its sub-components, which provide an
//! accessible numeric input with increment/decrement controls.

use dioxus::prelude::*;

fn clamp(value: f64, min: Option<f64>, max: Option<f64>) -> f64 {
    let value = if let Some(min) = min {
        value.max(min)
    } else {
        value
    };
    if let Some(max) = max {
        value.min(max)
    } else {
        value
    }
}

#[derive(Clone, Copy)]
struct NumberFieldCtx {
    value: Memo<f64>,
    set_value: Callback<f64>,
    min: ReadSignal<Option<f64>>,
    max: ReadSignal<Option<f64>>,
    step: ReadSignal<f64>,
    large_step: ReadSignal<Option<f64>>,
    disabled: ReadSignal<bool>,
    read_only: ReadSignal<bool>,
    required: ReadSignal<bool>,
    name: ReadSignal<String>,
    input_id: Signal<String>,
}

impl NumberFieldCtx {
    fn clamp(&self, value: f64) -> f64 {
        clamp(value, (self.min)(), (self.max)())
    }

    fn step_value(&self, direction: f64, magnitude: f64) {
        if (self.disabled)() || (self.read_only)() {
            return;
        }
        let base = (self.value)();
        self.set_value
            .call(self.clamp(base + direction * magnitude));
    }

    fn large_step(&self) -> f64 {
        (self.large_step)().unwrap_or_else(|| (self.step)() * 10.0)
    }

    fn at_min(&self) -> bool {
        (self.min)().is_some_and(|min| (self.value)() <= min)
    }

    fn at_max(&self) -> bool {
        (self.max)().is_some_and(|max| (self.value)() >= max)
    }
}

/// The props for the [`NumberFieldRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NumberFieldRootProps {
    /// The current numeric value.
    pub value: ReadSignal<Option<f64>>,

    /// The uncontrolled value when the field is initially rendered.
    #[props(default)]
    pub default_value: f64,

    /// Callback fired when the value changes.
    #[props(default)]
    pub on_value_change: Callback<f64>,

    /// The minimum value. Unbounded when `None`.
    #[props(default)]
    pub min: ReadSignal<Option<f64>>,

    /// The maximum value. Unbounded when `None`.
    #[props(default)]
    pub max: ReadSignal<Option<f64>>,

    /// The amount to increment/decrement by with the buttons or arrow keys.
    #[props(default = ReadSignal::new(Signal::new(1.0)))]
    pub step: ReadSignal<f64>,

    /// The amount to increment/decrement by with Page Up/Page Down. Defaults
    /// to 10x `step` when `None`.
    #[props(default)]
    pub large_step: ReadSignal<Option<f64>>,

    /// Whether the field ignores user interaction.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether the user can view but not change the field's value.
    #[props(default)]
    pub read_only: ReadSignal<bool>,

    /// Whether the field must have a value before a form can submit.
    #[props(default)]
    pub required: ReadSignal<bool>,

    /// The name of the field, used in forms.
    #[props(default)]
    pub name: ReadSignal<String>,

    /// Additional attributes to apply to the field's root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the number field, typically a [`NumberFieldGroup`].
    pub children: Element,
}

/// # NumberFieldRoot
///
/// The `NumberFieldRoot` component provides a controlled numeric value and
/// step/bounds context to its [`NumberFieldGroup`], [`NumberFieldInput`],
/// [`NumberFieldIncrement`], and [`NumberFieldDecrement`] children.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::number_field::{
///     NumberFieldRoot, NumberFieldGroup, NumberFieldInput, NumberFieldIncrement, NumberFieldDecrement,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         NumberFieldRoot { aria_label: "Quantity", default_value: 1.0, min: Some(0.0), max: Some(10.0),
///             NumberFieldGroup {
///                 NumberFieldDecrement { "-" }
///                 NumberFieldInput {}
///                 NumberFieldIncrement { "+" }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`NumberFieldRoot`] component defines the following data attribute
/// you can use to control styling:
/// - `data-disabled`: Indicates whether the field ignores user interaction.
#[component]
pub fn NumberFieldRoot(props: NumberFieldRootProps) -> Element {
    let (value, set_value) =
        crate::use_controlled(props.value, props.default_value, props.on_value_change);
    let input_id = crate::use_unique_id();

    use_context_provider(|| NumberFieldCtx {
        value,
        set_value,
        min: props.min,
        max: props.max,
        step: props.step,
        large_step: props.large_step,
        disabled: props.disabled,
        read_only: props.read_only,
        required: props.required,
        name: props.name,
        input_id,
    });

    rsx! {
        div { "data-disabled": props.disabled, ..props.attributes, {props.children} }
    }
}

/// The props for the [`NumberFieldGroup`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NumberFieldGroupProps {
    /// Additional attributes to apply to the group element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the group, typically [`NumberFieldDecrement`],
    /// [`NumberFieldInput`], and [`NumberFieldIncrement`].
    pub children: Element,
}

/// # NumberFieldGroup
///
/// A purely presentational wrapper grouping the field's input and
/// increment/decrement buttons. Must be used inside a [`NumberFieldRoot`].
#[component]
pub fn NumberFieldGroup(props: NumberFieldGroupProps) -> Element {
    rsx! {
        div { role: "group", ..props.attributes, {props.children} }
    }
}

/// The props for the [`NumberFieldInput`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NumberFieldInputProps {
    /// Additional attributes to apply to the input element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # NumberFieldInput
///
/// The editable text input for a [`NumberFieldRoot`]'s value. Keeps its own
/// local draft text while focused so an in-progress edit (e.g. `"1."`)
/// isn't reformatted mid-type; commits (parses, clamps, and calls
/// `on_value_change`) on blur or Enter, reverting to the last committed
/// value's text if the draft doesn't parse. Must be used inside a
/// `NumberFieldRoot`.
///
/// ## Styling
///
/// The [`NumberFieldInput`] component defines the following data attribute
/// you can use to control styling:
/// - `data-disabled`: Indicates whether the field ignores user interaction.
#[component]
pub fn NumberFieldInput(props: NumberFieldInputProps) -> Element {
    let ctx: NumberFieldCtx = use_context();
    let mut draft = use_signal(|| (ctx.value)().to_string());

    use_effect(move || {
        draft.set((ctx.value)().to_string());
    });

    let mut commit_draft = move || {
        if (ctx.disabled)() || (ctx.read_only)() {
            return;
        }
        let parsed = draft.peek().trim().parse::<f64>();
        match parsed {
            Ok(parsed) => ctx.set_value.call(ctx.clamp(parsed)),
            Err(_) => draft.set((ctx.value)().to_string()),
        }
    };

    rsx! {
        input {
            id: ctx.input_id,
            type: "text",
            inputmode: "decimal",
            role: "spinbutton",
            value: draft,
            disabled: ctx.disabled,
            readonly: ctx.read_only,
            required: ctx.required,
            name: ctx.name,
            "aria-valuemin": (ctx.min)(),
            "aria-valuemax": (ctx.max)(),
            "aria-valuenow": (ctx.value)(),
            "data-disabled": ctx.disabled,

            oninput: move |evt| draft.set(evt.value()),
            onblur: move |_| commit_draft(),
            onkeydown: move |evt: Event<KeyboardData>| {
                let step = (ctx.step)();
                let large_step = ctx.large_step();
                match evt.key() {
                    Key::Enter => {
                        commit_draft();
                        evt.prevent_default();
                    }
                    Key::ArrowUp => {
                        ctx.step_value(1.0, step);
                        evt.prevent_default();
                    }
                    Key::ArrowDown => {
                        ctx.step_value(-1.0, step);
                        evt.prevent_default();
                    }
                    Key::PageUp => {
                        ctx.step_value(1.0, large_step);
                        evt.prevent_default();
                    }
                    Key::PageDown => {
                        ctx.step_value(-1.0, large_step);
                        evt.prevent_default();
                    }
                    Key::Home => {
                        if let Some(min) = (ctx.min)() {
                            ctx.set_value.call(min);
                        }
                        evt.prevent_default();
                    }
                    Key::End => {
                        if let Some(max) = (ctx.max)() {
                            ctx.set_value.call(max);
                        }
                        evt.prevent_default();
                    }
                    _ => {}
                }
            },

            ..props.attributes,
        }
    }
}

/// The props for the [`NumberFieldIncrement`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NumberFieldIncrementProps {
    /// Additional attributes to apply to the button element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the increment button.
    pub children: Element,
}

/// # NumberFieldIncrement
///
/// A button that increments the ancestor [`NumberFieldRoot`]'s value by one
/// `step`. Disabled once the value reaches `max`. Must be used inside a
/// `NumberFieldRoot`.
#[component]
pub fn NumberFieldIncrement(props: NumberFieldIncrementProps) -> Element {
    let ctx: NumberFieldCtx = use_context();
    let disabled = (ctx.disabled)() || (ctx.read_only)() || ctx.at_max();

    rsx! {
        button {
            type: "button",
            tabindex: "-1",
            disabled,
            "aria-hidden": "true",
            "data-disabled": disabled,
            onclick: move |_| ctx.step_value(1.0, (ctx.step)()),
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`NumberFieldDecrement`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NumberFieldDecrementProps {
    /// Additional attributes to apply to the button element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the decrement button.
    pub children: Element,
}

/// # NumberFieldDecrement
///
/// A button that decrements the ancestor [`NumberFieldRoot`]'s value by one
/// `step`. Disabled once the value reaches `min`. Must be used inside a
/// `NumberFieldRoot`.
#[component]
pub fn NumberFieldDecrement(props: NumberFieldDecrementProps) -> Element {
    let ctx: NumberFieldCtx = use_context();
    let disabled = (ctx.disabled)() || (ctx.read_only)() || ctx.at_min();

    rsx! {
        button {
            type: "button",
            tabindex: "-1",
            disabled,
            "aria-hidden": "true",
            "data-disabled": disabled,
            onclick: move |_| ctx.step_value(-1.0, (ctx.step)()),
            ..props.attributes,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_bounds_only_the_side_that_is_set() {
        assert_eq!(clamp(5.0, Some(0.0), None), 5.0);
        assert_eq!(clamp(-5.0, Some(0.0), None), 0.0);
        assert_eq!(clamp(15.0, None, Some(10.0)), 10.0);
        assert_eq!(clamp(5.0, None, None), 5.0);
    }

    #[test]
    fn increment_button_disables_at_max() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                NumberFieldRoot { aria_label: "Quantity", default_value: 10.0, max: Some(10.0),
                    NumberFieldGroup {
                        NumberFieldIncrement { "+" }
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("disabled"), "{html}");
    }
}
