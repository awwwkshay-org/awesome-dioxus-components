//! Source-owned Calendar composition backed by adico's audited primitive layer.
//!
//! This initial registry façade intentionally preserves the primitive's
//! compositional Dioxus API, matching the approach already used by
//! `select`/`dropdown-menu`/`combobox`: with 20+ composable parts, applying
//! a full Tailwind restyle up front would be disproportionate to this
//! migration pass. Consumers install and own this module and apply Tailwind
//! classes directly to each part while the runtime retains keyboard
//! navigation, focus, and ARIA behavior; full default styling is deferred to
//! M4 parity hardening.

pub use adico_primitives::calendar::{
    Calendar, CalendarDay, CalendarGrid, CalendarGridBody, CalendarGridCell, CalendarGridDayHeader,
    CalendarGridHead, CalendarGridHeaderRow, CalendarGridRoot, CalendarGridWeek, CalendarHeader,
    CalendarMonthTitle, CalendarNavigation, CalendarNextMonthButton, CalendarPreviousMonthButton,
    CalendarSelectMonth, CalendarSelectMonthOption, CalendarSelectMonthSelect,
    CalendarSelectMonthValue, CalendarSelectYear, CalendarSelectYearOption,
    CalendarSelectYearSelect, CalendarSelectYearValue, CalendarView, RangeCalendar,
};
