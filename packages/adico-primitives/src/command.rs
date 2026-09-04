// shadcn's Command (built on `cmdk`) looks like a `combobox.rs`-shaped input+filtered-list at
// first glance, but its two defining differences make reusing `combobox.rs`'s own root/list
// directly the wrong fit rather than genuine composition:
//
// 1. Command's list is *always visible* under the input -- there is no open/closed popup
//    concept, and `combobox.rs`'s own `ComboboxList` unconditionally anchors via
//    `positioner::Positioner`, which floating-positions relative to an anchor exactly because a
//    combobox's list is normally a closable popup. Forcing that shape open permanently would be
//    composing the wrong primitive, not reusing the right one.
// 2. `cmdk`'s `Command.Item` carries its own `onSelect` callback per item (closer to
//    `menu.rs::MenuItem`'s per-item callback shape), and `Command.Root`'s own `value`/
//    `onValueChange` tracks which item is currently *highlighted* -- not a committed selection.
//    `combobox.rs`'s model is the opposite: a single root `on_value_change` fires only once an
//    option is *chosen*, closing the popup. These are genuinely different state machines, not
//    the same one under a different name.
//
// What *is* genuinely reused: `combobox::default_combobox_filter` (the same substring filter
// `Combobox`/`Autocomplete` already ship and this crate has tested) and `collection.rs`'s
// roving-focus `CollectionState` (the same `hidden`-aware navigation `toggle_group.rs`/
// `menu.rs`/every other roving-focus primitive in this crate already uses) -- `CommandItem`
// marks itself `hidden` when the query filter excludes it, so `CollectionState`'s existing
// navigation already skips filtered-out items with no new logic needed here. `CommandItem`'s own
// text-value registration (`text_values: Signal<HashMap<usize, String>>`) and per-item callback
// registration (`activate: Signal<HashMap<usize, Callback<()>>>`, letting `CommandInput`'s Enter
// key invoke whichever item is currently focused without a central "select_value" concept) both
// mirror `menu.rs::MenuItem`'s identical registration-effect-then-`use_effect_cleanup` shape,
// added this same session for its own typeahead wiring.
//
// One `collection.rs` wiring detail `CommandItem` deliberately does NOT copy from
// `menu.rs`/`toggle_group.rs`: it never sets `onmounted: item.onmounted()`. That call feeds
// `CollectionState::control_mount_focus`, which moves *real* DOM focus onto whichever item is
// "focused" in the roving sense -- correct for those roving-tabindex widgets, but wrong here.
// Real focus needs to stay on `CommandInput` at all times, with the roving highlight expressed
// only through `aria-activedescendant`/`data-highlighted` (found live: the first draft did wire
// `onmounted`, and a second ArrowDown appeared to do nothing because focus had silently jumped
// off the input onto the first item, so the *input's own* keydown handler stopped receiving
// events at all). `combobox::ComboboxOption` already omits the same wiring for the identical
// reason -- confirmed by re-reading it after finding this live, not guessed at.

//! Filterable command palette: an always-visible, roving-focus, query-filtered list of
//! per-item-callback actions, optionally wrapped in a [`crate::dialog::Dialog`] by the consumer
//! for the "command dialog" variant (no dedicated wrapper here -- composing `Dialog` directly is
//! already the whole of what that variant needs, matching this crate's "don't wrap a primitive
//! in another primitive for no behavioral reason" convention).

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::{
    collection::{CollectionState, collection_item, use_collection_provider, use_item},
    combobox::default_combobox_filter,
    use_controlled, use_effect_cleanup, use_id_or, use_unique_id,
};

#[derive(Clone, Copy)]
struct CommandContext {
    collection: CollectionState,
    query: Memo<String>,
    set_query: Callback<String>,
    filter: Callback<(String, String), bool>,
    disabled: ReadSignal<bool>,
    text_values: Signal<HashMap<usize, String>>,
    activate: Signal<HashMap<usize, Callback<()>>>,
    input_id: Signal<String>,
    list_id: Signal<String>,
}

impl CommandContext {
    fn is_visible(&self, index: usize) -> bool {
        let query = self.query.cloned();
        self.text_values
            .read()
            .get(&index)
            .is_some_and(|text| self.filter.call((query, text.clone())))
    }

    fn has_visible_items(&self) -> bool {
        self.text_values
            .read()
            .keys()
            .any(|&index| self.is_visible(index))
    }

    fn activate_focused(&mut self) {
        let Some(index) = self.collection.focused_index() else {
            return;
        };
        if let Some(callback) = self.activate.read().get(&index).copied() {
            callback.call(());
        }
    }
}

