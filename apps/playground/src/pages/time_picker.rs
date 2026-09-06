use dioxus::prelude::*;
use time::Time;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;
use crate::components::ui::TimePickerView;
use crate::generated::controls::{TimePickerPopoverControls, TimePickerPopoverDemoState};

const VIEW_OPTIONS: &[(&str, TimePickerView)] = &[
    ("Digital", TimePickerView::Digital),
    ("Analog", TimePickerView::Analog),
];

#[component]
pub fn TimePickerPage() -> Element {
    let mut picked_time = use_signal(|| None::<Time>);
    let disabled = use_signal(|| false);
    let read_only = use_signal(|| false);
    let is_12h = use_signal(|| false);
    let show_seconds = use_signal(|| false);
    let view = use_signal(|| TimePickerView::Digital);
    let popover_state = use_signal(TimePickerPopoverDemoState::default);
    rsx! {
        Demo {
            name: "TimePicker",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Read only", value: read_only }
                BoolControl { label: "12-hour", value: is_12h }
                BoolControl { label: "Show seconds", value: show_seconds }
                SelectControl { label: "View", value: view, options: VIEW_OPTIONS }
                TimePickerPopoverControls { state: popover_state }
            },
            components::ui::TimePicker {
                selected_time: picked_time(),
                on_value_change: move |time| picked_time.set(time),
                disabled: disabled(),
                read_only: read_only(),
                is_12h: is_12h(),
                show_seconds: show_seconds(),
                view: view(),
                components::ui::TimePickerPopover {
                    open: popover_state().open,
                    default_open: popover_state().default_open,
                    components::ui::TimePickerInput {
                        components::ui::TimePickerInputValue {}
                        // `compact` because this trigger sits inside the field
                        // that already shows the value -- the labeled default
                        // would render the same time twice (matching
                        // `DatePickerPage`'s icon-only trigger composition).
                        components::ui::TimePickerTrigger { compact: true }
                        components::ui::TimePickerContent { components::ui::TimePickerBody {} }
                    }
                }
            }
        }
    }
}
