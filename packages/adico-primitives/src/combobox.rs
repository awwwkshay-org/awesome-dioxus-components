// Implements the WAI-ARIA APG's Combobox (listbox popup) pattern: a text
// input that filters an anchored `role="listbox"` popup while DOM focus
// stays on the input itself (navigation moves `aria-activedescendant`, not
// real focus — unlike select.rs's listbox, which does move focus). `Combobox`/
// `ComboboxMulti` own the root open/query/value state, `ComboboxInput` is the
// APG's combobox input, and `ComboboxList` is the popup, anchored to the
// input via `positioner::Positioner` and dismissed via the shared
// `layer`/`use_escape_key`/`use_outside_dismiss` stack — this file previously
// rendered its listbox inline in normal document flow with no positioning or
// layer participation at all, the same gap `select.rs` had before its own
// task 2.1 rewrite. Filtering, selection, and roving focus are
// `selectable.rs`/`collection.rs`'s existing, independently-tested behavior;
// this file is the ARIA-role glue and event wiring around them.

//! Autocomplete input with a filterable, anchored popup list.
//!
//! `ComboboxInput` is the text input and trigger. `ComboboxList` contains
//! `ComboboxOption` children and is anchored to the input via
//! [`crate::positioner::Positioner`].

use dioxus::prelude::*;

use crate::{
    ContentAlign, ContentSide, Controlled,
    collection::{collection_item, use_item},
    listbox::{ListboxContext, ListboxItemIndicator, use_listbox_container},
    positioner::Positioner,
    selectable::{
        OptionState, RcPartialEqValue, SelectableContext, SelectableOptionConfig, SelectionMode,
        pointer_select_cancel, pointer_select_commit, pointer_select_start, use_selectable_option,
        use_selectable_root, use_single_selectable_value,
    },
    use_controlled, use_effect, use_escape_key, use_id_or, use_outside_dismiss, use_unique_id,
};

/// The default case-insensitive substring filter.
pub fn default_combobox_filter(query: &str, text: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty() || text.to_lowercase().contains(&query)
}

/// Shared state for every part of a [`Combobox`]/[`ComboboxMulti`] tree.
///
/// `pub` only for `packages/adico-primitives/tests/`, per this crate's test-placement
/// convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`);
/// not part of the intended public API.
#[derive(Clone, Copy)]
pub struct ComboboxContext {
    pub selectable: SelectableContext,
    pub query: Memo<String>,
    pub set_query: Callback<String>,
    pub filter: Callback<(String, String), bool>,
    /// The root element's stable id, bounding [`use_outside_dismiss`]'s
    /// pointer-outside check so clicking the input doesn't count as
    /// "outside" the combobox.
    pub root_id: Signal<String>,
    /// The input's resolved id (its own [`use_id_or`], honoring a
    /// caller-supplied override), published here so [`ComboboxList`] can use
    /// it as its [`Positioner`] anchor.
    pub input_id: Signal<String>,
}

impl ComboboxContext {
    pub fn set_open(&mut self, open: bool) {
        if open {
            self.selectable.collection.clear_focus();
        }
        self.selectable.set_open(open);
    }

    // Rust 2024's return-position `impl Trait` captures the elided `&self`
    // lifetime by default, which would tie the returned closure to `self`
    // and conflict with the `&mut self.selectable` calls below even though
    // the closure only holds `Copy` data extracted before it's built.
    // `use<>` opts back out, matching upstream's (pre-edition-2024) behavior.
    fn predicate_for(&self, query: String) -> impl Fn(&OptionState) -> bool + use<> {
        let filter = self.filter;
        move |option| filter.call((query.clone(), option.text_value.clone()))
    }

    fn predicate(&self) -> impl Fn(&OptionState) -> bool + use<> {
        self.predicate_for(self.query.cloned())
    }

    pub fn is_visible(&self, tab_index: usize) -> bool {
        let predicate = self.predicate();
        self.selectable
            .options
            .read()
            .iter()
            .find(|option| option.index == tab_index)
            .is_some_and(predicate)
    }

    pub fn has_visible_options(&self) -> bool {
        self.selectable.options.read().iter().any(self.predicate())
    }

