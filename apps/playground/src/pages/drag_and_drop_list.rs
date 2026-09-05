use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{DragAndDropListItemsControls, DragAndDropListItemsDemoState};

#[component]
pub fn DragAndDropListPage() -> Element {
    // `DragAndDropList` renders `DragAndDropInstructions` +
    // `DragAndDropListItems` + `DragAndDropLiveRegion` as its default
    // children when none are given (see its own doc comment); passing that
    // same composition explicitly lets `DragAndDropListItems`'s own
    // `aria_label` be driven by the generated panel. Its sibling generated
    // states, `DragAndDropListItemDemoState`/`DragAndDropDropIndicatorDemoState`,
    // aren't wired here -- both are per-item `index` values intrinsic to each
    // sortable row's position, not a free-standing knob a demo can bind one
    // shared control to.
    let state = use_signal(|| DragAndDropListItemsDemoState {
        aria_label: "Reorderable items".to_string(),
    });
    let items = ["Alpha", "Bravo", "Charlie"].map(|t| rsx! { {t} }).to_vec();
    rsx! {
        Demo {
            name: "DragAndDropList",
            controls: rsx! {
                DragAndDropListItemsControls { state }
            },
            components::ui::DragAndDropList { items,
                components::ui::DragAndDropInstructions {}
                components::ui::DragAndDropListItems { aria_label: state().aria_label }
                components::ui::DragAndDropLiveRegion {}
            }
        }
    }
}
