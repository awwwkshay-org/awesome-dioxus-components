// Implements the WAI-ARIA APG Accordion pattern: each AccordionTrigger is a button with
// aria-expanded/aria-controls pointing at its AccordionContent, with a single-tab-stop roving
// tabindex among triggers (arrow keys along the accordion's own orientation, Home/End to the
// ends) -- Accordion/AccordionMulti implement the pattern's single- vs. multiple-open-panel
// variants per shadcn/Radix's own AccordionPrimitive.Root convention. This file was already a
// genuine adaptation, not ported-unmodified, and that adaptation is unchanged here:
//
// Adapted from upstream: dropped the `crate::dioxus_elements::Key` import
// (upstream-internal re-export path that does not exist in this crate);
// `Key` already resolves through `dioxus::prelude::*`, matching every other
// primitive in this crate that matches on keyboard keys.
//
// Adapted further (task 7.8b): closes this file's own two long-standing
// TODOs. `Accordion`/`AccordionMulti` now split single/multiple-open mode
// into two components (matching this crate's established `Select`/
// `SelectMulti` convention) with real controlled/uncontrolled `String`
// `value`/`values` state, matching shadcn/Radix's `AccordionPrimitive.Root`
// exactly (Radix's own value type is `string`, so no generic `T` is needed
// here the way `Select<T>` needs one) — closing "TODO: controlled version".
// `AccordionItem.value` is now a required identity, replacing the previous
// internal numeric-id-only addressing. On "TODO: rewrite this to use
// collapsible": `AccordionContent`'s open-gated rendering already used the
// same technique (`use_animated_open`) `collapsible.rs`'s `CollapsibleContent`
// uses, so the pattern was already shared; literally nesting `Collapsible`
// components per item was not done, since layering this module's
// roving-focus keydown/focus handling on top of `CollapsibleTrigger`'s own
// handlers would need attribute-merging this crate has no established
// pattern for, and risks a subtle, currently-untestable-without-a-browser
// keyboard regression in an already Playwright-verified component for
// uncertain benefit over the existing (working, equivalent) approach.
// Also fixed, while touching `AccordionTrigger`: it never rendered
// `data-state`, so `registry/ui/accordion.rs`'s own
// `[&[data-state=open]>svg]:rotate-180` chevron-rotation selector could
// never have matched anything — a real, free, pre-existing bug fix.

//! Defines the [`Accordion`]/[`AccordionMulti`] components and their sub-components.

use crate::collection::{CollectionState, collection_item, use_collection_provider, use_item};
use crate::{use_animated_open, use_id_or, use_unique_id};
use dioxus::prelude::*;

/// Internal accordion context, shared by [`Accordion`] and [`AccordionMulti`].
/// `toggle` closes over each root's own single/multiple-open semantics, so
/// [`AccordionItem`]/[`AccordionTrigger`]/[`AccordionContent`] don't need to
/// know which mode they're in.
#[derive(Clone, Copy)]
struct AccordionContext {
    /// Used to assign each item's roving-focus navigation order.
    next_index: Signal<usize>,

    /// The currently open item value(s) — one for [`Accordion`], any number
    /// for [`AccordionMulti`].
    open_values: Memo<Vec<String>>,

    /// Toggles a single value's open/closed state, per the root's own mode.
    toggle: Callback<String>,

    /// Whether the entire accordion is disabled.
    disabled: ReadSignal<bool>,

    /// Whether the accordion is horizontal.
    horizontal: ReadSignal<bool>,

    /// Roving focus state, keyed by each item's registration order.
    focus: CollectionState,
}

impl AccordionContext {
    fn register_item(&mut self) -> usize {
        let mut next_index = self.next_index.write();
        let index = *next_index;
        *next_index += 1;
        index
    }

    fn is_open(&self, value: &str) -> bool {
        self.open_values.read().iter().any(|v| v == value)
    }

