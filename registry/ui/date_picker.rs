//! Source-owned Date Picker composition backed by adico's audited primitive
//! layer, composing the owned `calendar` and `popover` primitives.
//!
//! This initial registry façade intentionally preserves the primitive's
//! compositional Dioxus API, matching `calendar`'s own precedent: consumers
//! install and own this module and apply Tailwind classes directly to each
//! part while the runtime retains keyboard, focus, and ARIA behavior; full
//! default styling is deferred to M4 parity hardening.

pub use adico_primitives::date_picker::{
    DatePicker, DatePickerCalendar, DatePickerDaySegment, DatePickerInput, DatePickerInputValue,
    DatePickerMonthSegment, DatePickerPopover, DatePickerSeparator, DatePickerYearSegment,
    DateRangePicker, DateRangePickerCalendar, DateRangePickerEndValue, DateRangePickerInput,
    DateRangePickerInputValue, DateRangePickerStartValue,
};