    pub fn open_with_empty_query_and_focus_first(&mut self) {
        let query = String::new();
        self.set_query.call(query.clone());
        let initial_focus = self
            .selectable
            .first_matching_enabled_index(self.predicate_for(query));
        self.selectable.initial_focus.set(initial_focus);
        self.set_open(true);
    }

    pub fn open_with_empty_query_and_focus_last(&mut self) {
        let query = String::new();
        self.set_query.call(query.clone());
        let initial_focus = self
            .selectable
            .last_matching_enabled_index(self.predicate_for(query));
        self.selectable.initial_focus.set(initial_focus);
        self.set_open(true);
    }

    pub fn focused_option_id(&self) -> Option<String> {
        self.selectable.focused_option_id()
    }

    pub fn focus_next_visible(&mut self) {
        self.selectable.focus_next_where(self.predicate());
    }

    pub fn focus_prev_visible(&mut self) {
        self.selectable.focus_prev_where(self.predicate());
    }

    pub fn focus_first_visible(&mut self) {
        self.selectable.focus_first_where(self.predicate());
    }

    pub fn focus_last_visible(&mut self) {
        self.selectable.focus_last_where(self.predicate());
    }

    pub fn select_focused(&mut self) {
        self.selectable.select_focused();
    }
}

/// Props for [`Combobox`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxProps<T: Clone + PartialEq + 'static = String> {
    /// The controlled value. If supplied, the combobox is controlled
    /// and the signal's `None` value means no option is selected.
    #[props(default)]
    pub value: Option<ReadSignal<Option<T>>>,

    /// The default uncontrolled value.
    #[props(default)]
    pub default_value: Option<T>,

    /// Callback fired when the value changes.
    #[props(default)]
    pub on_value_change: Callback<Option<T>>,

    /// Whether the combobox is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// The controlled open state of the popup.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,

    /// The initial open state when uncontrolled.
    #[props(default)]
    pub default_open: ReadSignal<bool>,

    /// Callback fired when the popup open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// The controlled text query used to filter options.
    #[props(default)]
    pub query: ReadSignal<Option<String>>,

    /// The initial text query when uncontrolled.
    #[props(default)]
    pub default_query: ReadSignal<String>,

    /// Callback fired when the text query changes.
    #[props(default)]
    pub on_query_change: Callback<String>,

    /// Whether arrow-key navigation should wrap.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Custom filter callback. Receives `(query, option_text_value)`.
    #[props(default = Callback::new(|(q, t): (String, String)| default_combobox_filter(&q, &t)))]
    pub filter: Callback<(String, String), bool>,

    /// Additional attributes for the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children.
    pub children: Element,
}

/// Props for [`ComboboxMulti`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxMultiProps<T: Clone + PartialEq + 'static = String> {
    /// Controlled selected values. `None` leaves selection uncontrolled.
    #[props(default)]
    pub values: ReadSignal<Option<Vec<T>>>,
    /// Initial selected values when uncontrolled.
    #[props(default)]
    pub default_values: Vec<T>,
    /// Callback fired with the complete value set after an option toggles.
    #[props(default)]
    pub on_values_change: Callback<Vec<T>>,
    /// Whether the combobox is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Controlled popup state.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,
    /// Initial popup state when uncontrolled.
    #[props(default)]
    pub default_open: ReadSignal<bool>,
    /// Callback fired when the popup open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,
    /// Controlled filter text.
    #[props(default)]
    pub query: ReadSignal<Option<String>>,
    /// Initial filter text when uncontrolled.
    #[props(default)]
    pub default_query: ReadSignal<String>,
    /// Callback fired when filter text changes.
    #[props(default)]
    pub on_query_change: Callback<String>,
    /// Whether keyboard focus loops around visible options.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,
    /// Custom filter callback. Receives `(query, option_text_value)`.
    #[props(default = Callback::new(|(q, t): (String, String)| default_combobox_filter(&q, &t)))]
    pub filter: Callback<(String, String), bool>,
    /// Additional attributes for the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// Children.
    pub children: Element,
}

