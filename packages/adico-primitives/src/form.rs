// Implements a Dioxus-idiomatic translation of Base UI's `Form` root: a
// native `<form>` element whose native submit event is prevented by
// default (matching Base UI's own documented behavior: "preventDefault()
// is called on the native submit event when used"), relaying it to the
// caller's own `on_submit` callback instead.
//
// **Deliberately scoped down from Base UI's full `Form` API:** this does
// not implement cross-field validation orchestration -- Base UI's `Form`
// aggregates every nested `Field.Root`'s `validate` result on submit
// (respecting each field's own `validationMode`), exposes a
// `Form.Actions.validate()` imperative handle via `actionsRef`, and accepts
// an external `errors` object (field name -> message) for server-returned
// validation. None of that exists in this crate's `field.rs` yet (it has
// no internal `validate` callback or timing engine at all, by the same
// scope decision -- see that module's own doc comment), so there is
// nothing here to orchestrate. Building that validation-registry machinery
// is real, separate scope for a later pass, not silently dropped.

//! Defines the [`FormRoot`] component.

use dioxus::prelude::*;

/// The props for the [`FormRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct FormRootProps {
    /// Called when the form is submitted, after the native submit event's
    /// default action (a full-page navigation/reload) has already been
    /// prevented.
    #[props(default)]
    pub on_submit: Callback<Event<FormData>>,

    /// Additional attributes to apply to the form element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the form: typically one or more
    /// [`crate::field::FieldRoot`]s and a submit control.
    pub children: Element,
}

/// # FormRoot
///
/// The `FormRoot` component renders a native `form` element with its
/// native submit navigation already prevented, relaying the submit event
/// to [`FormRootProps::on_submit`] instead. See this module's own doc
/// comment for the Base UI `Form` requirements this deliberately does not
/// (yet) implement.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::form::FormRoot;
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         FormRoot {
///             on_submit: move |_event| {
///                 // handle submission
///             },
///             button { r#type: "submit", "Save" }
///         }
///     }
/// }
/// ```
#[component]
pub fn FormRoot(props: FormRootProps) -> Element {
    rsx! {
        form {
            onsubmit: move |event: Event<FormData>| {
                event.prevent_default();
                props.on_submit.call(event);
            },
            ..props.attributes,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[component]
    fn Harness() -> Element {
        rsx! {
            FormRoot {
                input { name: "email" }
                button { r#type: "submit", "Save" }
            }
        }
    }

    #[test]
    fn renders_a_native_form_element() {
        let mut dom = VirtualDom::new(Harness);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("<form"), "{html}");
        assert!(html.contains(r#"type="submit""#), "{html}");
    }
}