    fn is_disabled(&self) -> bool {
        (self.disabled)()
    }

    fn is_horizontal(&self) -> bool {
        (self.horizontal)()
    }
}

/// The props for the [`Accordion`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AccordionProps {
    /// The id of the accordion root element.
    pub id: Option<String>,

    /// The controlled open item's value (`None` means uncontrolled, using
    /// `default_value` instead — matching
    /// [`crate::menu::MenuRadioGroup`]'s identical convention for an
    /// optionally-controlled optional value).
    #[props(default)]
    pub value: Option<ReadSignal<Option<String>>>,

    /// The initially open item's value when uncontrolled.
    #[props(default)]
    pub default_value: Option<String>,

    /// Called with the newly open item's value (`None` if the open item was
    /// collapsed and `collapsible` allows that).
    #[props(default)]
    pub on_value_change: Callback<Option<String>>,

    /// Set whether the accordion is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether the open item can be collapsed, leaving nothing open.
    ///
    /// Defaults to true.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub collapsible: ReadSignal<bool>,

    /// Whether the accordion is horizontal.
    ///
    /// Settings this to true will use left/right keybinds for navigation instead of up/down. Defaults to false.
    #[props(default)]
    pub horizontal: ReadSignal<bool>,

    /// Attributes to extend the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the accordion, which should contain [`AccordionItem`] components.
    pub children: Element,
}

