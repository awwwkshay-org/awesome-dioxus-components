// Base UI's `CheckboxGroup` has exactly one part (`root`) and works by having each ordinary
// `Checkbox` in the group read ambient group context by its own `name` prop -- a mechanism this
// crate's `checkbox.rs` doesn't implement (it's a standalone controlled component with no
// awareness of an ancestor group). Rather than reaching into `checkbox.rs` to add implicit
// name-keyed group wiring (an unrelated file's behavior change for one new primitive's sake),
// this module translates the same capability as an explicit hook, `use_checkbox_group_item`,
// that a real `Checkbox` composes by name -- the same "hook, not a merge-props mechanism"
// translation `field.rs`'s `use_field_control()` already established for Base UI's one
// polymorphic part in that task's scope. `CheckboxGroupRoot::parent_state`/`toggle_all` gives a
// "select all" parent checkbox its checked/indeterminate/unchecked value and toggle behavior
// purely through composition (a consumer wires `Checkbox { checked: Some(ctx.parent_state()),
// on_checked_change: move |_| ctx.toggle_all() }`), matching the task's own instruction that
// Checkbox Group "compose `use_controlled` ... matching the uniform controlled/uncontrolled
// pattern every other primitive uses" without requiring changes to `checkbox.rs` itself.

//! Defines the [`CheckboxGroupRoot`] component and the [`use_checkbox_group_item`] hook, which
//! manage a named group of independent [`crate::checkbox::Checkbox`]es sharing one controlled
//! value set.

use std::collections::HashSet;

use dioxus::prelude::*;

use crate::{checkbox::CheckboxState, use_controlled};

#[derive(Clone, Copy)]
struct CheckboxGroupCtx {
    value: Memo<HashSet<String>>,
    set_value: Callback<HashSet<String>>,
    all_values: ReadSignal<Vec<String>>,
    disabled: ReadSignal<bool>,
}

impl CheckboxGroupCtx {
    fn is_checked(&self, name: &str) -> bool {
        (self.value)().contains(name)
    }

    fn set_checked(&self, name: String, checked: bool) {
        let mut next = (self.value)();
        if checked {
            next.insert(name);
        } else {
            next.remove(&name);
        }
        self.set_value.call(next);
    }

    /// The checked/indeterminate/unchecked state a "select all" parent
    /// checkbox should show, derived from how many of [`CheckboxGroupProps::all_values`]
    /// are currently in the group's value set.
    fn parent_state(&self) -> CheckboxState {
        let all = (self.all_values)();
        if all.is_empty() {
            return CheckboxState::Unchecked;
        }
        let value = (self.value)();
        let checked_count = all.iter().filter(|name| value.contains(*name)).count();
        if checked_count == 0 {
            CheckboxState::Unchecked
        } else if checked_count == all.len() {
            CheckboxState::Checked
        } else {
            CheckboxState::Indeterminate
        }
    }

    /// Selects every name in [`CheckboxGroupProps::all_values`] if not all are
    /// already selected, otherwise clears the value set. Intended for a
    /// "select all" parent checkbox's `on_checked_change`.
    fn toggle_all(&self) {
        let all = (self.all_values)();
        if all.is_empty() {
            return;
        }
        let value = (self.value)();
        let all_checked = all.iter().all(|name| value.contains(name));
        self.set_value.call(if all_checked {
            HashSet::new()
        } else {
            all.into_iter().collect()
        });
    }
}

/// The props for the [`CheckboxGroupRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxGroupRootProps {
    /// The names of the checkboxes in the group that are ticked. Used to
    /// drive the component when controlled.
    pub value: ReadSignal<Option<HashSet<String>>>,

    /// The names of the checkboxes that should be initially ticked, when the
    /// component is not controlled.
    #[props(default)]
    pub default_value: HashSet<String>,

    /// Callback fired when a checkbox in the group is ticked or unticked.
    #[props(default)]
    pub on_value_change: Callback<HashSet<String>>,

    /// The names of every checkbox in the group, used to compute a "select
    /// all" parent checkbox's checked/indeterminate state via
    /// [`use_checkbox_group_item`]'s consumer.
    #[props(default)]
    pub all_values: ReadSignal<Vec<String>>,

    /// Whether every checkbox in the group should ignore user interaction.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Additional attributes to apply to the group's root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the checkbox group, which should include
    /// [`crate::checkbox::Checkbox`]es whose `checked`/`on_checked_change`
    /// are wired through [`use_checkbox_group_item`].
    pub children: Element,
}

