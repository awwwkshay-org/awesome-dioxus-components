// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/combobox/components/list.rs. See provenance/records/adico-primitives-wave4-collection.json.

//! ComboboxList component.

use dioxus::prelude::*;

use super::super::context::ComboboxContext;
use crate::listbox::use_listbox_container;

/// Props for [`ComboboxList`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxListProps {
    /// Optional id for the list element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children, typically [`ComboboxOption`](super::option::ComboboxOption)s
    /// and an optional [`ComboboxEmpty`](super::empty::ComboboxEmpty).
    pub children: Element,
}

/// Listbox that contains the visible options.
#[component]
pub fn ComboboxList(props: ComboboxListProps) -> Element {
    let ctx = use_context::<ComboboxContext>();
    let open = ctx.selectable.open;
    let listbox = use_listbox_container(props.id, ctx.selectable);
    let render = listbox.render;

    rsx! {
        if render() {
            div {
                id: listbox.id,
                role: "listbox",
                "data-state": if open() { "open" } else { "closed" },
                onpointerdown: move |event| {
                    event.prevent_default();
                },
                ..props.attributes,
                {props.children}
            }
        } else {
            {props.children}
        }
    }
}