/// # Accordion
///
/// A single-open accordion: opening one item closes any other. For multiple
/// items open at once, see [`AccordionMulti`].
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::accordion::{
///     Accordion, AccordionContent, AccordionItem, AccordionTrigger,
/// };
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Accordion {
///             horizontal: false,
///             for i in 0..4 {
///                 AccordionItem {
///                     value: format!("item-{i}"),
///                     index: i,
///                     on_change: move |open| {
///                         tracing::info!("{open};");
///                     },
///                     on_trigger_click: move || {
///                         tracing::info!("trigger");
///                     },
///                     AccordionTrigger {
///                         "the quick brown fox"
///                     }
///                     AccordionContent {
///                         div { padding_bottom: "1rem",
///                             p {
///                                 padding: "0",
///                                 "Jumped over the lazy dog."
///                             }
///                         }
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`Accordion`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the accordion is disabled. values are `true` or `false`.
#[component]
pub fn Accordion(props: AccordionProps) -> Element {
    let mut internal_value: Signal<Option<String>> = use_signal(|| props.default_value.clone());
    let open_value = use_memo(move || match props.value {
        Some(value) => value.cloned(),
        None => internal_value.cloned(),
    });
    let open_values = use_memo(move || open_value().into_iter().collect::<Vec<_>>());

    let collapsible = props.collapsible;
    let on_value_change = props.on_value_change;
    let toggle = use_callback(move |value: String| {
        let next = if open_value().as_deref() == Some(value.as_str()) {
            if collapsible() { None } else { open_value() }
        } else {
            Some(value)
        };
        internal_value.set(next.clone());
        on_value_change.call(next);
    });

    let focus = use_collection_provider(ReadSignal::new(Signal::new(true)));
    let mut ctx = use_context_provider(|| AccordionContext {
        next_index: Signal::new(0),
        open_values,
        toggle,
        disabled: props.disabled,
        horizontal: props.horizontal,
        focus,
    });

    rsx! {
        div {
            id: props.id,
            "data-disabled": (props.disabled)(),

            onfocusout: move |_| {
                ctx.focus.clear_focus();
            },

            ..props.attributes,

            {props.children}
        }
    }
}

/// The props for the [`AccordionMulti`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AccordionMultiProps {
    /// The id of the accordion root element.
    pub id: Option<String>,

    /// The controlled set of open item values. Uncontrolled (using
    /// `default_values`) if not provided.
    #[props(default)]
    pub values: ReadSignal<Option<Vec<String>>>,

    /// The initially open item values when uncontrolled.
    #[props(default)]
    pub default_values: Vec<String>,

    /// Called with the full set of open item values whenever it changes.
    #[props(default)]
    pub on_values_change: Callback<Vec<String>>,

    /// Set whether the accordion is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether the accordion is horizontal.
    #[props(default)]
    pub horizontal: ReadSignal<bool>,

    /// Attributes to extend the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the accordion, which should contain [`AccordionItem`] components.
    pub children: Element,
}

/// # AccordionMulti
///
/// An accordion allowing more than one item open at once. For single-open
/// behavior, see [`Accordion`].
///
/// ## Styling
///
/// The [`AccordionMulti`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the accordion is disabled. values are `true` or `false`.
#[component]
pub fn AccordionMulti(props: AccordionMultiProps) -> Element {
    let mut internal_values: Signal<Vec<String>> = use_signal(|| props.default_values.clone());
    let open_values = use_memo(move || match props.values.cloned() {
        Some(values) => values,
        None => internal_values.cloned(),
    });

    let on_values_change = props.on_values_change;
    let toggle = use_callback(move |value: String| {
        let mut current = open_values();
        if let Some(pos) = current.iter().position(|v| v == &value) {
            current.remove(pos);
        } else {
            current.push(value);
        }
        internal_values.set(current.clone());
        on_values_change.call(current);
    });

    let focus = use_collection_provider(ReadSignal::new(Signal::new(true)));
    let mut ctx = use_context_provider(|| AccordionContext {
        next_index: Signal::new(0),
        open_values,
        toggle,
        disabled: props.disabled,
        horizontal: props.horizontal,
        focus,
    });

    rsx! {
        div {
            id: props.id,
            "data-disabled": (props.disabled)(),

            onfocusout: move |_| {
                ctx.focus.clear_focus();
            },

            ..props.attributes,

            {props.children}
        }
    }
}

/// The props for the [`AccordionItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AccordionItemProps {
    /// This item's identity within the enclosing [`Accordion`]/[`AccordionMulti`].
    pub value: ReadSignal<String>,

    /// Whether the accordion item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Callback for when the accordion's open/closed state changes.
    ///
    /// The new value is provided.
    #[props(default)]
    pub on_change: Callback<bool, ()>,

    /// Callback for when the trigger is clicked.
    #[props(default)]
    pub on_trigger_click: Callback,

    /// The index of the accordion item within the [`Accordion`]/[`AccordionMulti`].
    ///
    /// This is required to implement keyboard navigation and focus management.
    pub index: usize,

    /// Additional attributes to extend the item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the accordion item.
    pub children: Element,
}

/// # Accordion Item
///
/// The accordion item component represents a single item within an accordion, which can be expanded or collapsed to show or hide its content.
///
/// The [`AccordionItem`] component must be used underneath the [`Accordion`]/[`AccordionMulti`] component.
///
/// ## Styling
///
/// The [`AccordionItem`] component defines the following data attributes you can use to control styling:
/// - `data-open`: Indicates if the accordion item is open. values are `true` or `false`.
/// - `data-disabled`: Indicates if the accordion is disabled. values are `true` or `false`.
#[component]
pub fn AccordionItem(props: AccordionItemProps) -> Element {
    let mut ctx: AccordionContext = use_context();
    let aria_id = use_unique_id();

    let item = use_context_provider(|| Item {
        value: props.value,
        aria_id,
        disabled: props.disabled,
        on_trigger_click: props.on_trigger_click,
        index: ctx.register_item(),
    });

    // Handle calling `on_change` callback.
    use_effect(move || {
        let open = ctx.is_open(&item.value());
        props.on_change.call(open)
    });

    rsx! {
        div {
            "data-open": ctx.is_open(&item.value()),
            "data-disabled": ctx.is_disabled() || item.is_disabled(),
            ..props.attributes,

            {props.children}
        }
    }
}

