//! Black-box tests for `adico_primitives::listbox`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/listbox.rs`.
//!
//! Most of this module is thin `use_effect` glue already exercised indirectly, extensively,
//! through `select.rs`'s and `combobox.rs`'s own tests (both consume `use_listbox_container`/
//! `use_listbox_id`/`use_listbox_option`). `ListboxItemIndicator`'s conditional render is the
//! one piece with no such indirect coverage.

use adico_primitives::listbox::{ListboxItemIndicator, ListboxOptionContext};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn SelectedIndicator() -> Element {
    use_context_provider(|| ListboxOptionContext {
        selected: ReadSignal::new(Signal::new(true)),
    });
    rsx! {
        ListboxItemIndicator { "check" }
    }
}

#[test]
fn indicator_renders_its_children_when_the_option_is_selected() {
    let html = render(SelectedIndicator);
    assert!(html.contains("check"), "{html}");
}

#[component]
fn UnselectedIndicator() -> Element {
    use_context_provider(|| ListboxOptionContext {
        selected: ReadSignal::new(Signal::new(false)),
    });
    rsx! {
        ListboxItemIndicator { "check" }
    }
}

#[test]
fn indicator_renders_nothing_when_the_option_is_not_selected() {
    let html = render(UnselectedIndicator);
    assert!(!html.contains("check"), "{html}");
}
