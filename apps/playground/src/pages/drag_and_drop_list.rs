use adico_primitives::icons::GripVertical;
use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{DragAndDropListItemsControls, DragAndDropListItemsDemoState};

/// Composes the styled `DragAndDropListItem`/`DragAndDropDropIndicator`
/// parts explicitly (with a drag-handle glyph per row) instead of relying
/// on `DragAndDropListItems`'s default children, which fall back to the
/// *unstyled* primitive parts — see this file's own history for why the
/// bare-text rendering happened.
#[component]
pub fn DragAndDropListPage() -> Element {
    let state = use_signal(|| DragAndDropListItemsDemoState {
        aria_label: "Reorderable items".to_string(),
    });
    let items = ["Alpha", "Bravo", "Charlie", "Delta"]
        .map(|t| rsx! { {t} })
        .to_vec();
    rsx! {
        Demo {
            name: "DragAndDropList",
            controls: rsx! {
                DragAndDropListItemsControls { state }
            },
            components::ui::DragAndDropList { items, class: "w-full max-w-sm",
                components::ui::DragAndDropInstructions {}
                components::ui::DragAndDropListItems { aria_label: state().aria_label,
                    for item in components::ui::use_drag_and_drop_list_items() {
                        Fragment { key: "{item.key}",
                            components::ui::DragAndDropDropIndicator { index: item.index, position: "before" }
                            components::ui::DragAndDropListItem { index: item.index, item_key: item.key.clone(),
                                GripVertical { class: "size-4 shrink-0 cursor-grab text-muted-foreground" }
                                {item.children}
                            }
                            components::ui::DragAndDropDropIndicator { index: item.index, position: "after" }
                        }
                    }
                }
                components::ui::DragAndDropLiveRegion {}
            }
        }
    }
}