// Mirrors `use_selectable_root`'s already-accepted arg count plus the two
// combobox-specific controlled inputs (query, filter); grouping them would
// just relocate the same fields into a one-off struct with no other use.
#[allow(clippy::too_many_arguments)]
fn use_combobox_root(
    values: Memo<Vec<RcPartialEqValue>>,
    set_value: Callback<RcPartialEqValue>,
    selection_mode: SelectionMode,
    disabled: ReadSignal<bool>,
    roving_loop: ReadSignal<bool>,
    open: Controlled<bool>,
    query: Controlled<String>,
    filter: Callback<(String, String), bool>,
) -> (
    Memo<bool>,
    Signal<String>,
    impl FnMut(Event<KeyboardData>) + Clone,
) {
    let selectable = use_selectable_root(
        values,
        set_value,
        selection_mode,
        disabled,
        roving_loop,
        open,
    );
    let (query, set_query) = use_controlled(query.value, query.default.cloned(), query.on_change);
    let open = selectable.open;

    let root_id = use_unique_id();
    let input_id = use_signal(String::new);

    let mut ctx = ComboboxContext {
        selectable,
        query,
        set_query,
        filter,
        root_id,
        input_id,
    };
    use_context_provider(|| ctx);

    // Escape closes the combobox regardless of which descendant has focus;
    // see `use_escape_key`'s doc comment for why this lives on the root
    // rather than a document-level listener. `ComboboxList`'s own
    // outside-dismiss (below) joins this same layer via `use_layer_member`,
    // so both agree on which of them is topmost.
    let onkeydown = use_escape_key(move || ctx.set_open(false));

    (open, root_id, onkeydown)
}

/// A single-select autocomplete input with a filterable, anchored popup list.
#[component]
pub fn Combobox<T: Clone + PartialEq + 'static>(props: ComboboxProps<T>) -> Element {
    let (selected, set_value) = use_single_selectable_value(
        props.value,
        props.default_value,
        props.on_value_change,
        "combobox",
    );

    let (open, root_id, onkeydown) = use_combobox_root(
        selected,
        set_value,
        SelectionMode::Single,
        props.disabled,
        props.roving_loop,
        Controlled {
            value: props.open,
            default: props.default_open,
            on_change: props.on_open_change,
        },
        Controlled {
            value: props.query,
            default: props.default_query,
            on_change: props.on_query_change,
        },
        props.filter,
    );

    rsx! {
        div {
            id: root_id,
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": (props.disabled)(),
            onkeydown,
            ..props.attributes,
            {props.children}
        }
    }
}

/// A filterable multi-select autocomplete. Selecting an option toggles it and
/// keeps the popup open; the full selected value set is emitted to the caller.
#[component]
pub fn ComboboxMulti<T: Clone + PartialEq + 'static>(props: ComboboxMultiProps<T>) -> Element {
    let (selected_values, set_selected_values) =
        use_controlled(props.values, props.default_values, props.on_values_change);
    let values = use_memo(move || {
        selected_values()
            .into_iter()
            .map(RcPartialEqValue::new)
            .collect()
    });
    let set_value = use_callback(move |incoming: RcPartialEqValue| {
        let value = incoming
            .as_ref::<T>()
            .unwrap_or_else(|| panic!("combobox and option value types must match"))
            .clone();
        let mut current = selected_values();
        if let Some(index) = current.iter().position(|selected| selected == &value) {
            current.remove(index);
        } else {
            current.push(value);
        }
        set_selected_values.call(current);
    });

    let (open, root_id, onkeydown) = use_combobox_root(
        values,
        set_value,
        SelectionMode::Multiple,
        props.disabled,
        props.roving_loop,
        Controlled {
            value: props.open,
            default: props.default_open,
            on_change: props.on_open_change,
        },
        Controlled {
            value: props.query,
            default: props.default_query,
            on_change: props.on_query_change,
        },
        props.filter,
    );

    rsx! {
        div {
            id: root_id,
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": (props.disabled)(),
            onkeydown,
            ..props.attributes,
            {props.children}
        }
    }
}

/// Props for [`ComboboxInput`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxInputProps {
    /// Placeholder shown when the input is empty.
    #[props(default)]
    pub placeholder: ReadSignal<String>,

    /// Optional id for the input element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The text input that opens and filters the popup list, and which