/// The props for the [`CommandRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CommandRootProps {
    /// The controlled filter text.
    #[props(default)]
    pub query: ReadSignal<Option<String>>,

    /// The initial filter text when uncontrolled.
    #[props(default)]
    pub default_query: ReadSignal<String>,

    /// Callback fired when the filter text changes.
    #[props(default)]
    pub on_query_change: Callback<String>,

    /// Whether the whole command palette ignores user interaction.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether arrow-key navigation should wrap.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Custom filter callback. Receives `(query, item_text_value)`. Defaults
    /// to the same case-insensitive substring filter
    /// [`crate::combobox::Combobox`] ships.
    #[props(default = Callback::new(|(q, t): (String, String)| default_combobox_filter(&q, &t)))]
    pub filter: Callback<(String, String), bool>,

    /// Additional attributes for the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the command palette, typically a [`CommandInput`]
    /// followed by a [`CommandList`].
    pub children: Element,
}

/// # CommandRoot
///
/// The root of a command palette: an always-visible, query-filtered,
/// roving-focus list of actions. Unlike [`crate::combobox::Combobox`],
/// there is no popup to open or close -- [`CommandList`] is always
/// rendered, and each [`CommandItem`] carries its own `on_select` callback
/// rather than the root tracking a single committed value.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::command::{CommandRoot, CommandInput, CommandList, CommandItem, CommandEmpty};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         CommandRoot {
///             CommandInput { placeholder: "Search...".to_string() }
///             CommandList {
///                 CommandEmpty { "No results." }
///                 CommandItem::<String> { index: 0usize, value: "profile".to_string(), on_select: move |_| {}, "Profile" }
///                 CommandItem::<String> { index: 1usize, value: "billing".to_string(), on_select: move |_| {}, "Billing" }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`CommandRoot`] component defines the following data attribute you
/// can use to control styling:
/// - `data-disabled`: Indicates whether the palette ignores user interaction.
#[component]
pub fn CommandRoot(props: CommandRootProps) -> Element {
    let (query, set_query) = use_controlled(
        props.query,
        props.default_query.cloned(),
        props.on_query_change,
    );
    let collection = use_collection_provider(props.roving_loop);

    use_context_provider(|| CommandContext {
        collection,
        query,
        set_query,
        filter: props.filter,
        disabled: props.disabled,
        text_values: Signal::new(HashMap::new()),
        activate: Signal::new(HashMap::new()),
        input_id: Signal::new(String::new()),
        list_id: Signal::new(String::new()),
    });

    rsx! {
        div { "data-disabled": props.disabled, ..props.attributes, {props.children} }
    }
}

/// The props for the [`CommandInput`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CommandInputProps {
    /// Placeholder shown when the input is empty.
    #[props(default)]
    pub placeholder: ReadSignal<String>,

    /// Optional id for the input element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the input element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # CommandInput
///
/// The search input that filters the ancestor [`CommandList`]'s items.
/// ArrowUp/ArrowDown move the roving highlight (skipping filtered-out
/// items, via [`crate::collection::CollectionState`]'s existing
/// `hidden`-aware navigation); Enter activates the currently highlighted
/// item's own `on_select`. Must be used inside a [`CommandRoot`].
#[component]
pub fn CommandInput(props: CommandInputProps) -> Element {
    let mut ctx: CommandContext = use_context();
    let generated_id = use_unique_id();
    let id = use_id_or(generated_id, props.id);
    let mut input_id = ctx.input_id;
    use_effect(move || {
        input_id.set(id());
    });

    let query = ctx.query;
    let set_query = ctx.set_query;
    let active_descendant = move || ctx.collection.focused_key();

    rsx! {
        input {
            id,
            r#type: "text",
            value: query.cloned(),
            placeholder: props.placeholder,
            autocomplete: "off",
            spellcheck: "false",
            disabled: ctx.disabled,

            role: "textbox",
            aria_autocomplete: "list",
            aria_controls: ctx.list_id.cloned(),
            aria_activedescendant: active_descendant(),

            oninput: move |event| {
                set_query.call(event.value());
                ctx.collection.clear_focus();
            },
            onkeydown: move |event: Event<KeyboardData>| {
                match event.key() {
                    Key::ArrowDown => ctx.collection.focus_next(),
                    Key::ArrowUp => ctx.collection.focus_prev(),
                    Key::Home => ctx.collection.focus_first(),
                    Key::End => ctx.collection.focus_last(),
                    Key::Enter => ctx.activate_focused(),
                    _ => return,
                }
                event.prevent_default();
            },

            ..props.attributes,
        }
    }
}

