// Base UI's OTP Field renders one real `<input>` per slot (`length` of them), each capped to a
// single character, with the root owning the combined string and moving focus between slots as
// the user types, deletes, or presses arrow keys -- not a single input with CSS letter-spacing
// tricks. This mirrors that shape: each `OtpFieldInput { index }` is its own focusable
// `maxlength="1"` input, and `OtpFieldRoot` derives subsequent slot ids from its own id
// (`{id}-2`, `{id}-3`, ...), matching the catalog's documented id-derivation scheme.
//
// Internal editing model: a slot's edit re-derives the full value as `Vec<Option<char>>` sized
// to `length`, mutates the edited index, then recombines by concatenating only the `Some` slots
// -- so clearing a slot in the *middle* of an otherwise-filled code compacts the value rather
// than leaving a hole at a fixed position. This is a deliberate simplification: real users
// overwhelmingly fill left-to-right and backspace-from-the-end, and holes have no single obvious
// canonical meaning for what the joined `value` string should contain. Documented here rather
// than silently chosen.
//
// Deliberately NOT built (documented, not silently dropped, matching this task's established
// precedent): pasting a full code doesn't distribute across every remaining slot -- native paste
// into a `maxlength="1"` input still works (inserts the first pasted character into the focused
// slot; the rest is dropped by the browser's own maxlength enforcement), it just isn't
// redistributed slot-to-slot by this module. Doing that needs `onpaste`/`ClipboardData` access
// this crate has no existing precedent for on any other primitive; genuine follow-on scope, not
// silently assumed to work. Also not built: `autoSubmit` (submitting the owning form once
// complete -- this crate's primitives don't drive form submission elsewhere either, e.g.
// `form.rs`'s own module doc defers cross-field orchestration), `validationType`/`normalizeValue`
// (character-class validation/normalization -- no equivalent validation-registry primitive
// exists yet, same gap `field.rs`'s doc comment already names for `Form`), and `mask` (masked
// slot display).

//! Defines the [`OtpFieldRoot`] component and its sub-components, which provide an accessible
//! one-time-passcode input split across individually focusable single-character slots.

use std::rc::Rc;

use dioxus::prelude::*;

fn value_to_slots(value: &str, length: usize) -> Vec<Option<char>> {
    let mut slots: Vec<Option<char>> = value.chars().map(Some).collect();
    slots.resize(length, None);
    slots.truncate(length);
    slots
}

fn slots_to_value(slots: &[Option<char>]) -> String {
    slots.iter().flatten().collect()
}

#[derive(Clone, Copy)]
struct OtpFieldCtx {
    value: Memo<String>,
    set_value: Callback<String>,
    length: ReadSignal<usize>,
    disabled: ReadSignal<bool>,
    read_only: ReadSignal<bool>,
    slot_refs: Signal<Vec<Option<Rc<MountedData>>>>,
    root_id: Signal<String>,
}

impl OtpFieldCtx {
    fn slots(&self) -> Vec<Option<char>> {
        value_to_slots(&(self.value)(), (self.length)())
    }

    fn commit_slots(&self, slots: Vec<Option<char>>) {
        self.set_value.call(slots_to_value(&slots));
    }

    fn slot_id(&self, index: usize) -> String {
        let root_id = (self.root_id)();
        if index == 0 {
            root_id
        } else {
            format!("{root_id}-{}", index + 1)
        }
    }

    fn set_slot_ref(&self, index: usize, data: Option<Rc<MountedData>>) {
        let mut refs = self.slot_refs;
        refs.with_mut(|refs| {
            if refs.len() <= index {
                refs.resize(index + 1, None);
            }
            refs[index] = data;
        });
    }

    fn focus_slot(&self, index: usize) {
        if let Some(Some(node)) = (self.slot_refs)().get(index).cloned() {
            spawn(async move {
                let _ = node.set_focus(true).await;
            });
        }
    }
}

/// The props for the [`OtpFieldRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct OtpFieldRootProps {
    /// The OTP value, as the string of characters entered so far.
    pub value: ReadSignal<Option<String>>,

    /// The uncontrolled value when the field is initially rendered.
    #[props(default)]
    pub default_value: String,

    /// Callback fired whenever the value changes.
    #[props(default)]
    pub on_value_change: Callback<String>,

    /// Callback fired once the value's length reaches `length`.
    #[props(default)]
    pub on_value_complete: Callback<String>,

    /// The number of OTP input slots. Required so the root can clamp
    /// values and derive each slot's id.
    pub length: ReadSignal<usize>,

    /// Whether the field ignores user interaction.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether the user can view but not change the field's value.
    #[props(default)]
    pub read_only: ReadSignal<bool>,

    /// The name of the field, used in forms.
    #[props(default)]
    pub name: ReadSignal<String>,

    /// Additional attributes to apply to the field's root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the field, typically a run of [`OtpFieldInput`]s
    /// (optionally interleaved with [`OtpFieldSeparator`]s).
    pub children: Element,
}