/// [`ComboboxList`] anchors its position to.
#[component]
pub fn ComboboxInput(props: ComboboxInputProps) -> Element {
    let mut ctx = use_context::<ComboboxContext>();

    let id = use_unique_id();
    let id = use_id_or(id, props.id);
    let mut ctx_input_id = ctx.input_id;
    use_effect(move || {
        ctx_input_id.set(id());
    });

    let open = ctx.selectable.open;
    let query = ctx.query;
    let set_query = ctx.set_query;

    let active_descendant = use_memo(move || {
        if !open() {
            return None;
        }
        ctx.focused_option_id()
    });

    let display_value = use_memo(move || {
        if open() {
            query.cloned()
        } else {
            ctx.selectable.selected_texts().join(", ")
        }
    });

    let onkeydown = move |event: KeyboardEvent| match event.key() {
        Key::ArrowDown => {
            if !open() {
                ctx.open_with_empty_query_and_focus_first();
            } else {
                ctx.focus_next_visible();
            }
            event.prevent_default();
            event.stop_propagation();
        }
        Key::ArrowUp => {
            if !open() {
                ctx.open_with_empty_query_and_focus_last();
            } else {
                ctx.focus_prev_visible();
            }
            event.prevent_default();
            event.stop_propagation();
        }
        Key::Home if open() => {
            ctx.focus_first_visible();
            event.prevent_default();
            event.stop_propagation();
        }
        Key::End if open() => {
            ctx.focus_last_visible();
            event.prevent_default();
            event.stop_propagation();
        }
        Key::Enter if open() => {
            ctx.select_focused();
            event.prevent_default();
            event.stop_propagation();
        }
        // Escape is handled by the root's own `onkeydown` (via
        // `use_escape_key`), not here — falling through to `_` lets it
        // bubble up uninterrupted.
        _ => {}
    };

    rsx! {
        input {
            id,
            r#type: "text",
            value: display_value(),
            placeholder: props.placeholder,
            autocomplete: "off",
            spellcheck: "false",
            disabled: (ctx.selectable.disabled)(),

            role: "combobox",
            aria_autocomplete: "list",
            aria_haspopup: "listbox",
            aria_expanded: open(),
            aria_controls: ctx.selectable.list_id,
            aria_activedescendant: active_descendant(),

            "data-state": if open() { "open" } else { "closed" },

            onclick: move |_| {
                if !open() {
                    set_query.call(String::new());
                    ctx.set_open(true);
                }
            },
            oninput: move |event| {
                let was_open = open();
                let value = event.value();
                let next_query = if was_open {
                    value
                } else {
                    ctx.selectable
                        .selected_text()
                        .and_then(|selected| value.strip_prefix(&selected).map(ToString::to_string))
                        .unwrap_or(value)
                };
                set_query.call(next_query);
                if was_open {
                    ctx.selectable.collection.clear_focus();
                } else {
                    ctx.set_open(true);
                }
            },
            onkeydown,
            onblur: move |_| {
                if open() {
                    ctx.set_open(false);
                }
            },

            ..props.attributes,
        }
    }
}

/// Props for [`ComboboxList`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxListProps {
    /// Optional id for the list element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children, typically [`ComboboxOption`]s and an optional [`ComboboxEmpty`].
    pub children: Element,
}

/// Listbox that contains the visible options, anchored to [`ComboboxInput`]
/// via [`crate::positioner::Positioner`].
#[component]
pub fn ComboboxList(props: ComboboxListProps) -> Element {
    let mut ctx = use_context::<ComboboxContext>();
    let open = ctx.selectable.open;
    let listbox = use_listbox_container(props.id, ctx.selectable);
    let render = listbox.render;

    // Outside-pointer dismiss joins the root's layer via `use_layer_member`
    // (see its doc comment) so this later-mounted scope never shadows the
    // root's own `is_topmost` check for Escape.
    use_outside_dismiss(ctx.root_id, move || ctx.set_open(false));

    rsx! {
        if render() {
            Positioner {
                id: (listbox.id)(),
                anchor_id: ctx.input_id,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                offset: 4.0,
                role: "listbox",
                aria_multiselectable: ctx.selectable.selection_mode.is_multiple(),

                "data-state": if open() { "open" } else { "closed" },

                // Keeps focus in the input rather than having it stolen by
                // clicking inside the list.
                on_pointer_down: move |event: Event<PointerData>| {
                    event.prevent_default();
                },

                attributes: props.attributes,
                {props.children}
            }
        } else {
            {props.children}
        }
    }
}

