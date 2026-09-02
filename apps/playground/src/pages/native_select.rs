use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn NativeSelectPage() -> Element {
    let mut size = use_signal(|| components::ui::NativeSelectSize::Default);
    rsx! {
        Demo { name: "NativeSelect",
            controls: rsx! {
                SelectControl {
                    label: "Size",
                    value: size(),
                    options: crate::generated::controls::NATIVE_SELECT_SIZE_OPTIONS.to_vec(),
                    on_change: move |value| size.set(value),
                }
            },
            div { class: "flex flex-col gap-4",
                components::ui::NativeSelect {
                    size: size(),
                    components::ui::NativeSelectOption { value: "apple", "Apple" }
                    components::ui::NativeSelectOption { value: "banana", "Banana" }
                    components::ui::NativeSelectOption { value: "cherry", "Cherry" }
                }
                components::ui::NativeSelect {
                    size: components::ui::NativeSelectSize::Sm,
                    components::ui::NativeSelectOptGroup { label: "Fruits",
                        components::ui::NativeSelectOption { value: "apple", "Apple" }
                        components::ui::NativeSelectOption { value: "banana", "Banana" }
                    }
                    components::ui::NativeSelectOptGroup { label: "Vegetables",
                        components::ui::NativeSelectOption { value: "carrot", "Carrot" }
                        components::ui::NativeSelectOption { value: "pea", "Pea" }
                    }
                }
                components::ui::NativeSelect {
                    disabled: true,
                    components::ui::NativeSelectOption { value: "disabled", "Disabled" }
                }
            }
        }
    }
}
