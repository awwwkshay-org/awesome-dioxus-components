// Implements the WAI-ARIA APG Listbox pattern's popup-mounting/id/registration wiring shared
// by `select.rs`/`combobox.rs`, which already derived their own ARIA-role specs against this
// pattern (tasks 2.1/2.2) — this file is the thin `use_effect` glue connecting their anchored
// popup rendering (`use_animated_open` presence gating) and option registration
// (`selection.rs`'s `sync_option`/`remove_option`) to a stable id. Its own logic is exercised
// indirectly, extensively, through `select.rs`'s/`combobox.rs`'s tests (both consume every
// hook here); `ListboxItemIndicator`'s conditional render is the one piece with no such
// indirect coverage, tested directly instead.

//! Shared listbox popup hooks.

use dioxus::prelude::*;

use crate::{
    selectable::SelectableContext,
    selection::{OptionState, RcPartialEqValue, option_text_value, remove_option, sync_option},
    use_animated_open, use_effect, use_effect_cleanup, use_id_or, use_unique_id,
};

#[derive(Clone, Copy)]
pub struct ListboxContext {
    pub render: ReadSignal<bool>,
}

#[derive(Clone, Copy)]
pub struct ListboxOptionContext {
    pub selected: ReadSignal<bool>,
}

pub struct ListboxState {
    pub id: Memo<String>,
    pub render: Memo<bool>,
}

pub fn use_listbox_id(
    id: ReadSignal<Option<String>>,
    mut list_id: Signal<Option<String>>,
) -> Memo<String> {
    let generated_id = use_unique_id();
    let id = use_id_or(generated_id, id);

    use_effect(move || {
        list_id.set(Some(id()));
    });

    id
}

pub fn use_listbox_render(
    id: impl Readable<Target = String> + Copy + 'static,
    open: impl Readable<Target = bool> + Copy + 'static,
) -> Memo<bool> {
    let render = use_animated_open(id, open);
    use_memo(render)
}

pub fn use_listbox_container(
    id: ReadSignal<Option<String>>,
    mut selectable: SelectableContext,
) -> ListboxState {
    let id = use_listbox_id(id, selectable.list_id);
    let render = use_listbox_render(id, selectable.open);

    use_context_provider(|| ListboxContext {
        render: render.into(),
    });

    use_effect(move || {
        if !render.cloned() {
            selectable.initial_focus.set(None);
            return;
        }

        if let Some(index) = selectable.initial_focus.cloned() {
            selectable.collection.set_focus(Some(index));
            selectable.initial_focus.set(None);
        }
    });

    ListboxState { id, render }
}

pub fn use_listbox_option<T: Clone + PartialEq + 'static>(
    id: ReadSignal<Option<String>>,
    index: ReadSignal<usize>,
    value: ReadSignal<T>,
    text_value: ReadSignal<Option<String>>,
    options: Signal<Vec<OptionState>>,
    component_name: &'static str,
) -> Memo<String> {
    let generated_id = use_unique_id();
    let id = use_id_or(generated_id, id);
    let mut previous_id: Signal<Option<String>> = use_signal(|| None);
    let text_value =
        use_memo(move || option_text_value(&*value.read(), text_value(), component_name));

    use_effect(move || {
        let option_id = id();
        let option_index = index();
        let stale_id = previous_id.peek().clone();
        if let Some(stale_id) = stale_id
            && stale_id != option_id
        {
            remove_option(options, &stale_id);
        }
        sync_option(
            options,
            OptionState {
                id: option_id.clone(),
                index: option_index,
                value: RcPartialEqValue::new(value.cloned()),
                text_value: text_value.cloned(),
            },
        );
        previous_id.set(Some(option_id));
    });

    use_effect_cleanup(move || {
        if let Some(option_id) = previous_id.peek().as_deref() {
            remove_option(options, option_id);
        }
    });

    id
}

#[component]
pub fn ListboxItemIndicator(children: Element) -> Element {
    let ctx: ListboxOptionContext = use_context();
    if !(ctx.selected)() {
        return rsx! {};
    }
    rsx! {
        {children}
    }
}