/// The props for the [`AccordionContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AccordionContentProps {
    /// The id of the accordion content element.
    pub id: ReadSignal<Option<String>>,
    /// Additional attributes to extend the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the accordion content element.
    pub children: Element,
}

/// # Accordion Content
///
/// The accordion content component represents the content of an accordion item that can be
/// expanded or collapsed. The contents will only be displayed when the [`AccordionItem`] is open.
///
/// This must be used underneath the [`AccordionItem`] component.
///
/// ## Styling
///
/// The [`AccordionContent`] component defines the following data attributes you can use to control styling:
/// - `data-open`: Indicates if the accordion item is open. values are `true` or `false`.
#[component]
pub fn AccordionContent(props: AccordionContentProps) -> Element {
    let item: Item = use_context();
    let id = use_id_or(item.aria_id, props.id);
    let ctx: AccordionContext = use_context();
    let open = use_memo(move || ctx.is_open(&item.value()));

    let render_element = use_animated_open(id, open);

    rsx! {
        if render_element() {
            div {
                id: id,
                "data-open": open,
                ..props.attributes,

                {props.children}
            }
        }
    }
}

/// The props for the [`AccordionTrigger`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AccordionTriggerProps {
    /// THe id of the accordion trigger element.
    pub id: Option<String>,
    /// Additional attributes to extend the trigger element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the accordion trigger element.
    pub children: Element,
}

/// # Accordion Trigger
///
/// The accordion trigger component is a button that toggles the open/closed state of an [`AccordionItem`].
///
/// The [`AccordionTrigger`] component must be used underneath the [`AccordionItem`] component.
#[component]
pub fn AccordionTrigger(props: AccordionTriggerProps) -> Element {
    let mut ctx: AccordionContext = use_context();
    let item: Item = use_context();

    let disabled = move || ctx.is_disabled() || item.is_disabled();
    // The trigger is the focusable element, so it registers this accordion item.
    let index_signal = use_signal(|| item.index);
    let onmounted =
        use_item(collection_item(ctx.focus, index_signal).disabled(disabled)).onmounted();
    let open = move || ctx.is_open(&item.value());

    rsx! {
        button {
            id: props.id,
            disabled: disabled(),
            tabindex: "0",
            r#type: "button",

            aria_controls: item.aria_id(),
            aria_expanded: open(),
            "data-state": if open() { "open" } else { "closed" },

            onmounted,
            onfocus: move |_| {
                ctx.focus.set_focus(Some(item.index));
            },
            onkeydown: move |event| {
                let key = event.key();
                let horizontal = ctx.is_horizontal();
                let mut prevent_default = true;

                match key {
                    Key::ArrowUp if !horizontal => ctx.focus.focus_prev(),
                    Key::ArrowDown if !horizontal => ctx.focus.focus_next(),
                    Key::ArrowLeft if horizontal => ctx.focus.focus_prev(),
                    Key::ArrowRight if horizontal => ctx.focus.focus_next(),
                    Key::Home => ctx.focus.focus_first(),
                    Key::End => ctx.focus.focus_last(),
                    _ => prevent_default = false,
                };

                if prevent_default {
                    event.prevent_default();
                }
            },

            onclick: move |_| {
                if disabled() {
                    return;
                }
                item.on_trigger_click.call(());
                ctx.toggle.call(item.value());
            },

            ..props.attributes,

            {props.children}
        }
    }
}

/// Internal accordion-item context.
#[derive(Clone, Copy, PartialEq)]
struct Item {
    value: ReadSignal<String>,
    aria_id: Signal<String>,
    disabled: ReadSignal<bool>,
    on_trigger_click: Callback,
    index: usize,
}

impl Item {
    pub fn is_disabled(&self) -> bool {
        (self.disabled)()
    }

    pub fn aria_id(&self) -> String {
        (self.aria_id)()
    }

    pub fn value(&self) -> String {
        (self.value)()
    }
}