/// The props for the [`CommandList`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CommandListProps {
    /// Optional id for the list element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the list element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the list, typically [`CommandGroup`]s and/or
    /// [`CommandItem`]s, and an optional [`CommandEmpty`].
    pub children: Element,
}

/// # CommandList
///
/// The always-visible container for a [`CommandRoot`]'s
/// [`CommandItem`]s -- unlike [`crate::combobox::ComboboxList`], this never
/// anchors via [`crate::positioner::Positioner`], since a command palette's
/// list has no popup to position. Must be used inside a [`CommandRoot`].
#[component]
pub fn CommandList(props: CommandListProps) -> Element {
    let ctx: CommandContext = use_context();
    let generated_id = use_unique_id();
    let id = use_id_or(generated_id, props.id);
    let mut list_id = ctx.list_id;
    use_effect(move || {
        list_id.set(id());
    });

    rsx! {
        div { id, role: "listbox", ..props.attributes, {props.children} }
    }
}

/// The props for the [`CommandGroup`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CommandGroupProps {
    /// Additional attributes for the group element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The group's contents, typically a heading followed by [`CommandItem`]s.
    pub children: Element,
}

/// # CommandGroup
///
/// A purely visual/ARIA grouping of related items; does not affect
/// keyboard navigation ordering (matching [`crate::menu::MenuGroup`]'s
/// identical non-participating shape).
#[component]
pub fn CommandGroup(props: CommandGroupProps) -> Element {
    rsx! {
        div { role: "group", ..props.attributes, {props.children} }
    }
}

/// The props for the [`CommandSeparator`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CommandSeparatorProps {
    /// Additional attributes for the separator element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # CommandSeparator
///
/// A visual divider between groups of items.
#[component]
pub fn CommandSeparator(props: CommandSeparatorProps) -> Element {
    rsx! {
        div { role: "separator", "aria-orientation": "horizontal", ..props.attributes }
    }
}

/// The props for the [`CommandItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CommandItemProps<T: Clone + PartialEq + 'static> {
    /// This item's position for keyboard navigation ordering.
    pub index: ReadSignal<usize>,

    /// Optional id for the item element. Generated if not provided.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// The value passed to `on_select` when this item is activated.
    pub value: ReadSignal<T>,

    /// Display/searchable text used for filtering. Required for non-`String`
    /// value types; defaults to the value's own text for `T = String`
    /// consumers that don't set it (matching
    /// [`crate::combobox::ComboboxOption`]'s identical default).
    #[props(default)]
    pub text_value: ReadSignal<Option<String>>,

    /// Whether this item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Called when this item is activated (click, or Enter while
    /// highlighted).
    #[props(default)]
    pub on_select: Callback<T>,

    /// Additional attributes for the item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The item's contents.
    pub children: Element,
}

/// # CommandItem
///
/// A filterable action inside a [`CommandList`]. Hidden (and excluded from
/// roving-focus navigation) whenever the ancestor [`CommandInput`]'s query
/// doesn't match its `text_value`. Must be used inside a [`CommandRoot`].
///
/// ## Styling
///
/// The [`CommandItem`] component defines the following data attributes you
/// can use to control styling:
/// - `data-highlighted`: Indicates whether this item currently has the
///   roving-focus highlight.
/// - `data-disabled`: Indicates whether this item ignores user interaction.
#[component]
pub fn CommandItem<T: Clone + PartialEq + 'static>(props: CommandItemProps<T>) -> Element {
    let mut ctx: CommandContext = use_context();
    let index = props.index;
    let disabled = move || (ctx.disabled)() || (props.disabled)();

    // Computed directly from this item's own props (query + filter + its
    // own text_value), not via `ctx`'s `text_values` registry -- that
    // registry is only populated by `use_effect` (below), which hasn't run
    // yet on this same, first synchronous render. An item's *own* render
    // gate must not depend on data only its own not-yet-run effect would
    // supply, or every item would incorrectly hide itself on first paint
    // (found live while testing this component, not a hypothetical).
    let text_value = use_memo(move || {
        crate::selection::option_text_value(
            &(props.value)(),
            props.text_value.cloned(),
            "CommandItem",
        )
    });
    let visible = move || {
        let query = ctx.query.cloned();
        ctx.filter.call((query, text_value.cloned()))
    };

    let generated_id = use_unique_id();
    let id = use_id_or(generated_id, props.id);

    let item = use_item(
        collection_item(ctx.collection, index)
            .key(move || Some(id.cloned()))
            .disabled(disabled)
            .hidden(move || !visible()),
    );
    let focused = move || item.focused();

    // `ctx.text_values` (below) exists only for `CommandEmpty`'s
    // cross-component "does anything match" check, which genuinely cannot
    // be computed synchronously from a differently-positioned sibling's own
    // render -- see `CommandEmpty`'s own doc comment for the one-render-late
    // consequence this has for non-hydrated SSR output specifically.
    let mut registered_index: Signal<Option<usize>> = use_signal(|| None);
    use_effect(move || {
        let current = index.cloned();
        let mut text_values = ctx.text_values;
        if let Some(previous) = registered_index.peek().as_ref()
            && *previous != current
        {
            text_values.write().remove(previous);
        }
        text_values.write().insert(current, text_value.cloned());
        registered_index.set(Some(current));
    });
    use_effect_cleanup(move || {
        if let Some(index) = *registered_index.peek() {
            ctx.text_values.write().remove(&index);
        }
    });

    let activate = use_callback(move |()| {
        if !disabled() {
            props.on_select.call((props.value)());
        }
    });
    use_effect(move || {
        let current = index.cloned();
        ctx.activate.write().insert(current, activate);
    });
    use_effect_cleanup(move || {
        ctx.activate.write().remove(&(index.cloned()));
    });

    rsx! {
        if visible() {
            div {
                id,
                role: "option",
                "aria-selected": focused(),
                "data-highlighted": focused(),
                "data-disabled": disabled(),
                tabindex: "-1",

                onmouseenter: move |_| {
                    if !disabled() {
                        ctx.collection.set_focus(Some(index.cloned()));
                    }
                },
                onclick: move |_| activate.call(()),

                ..props.attributes,
                {props.children}
            }
        }
    }
}

