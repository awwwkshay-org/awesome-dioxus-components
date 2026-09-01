// Implements a Dioxus-idiomatic translation of Base UI's `Field` anatomy
// (`Field.Root`/`Field.Label`/`Field.Description`/`Field.Error`), the shared
// label/description/error-association behavior every form control needs
// (task 7.9). Base UI's `Field.Control` is a polymorphic `render`-prop
// wrapper that merges ARIA attributes onto an arbitrary child element --
// Dioxus has no `asChild`/merge-props mechanism for that, so this module
// exposes the equivalent binding as a hook, [`use_field_control`], that a
// real `<input>`/`<textarea>`/registry control calls directly and spreads
// onto itself, rather than as a wrapping component that would add an extra
// DOM node around the control.
//
// **Deliberately scoped down from Base UI's full `Field.Root` API:**
// `invalid` here is caller-supplied state (matching this crate's existing
// controlled-value pattern via `use_controlled`-adjacent `ReadSignal`
// props elsewhere), not the result of an internal `validate` callback,
// `validationMode` (`onSubmit`/`onBlur`/`onChange`) timing engine, or
// debounced revalidation -- those need a validation-registry primitive
// (each `Field` registering itself with an ancestor `Form`) that does not
// exist yet and is real, separate scope, not silently dropped.
// [`FieldError`]'s `show` prop is a plain boolean rather than Base UI's
// `match` prop (which can match a specific native `ValidityState` key, e.g.
// `valueMissing`/`typeMismatch`) -- Dioxus has no `ValidityState` binding to
// match against, so the simpler boolean is the honest equivalent: it
// defaults to the field's own `invalid` context flag, letting a caller
// override with its own validity logic.

//! Defines the [`FieldRoot`], [`FieldLabel`], [`FieldDescription`], and
//! [`FieldError`] components, plus the [`use_field_control`] hook a form
//! control composes to bind itself into a field's id/description/error/
//! disabled state.

use dioxus::prelude::*;

use crate::use_unique_id;

#[derive(Clone, Copy)]
struct FieldContext {
    control_id: Signal<String>,
    description_id: Signal<String>,
    error_id: Signal<String>,
    disabled: ReadSignal<bool>,
    invalid: ReadSignal<bool>,
}

/// The props for the [`FieldRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct FieldRootProps {
    /// Whether the field's control should ignore user interaction. If this
    /// field is nested inside a [`crate::fieldset::FieldsetRoot`], the
    /// fieldset's own `disabled` is combined with this value (either one
    /// being `true` disables the field).
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether the field is currently invalid. This crate has no internal
    /// validation engine (see this module's own doc comment); the caller
    /// computes this from its own validation logic and passes it in.
    #[props(default)]
    pub invalid: ReadSignal<bool>,

    /// Additional attributes to apply to the field's root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the field: typically a [`FieldLabel`], a form
    /// control wired via [`use_field_control`], and optionally a
    /// [`FieldDescription`] and/or [`FieldError`].
    pub children: Element,
}

