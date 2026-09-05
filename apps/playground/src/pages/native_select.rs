use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{NativeSelectControls, NativeSelectDemoState};

#[component]
pub fn NativeSelectPage() -> Element {
    let state = use_signal(NativeSelectDemoState::default);
    rsx! {
        Demo { name: "NativeSelect",
            controls: rsx! {
                NativeSelectControls { state }
            },
            div { class: "flex flex-col gap-4",
                components::ui::NativeSelect {
                    size: state().size,
                    disabled: state().disabled,
                    required: state().required,
                    invalid: state().invalid,
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
