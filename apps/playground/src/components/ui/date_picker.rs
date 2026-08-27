//! Styled Date Picker roots composed from adico's owned date/calendar/popover
//! primitives. The primitives keep editing, range constraints, focus, Escape,
//! and ARIA behavior; the roots supply the semantic layout hook.

use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronDown;
use time::{Date, macros::date};

use super::popover::{PopoverContent, PopoverTrigger};
use crate::adico_lib::cn::cn;
use adico_primitives::calendar::DateRange;
use adico_primitives::date_picker::{
    DatePicker as PrimitiveDatePicker, DatePickerInput as PrimitiveDatePickerInput,
    DatePickerInputValue as PrimitiveDatePickerInputValue,
    DatePickerPopover as PrimitiveDatePickerPopover, DateRangePicker as PrimitiveDateRangePicker,
};
use adico_primitives::popover::{PopoverRoot, PopoverRootProps};

pub use adico_primitives::date_picker::{
    DatePickerCalendar, DatePickerDaySegment, DatePickerMonthSegment, DatePickerSeparator,
    DatePickerYearSegment, DateRangePickerCalendar, DateRangePickerEndValue, DateRangePickerInput,
    DateRangePickerInputValue, DateRangePickerStartValue,
};

/// Props for the styled single-date picker root.
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerProps {
    #[props(default)]
    pub on_value_change: Callback<Option<Date>>,
    #[props(default)]
    pub selected_date: ReadSignal<Option<Date>>,
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub read_only: ReadSignal<bool>,
    #[props(default = date!(1925-01-01))]
    pub min_date: Date,
    #[props(default = date!(2050-12-31))]
    pub max_date: Date,
    #[props(default)]
    pub disabled_ranges: ReadSignal<Vec<DateRange>>,
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub roving_loop: ReadSignal<bool>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// A styled single-date picker retaining the primitive's full interaction API.
#[component]
pub fn DatePicker(props: DatePickerProps) -> Element {
    let class = cn(&[
        "relative inline-block",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveDatePicker {
            on_value_change: props.on_value_change,
            selected_date: props.selected_date,
            disabled: props.disabled,
            read_only: props.read_only,
            min_date: props.min_date,
            max_date: props.max_date,
            disabled_ranges: props.disabled_ranges,
            roving_loop: props.roving_loop,
            class,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerPopoverProps {
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub is_modal: ReadSignal<bool>,
    pub open: ReadSignal<Option<bool>>,
    #[props(default)]
    pub default_open: bool,
    #[props(default)]
    pub on_open_change: Callback<bool>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
    #[props(default = PopoverRoot)]
    pub popover_root: fn(PopoverRootProps) -> Element,
}

/// Styled popover root. The named group lets descendants reflect open state
/// without duplicating primitive state in the registry façade.
#[component]
pub fn DatePickerPopover(props: DatePickerPopoverProps) -> Element {
    let class = cn(&[
        "group/date-picker",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveDatePickerPopover {
            is_modal: props.is_modal,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            popover_root: props.popover_root,
            class,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DatePickerInputProps {
    #[props(default = Callback::new(|_| "D".to_string()))]
    pub on_format_day_placeholder: Callback<(), String>,
    #[props(default = Callback::new(|_| "M".to_string()))]
    pub on_format_month_placeholder: Callback<(), String>,
    #[props(default = Callback::new(|_| "Y".to_string()))]
    pub on_format_year_placeholder: Callback<(), String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Option<Element>,
}

/// A compact editable date field matching the Dioxus Components composition.
#[component]
pub fn DatePickerInput(props: DatePickerInputProps) -> Element {
    let class = cn(&[
        "inline-flex h-9 items-center gap-1 rounded-md border border-input bg-background px-2 text-sm shadow-xs transition-colors focus-within:border-ring focus-within:ring-2 focus-within:ring-ring/50 data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50",
        props.class.as_deref().unwrap_or_default(),
    ]);
    let children = props.children.unwrap_or_else(|| {
        rsx! {
            DatePickerInputValue {
                on_format_day_placeholder: props.on_format_day_placeholder,
                on_format_month_placeholder: props.on_format_month_placeholder,
                on_format_year_placeholder: props.on_format_year_placeholder,
            }
            DatePickerTrigger {}
        }
    });

    rsx! {
        PrimitiveDatePickerInput {
            on_format_day_placeholder: props.on_format_day_placeholder,
            on_format_month_placeholder: props.on_format_month_placeholder,
            on_format_year_placeholder: props.on_format_year_placeholder,
            class,
            attributes: props.attributes,
            children: Some(children),
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DatePickerInputValueProps {
    #[props(default = Callback::new(|_| "D".to_string()))]
    pub on_format_day_placeholder: Callback<(), String>,
    #[props(default = Callback::new(|_| "M".to_string()))]
    pub on_format_month_placeholder: Callback<(), String>,
    #[props(default = Callback::new(|_| "Y".to_string()))]
    pub on_format_year_placeholder: Callback<(), String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub children: Option<Element>,
}

/// Editable `YYYY - MM - DD` segments; each segment keeps the primitive's
/// spinbutton keyboard semantics.
#[component]
pub fn DatePickerInputValue(props: DatePickerInputValueProps) -> Element {
    let class = cn(&[
        "flex items-center gap-1 text-sm tabular-nums text-foreground [&_[role=spinbutton]]:rounded-sm [&_[role=spinbutton]]:px-0.5 [&_[role=spinbutton]]:outline-none [&_[role=spinbutton]:focus]:bg-accent [&_[role=spinbutton][no-date=true]]:text-muted-foreground [&_[is-separator=true]]:text-muted-foreground",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span { class,
            PrimitiveDatePickerInputValue {
                on_format_day_placeholder: props.on_format_day_placeholder,
                on_format_month_placeholder: props.on_format_month_placeholder,
                on_format_year_placeholder: props.on_format_year_placeholder,
                children: props.children,
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DatePickerTriggerProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(default)]
    pub children: Option<Element>,
}

/// Disclosure trigger composed from the installed Popover façade.
#[component]
pub fn DatePickerTrigger(props: DatePickerTriggerProps) -> Element {
    let class = cn(&[
        "ml-1 size-6 rounded-sm text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        props.class.as_deref().unwrap_or_default(),
    ]);
    let children = props
        .children
        .unwrap_or_else(|| rsx! {
            ChevronDown {
                class: "size-4 shrink-0 transition-transform duration-200 group-data-[state=open]/date-picker:rotate-180",
                size: 16,
            }
            span { class: "sr-only", "Toggle calendar" }
        });
    rsx! { PopoverTrigger { class, {children} } }
}

#[derive(Props, Clone, PartialEq)]
pub struct DatePickerContentProps {
    #[props(default)]
    pub class: Option<String>,
    pub children: Element,
}

/// Popup shell with no duplicate frame; the Calendar view owns the surface.
#[component]
pub fn DatePickerContent(props: DatePickerContentProps) -> Element {
    let class = cn(&[
        "adico-date-picker-popover z-[1000] w-auto border-0 bg-transparent p-0 shadow-none",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! { PopoverContent { class, {props.children} } }
}

/// Props for the styled date-range picker root.
#[derive(Props, Clone, PartialEq)]
pub struct DateRangePickerProps {
    #[props(default)]
    pub on_range_change: Callback<Option<DateRange>>,
    #[props(default)]
    pub selected_range: ReadSignal<Option<DateRange>>,
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub read_only: ReadSignal<bool>,
    #[props(default = date!(1925-01-01))]
    pub min_date: Date,
    #[props(default = date!(2050-12-31))]
    pub max_date: Date,
    #[props(default)]
    pub disabled_ranges: ReadSignal<Vec<DateRange>>,
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub roving_loop: ReadSignal<bool>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// A styled range picker retaining typed range selection and constraints.
#[component]
pub fn DateRangePicker(props: DateRangePickerProps) -> Element {
    let class = cn(&[
        "relative inline-block",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveDateRangePicker {
            on_range_change: props.on_range_change,
            selected_range: props.selected_range,
            disabled: props.disabled,
            read_only: props.read_only,
            min_date: props.min_date,
            max_date: props.max_date,
            disabled_ranges: props.disabled_ranges,
            roving_loop: props.roving_loop,
            class,
            attributes: props.attributes,
            {props.children}
        }
    }
}