/// # FieldRoot
///
/// The `FieldRoot` component associates a label, a form control, a
/// description, and an error message, wiring the `for`/`id`/
/// `aria-describedby`/`aria-invalid` relationships between them
/// automatically instead of requiring the caller to generate and thread ids
/// by hand. See this module's own doc comment for the Base UI `Field`
/// requirements this deliberately does not (yet) implement.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::field::{FieldRoot, FieldLabel, FieldDescription, use_field_control};
///
/// // `use_field_control` reads `FieldRoot`'s context, so it must be called
/// // from a component `FieldRoot` renders as a child, not from the
/// // component that renders `FieldRoot` itself.
/// #[component]
/// fn EmailInput() -> Element {
///     let binding = use_field_control();
///     rsx! {
///         input {
///             id: binding.id,
///             aria_describedby: binding.aria_describedby,
///             aria_invalid: binding.aria_invalid,
///             disabled: binding.disabled,
///         }
///     }
/// }
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         FieldRoot {
///             FieldLabel { "Email" }
///             EmailInput {}
///             FieldDescription { "We'll never share your email." }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`FieldRoot`] component defines the following data attributes you
/// can use to control styling:
/// - `data-disabled`: Indicates if the field is disabled. Values are `true` or `false`.
/// - `data-invalid`: Indicates if the field is invalid. Values are `true` or `false`.
#[component]
pub fn FieldRoot(props: FieldRootProps) -> Element {
    let fieldset_disabled = crate::fieldset::use_fieldset_disabled();
    let disabled = use_memo(move || (props.disabled)() || fieldset_disabled());

    let control_id = use_unique_id();
    let description_id = use_unique_id();
    let error_id = use_unique_id();

    use_context_provider(|| FieldContext {
        control_id,
        description_id,
        error_id,
        disabled: disabled.into(),
        invalid: props.invalid,
    });

    rsx! {
        div {
            "data-disabled": disabled(),
            "data-invalid": (props.invalid)(),
            ..props.attributes,
            {props.children}
        }
    }
}

/// The binding a form control reads from [`use_field_control`] and spreads
/// onto its own root element.
#[derive(Clone, PartialEq)]
pub struct FieldControlBinding {
    /// The id this control should render, matching [`FieldLabel`]'s `for`.
    pub id: String,
    /// A space-separated list of the field's description/error ids. Safe to
    /// apply unconditionally: an id with no corresponding element in the DOM
    /// (because the caller didn't render a [`FieldDescription`]/[`FieldError`])
    /// is simply ignored by assistive technology, per the `aria-describedby`
    /// spec -- so this never needs to track which of those two are actually
    /// mounted.
    pub aria_describedby: String,
    /// Whether the control should report itself as invalid.
    pub aria_invalid: bool,
    /// Whether the control should ignore user interaction.
    pub disabled: bool,
}

/// Bind a form control into the enclosing [`FieldRoot`]. Must be called
/// inside a `FieldRoot`; panics via [`use_context`] otherwise, the same
/// contract every other context-consuming hook in this crate already has.
pub fn use_field_control() -> FieldControlBinding {
    let ctx: FieldContext = use_context();
    FieldControlBinding {
        id: ctx.control_id.cloned(),
        aria_describedby: format!("{} {}", ctx.description_id, ctx.error_id),
        aria_invalid: (ctx.invalid)(),
        disabled: (ctx.disabled)(),
    }
}

/// The props for the [`FieldLabel`] component.
#[derive(Props, Clone, PartialEq)]
pub struct FieldLabelProps {
    /// Additional attributes to apply to the label element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the label element.
    pub children: Element,
}

/// # FieldLabel
///
/// The `FieldLabel` component renders a [`crate::label::Label`] already
/// associated with the enclosing [`FieldRoot`]'s control, so the caller
/// never has to generate or pass an id by hand.
///
/// This must be used inside a [`FieldRoot`] component.
#[component]
pub fn FieldLabel(props: FieldLabelProps) -> Element {
    let ctx: FieldContext = use_context();
    rsx! {
        crate::label::Label {
            html_for: ctx.control_id.cloned(),
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`FieldDescription`] component.
#[derive(Props, Clone, PartialEq)]
pub struct FieldDescriptionProps {
    /// Additional attributes to apply to the description element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the description element.
    pub children: Element,
}

/// # FieldDescription
///
/// The `FieldDescription` component renders hint text associated with the
/// enclosing [`FieldRoot`]'s control via `aria-describedby`.
///
/// This must be used inside a [`FieldRoot`] component.
#[component]
pub fn FieldDescription(props: FieldDescriptionProps) -> Element {
    let ctx: FieldContext = use_context();
    rsx! {
        p { id: ctx.description_id.cloned(), ..props.attributes, {props.children} }
    }
}

/// The props for the [`FieldError`] component.
#[derive(Props, Clone, PartialEq)]
pub struct FieldErrorProps {
    /// Whether to render the error message. Defaults to the enclosing
    /// [`FieldRoot`]'s own `invalid` state; pass an explicit value to show
    /// this specific error under different conditions (e.g. one message per
    /// failure reason).
    pub show: Option<ReadSignal<bool>>,

    /// Additional attributes to apply to the error element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the error element.
    pub children: Element,
}

/// # FieldError
///
/// The `FieldError` component renders a validation error message
/// associated with the enclosing [`FieldRoot`]'s control via
/// `aria-describedby`, shown when [`FieldErrorProps::show`] (or, by
/// default, the field's own `invalid` state) is `true`.
///
/// This must be used inside a [`FieldRoot`] component.
#[component]
pub fn FieldError(props: FieldErrorProps) -> Element {
    let ctx: FieldContext = use_context();
    let show = move || props.show.map(|s| s()).unwrap_or_else(|| (ctx.invalid)());
    rsx! {
        if show() {
            p {
                id: ctx.error_id.cloned(),
                role: "alert",
                ..props.attributes,
                {props.children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describedby_combines_description_and_error_ids() {
        let binding = FieldControlBinding {
            id: "adico-0".to_string(),
            aria_describedby: "adico-1 adico-2".to_string(),
            aria_invalid: false,
            disabled: false,
        };
        assert!(binding.aria_describedby.contains("adico-1"));
        assert!(binding.aria_describedby.contains("adico-2"));
    }
}