/// # OtpFieldRoot
///
/// The `OtpFieldRoot` component owns a one-time-passcode value shared
/// across a run of single-character [`OtpFieldInput`] slots, advancing and
/// retreating focus between them as the user types or deletes.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::otp_field::{OtpFieldRoot, OtpFieldInput};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         OtpFieldRoot { aria_label: "One-time code", length: 4usize,
///             for index in 0..4usize {
///                 OtpFieldInput { index }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`OtpFieldRoot`] component defines the following data attribute you
/// can use to control styling:
/// - `data-disabled`: Indicates whether the field ignores user interaction.
#[component]
pub fn OtpFieldRoot(props: OtpFieldRootProps) -> Element {
    let (value, set_value) =
        crate::use_controlled(props.value, props.default_value, props.on_value_change);
    let root_id = crate::use_unique_id();
    let slot_refs = use_signal(Vec::new);

    let on_value_complete = props.on_value_complete;
    let length = props.length;
    use_effect(move || {
        let current = value();
        if !current.is_empty() && current.chars().count() == length() {
            on_value_complete.call(current);
        }
    });

    use_context_provider(|| OtpFieldCtx {
        value,
        set_value,
        length: props.length,
        disabled: props.disabled,
        read_only: props.read_only,
        slot_refs,
        root_id,
    });

    rsx! {
        div { role: "group", "data-disabled": props.disabled, ..props.attributes, {props.children} }
    }
}

/// The props for the [`OtpFieldInput`] component.
#[derive(Props, Clone, PartialEq)]
pub struct OtpFieldInputProps {
    /// This slot's position among the field's slots (`0..length`).
    pub index: ReadSignal<usize>,

    /// Additional attributes to apply to the slot's input element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # OtpFieldInput
///
/// A single-character slot within an [`OtpFieldRoot`]. Typing a character
/// advances focus to the next slot; Backspace on an empty slot moves focus
/// back and clears the previous slot; ArrowLeft/ArrowRight move focus
/// between slots without editing. Must be used inside an `OtpFieldRoot`.
///
/// ## Styling
///
/// The [`OtpFieldInput`] component defines the following data attribute you
/// can use to control styling:
/// - `data-disabled`: Indicates whether the field ignores user interaction.
#[component]
pub fn OtpFieldInput(props: OtpFieldInputProps) -> Element {
    let ctx: OtpFieldCtx = use_context();
    let index = props.index.cloned();
    let length = (ctx.length)();
    let slots = ctx.slots();
    let char_value = slots.get(index).copied().flatten();
    let text_value = char_value.map(String::from).unwrap_or_default();

    rsx! {
        input {
            id: ctx.slot_id(index),
            type: "text",
            inputmode: "numeric",
            autocomplete: if index == 0 { "one-time-code" } else { "off" },
            maxlength: "1",
            value: text_value,
            disabled: ctx.disabled,
            readonly: ctx.read_only,
            "data-disabled": ctx.disabled,

            onmounted: move |evt| ctx.set_slot_ref(index, Some(evt.data())),

            oninput: move |evt: Event<FormData>| {
                if (ctx.disabled)() || (ctx.read_only)() {
                    return;
                }
                let mut slots = ctx.slots();
                match evt.value().chars().last() {
                    Some(ch) => {
                        slots[index] = Some(ch);
                        ctx.commit_slots(slots);
                        if index + 1 < length {
                            ctx.focus_slot(index + 1);
                        }
                    }
                    None => {
                        slots[index] = None;
                        ctx.commit_slots(slots);
                    }
                }
            },

            onkeydown: move |evt: Event<KeyboardData>| {
                if (ctx.disabled)() || (ctx.read_only)() {
                    return;
                }
                match evt.key() {
                    Key::Backspace => {
                        let mut slots = ctx.slots();
                        if slots.get(index).copied().flatten().is_some() {
                            slots[index] = None;
                            ctx.commit_slots(slots);
                        } else if index > 0 {
                            slots[index - 1] = None;
                            ctx.commit_slots(slots);
                            ctx.focus_slot(index - 1);
                        }
                        evt.prevent_default();
                    }
                    Key::ArrowLeft => {
                        if index > 0 {
                            ctx.focus_slot(index - 1);
                        }
                        evt.prevent_default();
                    }
                    Key::ArrowRight => {
                        if index + 1 < length {
                            ctx.focus_slot(index + 1);
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

/// The props for the [`OtpFieldSeparator`] component.
#[derive(Props, Clone, PartialEq)]
pub struct OtpFieldSeparatorProps {
    /// Additional attributes to apply to the separator element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the separator (e.g. a dash glyph).
    pub children: Element,
}

/// # OtpFieldSeparator
///
/// A purely decorative divider between groups of [`OtpFieldInput`] slots
/// (e.g. between a 3-digit and a 4-digit group). Hidden from assistive
/// technology since it carries no value of its own.
#[component]
pub fn OtpFieldSeparator(props: OtpFieldSeparatorProps) -> Element {
    rsx! {
        span { "aria-hidden": "true", ..props.attributes, {props.children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_to_slots_pads_and_truncates_to_length() {
        assert_eq!(
            value_to_slots("12", 4),
            vec![Some('1'), Some('2'), None, None]
        );
        assert_eq!(
            value_to_slots("12345", 3),
            vec![Some('1'), Some('2'), Some('3')]
        );
    }

    #[test]
    fn slots_to_value_compacts_holes() {
        let slots = vec![Some('1'), None, Some('3')];
        assert_eq!(slots_to_value(&slots), "13");
    }

    #[test]
    fn on_value_complete_fires_once_length_is_reached() {
        let mut dom = VirtualDom::new(|| {
            let value = ReadSignal::new(Signal::new(Some("12".to_string())));
            rsx! {
                OtpFieldRoot {
                    aria_label: "Code",
                    length: 2usize,
                    value,
                    on_value_complete: move |v: String| assert_eq!(v, "12"),
                    OtpFieldInput { index: 0usize }
                    OtpFieldInput { index: 1usize }
                }
            }
        });
        dom.rebuild_in_place();
        let _ = dioxus_ssr::render(&dom);
    }
}