/// # CheckboxGroupRoot
///
/// The `CheckboxGroupRoot` component manages a set of independently rendered
/// [`crate::checkbox::Checkbox`]es sharing one controlled `HashSet<String>`
/// value keyed by each checkbox's `name`. A checkbox that wants to
/// participate in the group calls [`use_checkbox_group_item`] with its own
/// name and wires the returned checked/change pair onto itself.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::{
///     checkbox::{Checkbox, CheckboxIndicator},
///     checkbox_group::{CheckboxGroupRoot, use_checkbox_group_item},
/// };
///
/// #[component]
/// fn Item(name: &'static str) -> Element {
///     let (checked, set_checked) = use_checkbox_group_item(ReadSignal::new(Signal::new(name.to_string())));
///     rsx! {
///         Checkbox {
///             name,
///             checked: Some(if checked() { adico_primitives::checkbox::CheckboxState::Checked } else { adico_primitives::checkbox::CheckboxState::Unchecked }),
///             on_checked_change: move |state: adico_primitives::checkbox::CheckboxState| set_checked.call(state.into()),
///             CheckboxIndicator { "✅" }
///         }
///     }
/// }
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         CheckboxGroupRoot {
///             Item { name: "email" }
///             Item { name: "sms" }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`CheckboxGroupRoot`] component defines the following data
/// attributes you can use to control styling:
/// - `data-disabled`: Indicates whether the whole group ignores user
///   interaction. Values are `true` or `false`.
#[component]
pub fn CheckboxGroupRoot(props: CheckboxGroupRootProps) -> Element {
    let (value, set_value) =
        use_controlled(props.value, props.default_value, props.on_value_change);

    use_context_provider(|| CheckboxGroupCtx {
        value,
        set_value,
        all_values: props.all_values,
        disabled: props.disabled,
    });

    rsx! {
        div {
            role: "group",
            "data-disabled": props.disabled,
            ..props.attributes,

            {props.children}
        }
    }
}

/// Wires an individual [`crate::checkbox::Checkbox`] identified by `name`
/// into an ancestor [`CheckboxGroupRoot`]'s shared value set. Must be called
/// inside a `CheckboxGroupRoot`; panics otherwise (matches this crate's
/// established subcomponent convention, e.g. `checkbox::CheckboxIndicator`).
///
/// Returns `(checked, set_checked)`: `checked` reflects whether `name` is
/// currently in the group's value set, and calling `set_checked` inserts or
/// removes `name` from it.
pub fn use_checkbox_group_item(name: ReadSignal<String>) -> (Memo<bool>, Callback<bool>) {
    let ctx: CheckboxGroupCtx = use_context();
    let checked = use_memo(move || ctx.is_checked(&name()));
    let set_checked = use_callback(move |checked: bool| ctx.set_checked(name(), checked));
    (checked, set_checked)
}

/// Returns the ancestor [`CheckboxGroupRoot`]'s "select all" parent state
/// (checked/indeterminate/unchecked, derived from [`CheckboxGroupRootProps::all_values`])
/// and a callback that selects all or clears the group when called. Intended
/// for a group's own "select all" checkbox. Must be called inside a
/// `CheckboxGroupRoot`; panics otherwise.
pub fn use_checkbox_group_parent() -> (Memo<CheckboxState>, Callback<()>) {
    let ctx: CheckboxGroupCtx = use_context();
    let state = use_memo(move || ctx.parent_state());
    let toggle_all = use_callback(move |()| ctx.toggle_all());
    (state, toggle_all)
}

/// Whether the ancestor [`CheckboxGroupRoot`] as a whole is disabled. A
/// group member composes this alongside its own `disabled` prop, matching
/// how every other grouped primitive in this crate (e.g. `radio_group.rs`,
/// `toggle_group.rs`) cascades group-level disablement.
pub fn use_checkbox_group_disabled() -> ReadSignal<bool> {
    let ctx: CheckboxGroupCtx = use_context();
    ctx.disabled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_state_reflects_partial_selection() {
        let mut dom = VirtualDom::new(|| {
            let value = ReadSignal::new(Signal::new(Some(HashSet::from(["a".to_string()]))));
            let all_values = ReadSignal::new(Signal::new(vec!["a".to_string(), "b".to_string()]));
            rsx! {
                CheckboxGroupRoot { value, all_values,
                    Parent {}
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("indeterminate"), "{html}");
    }

    #[test]
    fn parent_state_is_unchecked_when_all_values_is_empty() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                CheckboxGroupRoot {
                    Parent {}
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("unchecked"), "{html}");
    }

    #[component]
    fn Parent() -> Element {
        let (state, _toggle_all) = use_checkbox_group_parent();
        rsx! {
            span { "data-state": state().to_data_state() }
        }
    }
}