/// Props for [`ComboboxOption`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxOptionProps<T: Clone + PartialEq + 'static> {
    /// The value carried by this option.
    pub value: ReadSignal<T>,

    /// Display/searchable text. Required for non-string types.
    #[props(default)]
    pub text_value: ReadSignal<Option<String>>,

    /// Whether the option is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Optional id for the option element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Registration order used for keyboard navigation.
    pub index: ReadSignal<usize>,

    /// Optional aria-label.
    #[props(default)]
    pub aria_label: Option<String>,

    /// Optional aria-roledescription.
    #[props(default)]
    pub aria_roledescription: Option<String>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children rendered inside the option.
    pub children: Element,
}

/// A selectable option inside a [`ComboboxList`].
#[component]
pub fn ComboboxOption<T: PartialEq + Clone + 'static>(props: ComboboxOptionProps<T>) -> Element {
    let index = props.index;

    let mut ctx: ComboboxContext = use_context();
    let visible = move || ctx.is_visible(index());
    let option = use_selectable_option(
        ctx.selectable,
        SelectableOptionConfig {
            id: props.id,
            index,
            value: props.value,
            text_value: props.text_value,
            option_disabled: props.disabled,
            component_name: "ComboboxOption",
        },
    );
    use_item(
        collection_item(ctx.selectable.collection, index)
            .key(move || Some(option.id.cloned()))
            .disabled(move || option.disabled.cloned())
            .hidden(move || !visible())
            .selected(move || (option.selected)()),
    );

    let render = use_context::<ListboxContext>().render;

    rsx! {
        if render() && visible() {
            div {
                role: "option",
                id: option.id,

                aria_selected: (option.selected)(),
                aria_disabled: (option.disabled)(),
                aria_label: props.aria_label.clone(),
                aria_roledescription: props.aria_roledescription.clone(),

                "data-highlighted": (option.focused)(),
                "data-disabled": (option.disabled)(),
                "data-selected": (option.selected)(),

                onmouseenter: move |_| {
                    if !(option.disabled)() {
                        ctx.selectable.collection.set_focus(Some((option.index)()));
                    }
                },
                onpointerdown: move |event| {
                    pointer_select_start(&event, (option.disabled)(), option.down_pos);
                },
                onpointerup: move |event| {
                    if pointer_select_commit(&event, (option.disabled)(), option.down_pos) {
                        ctx.selectable.select_value(RcPartialEqValue::new(option.value.cloned()));
                    }
                },
                onpointercancel: move |_| {
                    pointer_select_cancel(option.down_pos);
                },

                ..props.attributes,
                {props.children}
            }
        }
    }
}

/// Props for [`ComboboxItemIndicator`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxItemIndicatorProps {
    /// Children rendered only when the parent option is selected.
    pub children: Element,
}

/// Renders its children when the parent option is selected.
#[component]
pub fn ComboboxItemIndicator(props: ComboboxItemIndicatorProps) -> Element {
    rsx! {
        ListboxItemIndicator { {props.children} }
    }
}

/// Props for [`ComboboxEmpty`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxEmptyProps {
    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children rendered when no options match.
    pub children: Element,
}

/// Renders when no option matches the current query.
#[component]
pub fn ComboboxEmpty(props: ComboboxEmptyProps) -> Element {
    let ctx = use_context::<ComboboxContext>();
    let render = use_context::<ListboxContext>().render;

    let any_visible = use_memo(move || ctx.has_visible_options());

    if !render() || any_visible() {
        return rsx! {};
    }

    rsx! {
        div {
            role: "presentation",
            ..props.attributes,
            {props.children}
        }
    }
}