/// The props for the [`CommandEmpty`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CommandEmptyProps {
    /// Additional attributes for the element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children rendered when no item matches the current query.
    pub children: Element,
}

/// # CommandEmpty
///
/// Renders only when no [`CommandItem`] in the ancestor [`CommandRoot`]
/// matches the current filter query. Must be used inside a `CommandRoot`.
///
/// Reads each `CommandItem`'s text value from a registry those items
/// populate via their own mount effect, since which items exist (and
/// whether any of them match) can only be known from *outside* any single
/// item's own render -- unlike each `CommandItem`'s own visibility (computed
/// synchronously from its own props, not this registry). Consequence: on
/// non-hydrated server-rendered output specifically (a static snapshot that
/// never runs client-side effects at all), this can render as if nothing
/// matches even when items do -- a real client render/hydration corrects it
/// on the very next tick, the same class of one-render-late caveat this
/// crate already accepts for `use_animated_open`-gated content elsewhere.
#[component]
pub fn CommandEmpty(props: CommandEmptyProps) -> Element {
    let ctx: CommandContext = use_context();
    if ctx.has_visible_items() {
        return rsx!({});
    }
    rsx! {
        div { ..props.attributes, {props.children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_visible_reflects_the_filter_predicate_on_the_very_first_render() {
        // Deliberately a single `rebuild_in_place()` with no follow-up
        // effect flush: this is exactly the scenario `CommandItem`'s fix
        // (computing its own visibility synchronously, not from the
        // effect-populated `text_values` registry) targets. Before that
        // fix, every item hid itself on this same first render regardless
        // of whether it matched.
        let mut dom = VirtualDom::new(|| {
            rsx! {
                CommandRoot { default_query: "ban".to_string(),
                    CommandInput {}
                    CommandList {
                        CommandItem::<String> { index: 0usize, value: "apple".to_string(), text_value: "Apple".to_string(), "Apple" }
                        CommandItem::<String> { index: 1usize, value: "banana".to_string(), text_value: "Banana".to_string(), "Banana" }
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(!html.contains(">Apple<"), "{html}");
        assert!(html.contains(">Banana<"), "{html}");
    }

    // No "hides once a matching item exists" test: `CommandEmpty`'s own doc
    // comment documents why that specifically requires an effect flush this
    // harness has no way to drive after the very first render (see that
    // comment for the real-world consequence, limited to non-hydrated SSR
    // output). What *is* tested below is the one behavior that's true
    // regardless of effect timing: with nothing registered yet (true on
    // every first render) and a query that matches nothing either way, it
    // shows.
    #[test]
    fn empty_state_shows_when_nothing_matches() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                CommandRoot { default_query: "zzz".to_string(),
                    CommandInput {}
                    CommandList {
                        CommandEmpty { "No results." }
                        CommandItem::<String> { index: 0usize, value: "banana".to_string(), text_value: "Banana".to_string(), "Banana" }
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("No results."), "{html}");
    }
}
