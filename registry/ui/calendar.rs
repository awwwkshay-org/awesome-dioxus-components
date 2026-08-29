//! Source-owned shadcn-style Calendar composition for Dioxus.
//!
//! Selection, range state, month navigation, keyboard focus, and ARIA remain
//! owned by `adico-primitives`. These wrappers add a semantic visual façade to
//! the primitive parts while preserving their native Dioxus composition API.

use adico_primitives::icons::ChevronDown;
use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use adico_primitives::calendar::{
    CalendarGrid as PrimitiveCalendarGrid, CalendarHeader as PrimitiveCalendarHeader,
    CalendarMonthTitle as PrimitiveCalendarMonthTitle,
    CalendarNavigation as PrimitiveCalendarNavigation,
    CalendarNextMonthButton as PrimitiveCalendarNextMonthButton,
    CalendarPreviousMonthButton as PrimitiveCalendarPreviousMonthButton,
    CalendarSelectMonth as PrimitiveCalendarSelectMonth,
    CalendarSelectMonthSelect as PrimitiveCalendarSelectMonthSelect,
    CalendarSelectMonthValue as PrimitiveCalendarSelectMonthValue,
    CalendarSelectYear as PrimitiveCalendarSelectYear,
    CalendarSelectYearSelect as PrimitiveCalendarSelectYearSelect,
    CalendarSelectYearValue as PrimitiveCalendarSelectYearValue,
    CalendarView as PrimitiveCalendarView,
};

pub use adico_primitives::calendar::{
    Calendar, CalendarDay, CalendarGridBody, CalendarGridCell, CalendarGridDayHeader,
    CalendarGridHead, CalendarGridHeaderRow, CalendarGridRoot, CalendarGridWeek,
    CalendarSelectMonthOption, CalendarSelectYearOption, RangeCalendar,
};

#[derive(Props, Clone, PartialEq)]
pub struct CalendarViewProps {
    #[props(default)]
    pub offset: Option<u8>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

/// A styled month panel. Render one view for each visible month.
#[component]
pub fn CalendarView(props: CalendarViewProps) -> Element {
    let class = cn(&[
        "h-[20rem] w-[18rem] rounded-md border bg-popover p-3 text-popover-foreground shadow-sm",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarView {
            offset: props.offset,
            class,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CalendarHeaderProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn CalendarHeader(props: CalendarHeaderProps) -> Element {
    let class = cn(&["relative mb-2", props.class.as_deref().unwrap_or_default()]);
    rsx! {
        PrimitiveCalendarHeader {
            id: props.id,
            class,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CalendarNavigationProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

#[component]
pub fn CalendarNavigation(props: CalendarNavigationProps) -> Element {
    let class = cn(&[
        "flex h-8 items-center justify-between gap-2",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarNavigation { class, attributes: props.attributes, {props.children} }
    }
}

const NAV_BUTTON_CLASSES: &str = "inline-flex size-8 items-center justify-center rounded-md border border-input bg-background text-muted-foreground shadow-xs transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50";

#[derive(Props, Clone, PartialEq)]
pub struct CalendarNavigationButtonProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,
    pub children: Element,
}

#[component]
pub fn CalendarPreviousMonthButton(props: CalendarNavigationButtonProps) -> Element {
    let class = cn(&[
        NAV_BUTTON_CLASSES,
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarPreviousMonthButton { class, attributes: props.attributes, {props.children} }
    }
}

#[component]
pub fn CalendarNextMonthButton(props: CalendarNavigationButtonProps) -> Element {
    let class = cn(&[
        NAV_BUTTON_CLASSES,
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarNextMonthButton { class, attributes: props.attributes, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CalendarMonthTitleProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn CalendarMonthTitle(props: CalendarMonthTitleProps) -> Element {
    let class = cn(&[
        "text-sm font-medium",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarMonthTitle { class, attributes: props.attributes }
    }
}

// Fixed percentage widths (of the flex-1 group in `CalendarNavigation` that
// holds both selects) rather than content-driven auto widths. Month/year
// names are variable-length ("May" vs. "September"); an auto-width pill
// resizes every time the value changes, shoving its neighbor sideways. A
// fixed share is stable across every possible value, so picking a new
// month/year never shifts anything else in the row.
const SELECT_TRIGGER_CLASSES: &str = "pointer-events-none flex h-8 w-full min-w-0 items-center justify-center gap-1 truncate rounded-md px-2 text-sm font-medium";
const SELECT_NATIVE_CLASSES: &str =
    "absolute inset-0 h-full w-full cursor-pointer appearance-none opacity-0";

#[derive(Props, Clone, PartialEq)]
pub struct CalendarSelectContainerProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    #[props(default)]
    pub children: Element,
}

/// Container pairing an invisible native `<select>` with a styled value
/// display; clicking anywhere in the pill opens the native dropdown.
///
/// Fixed at 60% of the enclosing flex group (see `SELECT_TRIGGER_CLASSES`)
/// so its width never depends on which month is selected.
#[component]
pub fn CalendarSelectMonth(props: CalendarSelectContainerProps) -> Element {
    let class = cn(&[
        "relative flex w-[60%] min-w-0 shrink-0 grow-0 overflow-hidden rounded-md hover:bg-accent has-disabled:pointer-events-none has-disabled:opacity-50 has-disabled:hover:bg-transparent",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarSelectMonth { class, attributes: props.attributes, {props.children} }
    }
}

/// Fixed at 40% of the enclosing flex group; see [`CalendarSelectMonth`].
#[component]
pub fn CalendarSelectYear(props: CalendarSelectContainerProps) -> Element {
    let class = cn(&[
        "relative flex w-[40%] min-w-0 shrink-0 grow-0 overflow-hidden rounded-md hover:bg-accent has-disabled:pointer-events-none has-disabled:opacity-50 has-disabled:hover:bg-transparent",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarSelectYear { class, attributes: props.attributes, {props.children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CalendarSelectFieldProps {
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn CalendarSelectMonthSelect(props: CalendarSelectFieldProps) -> Element {
    let class = cn(&[
        SELECT_NATIVE_CLASSES,
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarSelectMonthSelect { class, attributes: props.attributes }
    }
}

#[component]
pub fn CalendarSelectYearSelect(props: CalendarSelectFieldProps) -> Element {
    let class = cn(&[
        SELECT_NATIVE_CLASSES,
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarSelectYearSelect { class, attributes: props.attributes }
    }
}

#[component]
pub fn CalendarSelectMonthValue(props: CalendarSelectFieldProps) -> Element {
    let class = cn(&[
        SELECT_TRIGGER_CLASSES,
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarSelectMonthValue { class, attributes: props.attributes,
            ChevronDown { class: "size-3.5 text-muted-foreground", size: 14 }
        }
    }
}

#[component]
pub fn CalendarSelectYearValue(props: CalendarSelectFieldProps) -> Element {
    let class = cn(&[
        SELECT_TRIGGER_CLASSES,
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarSelectYearValue { class, attributes: props.attributes,
            ChevronDown { class: "size-3.5 text-muted-foreground", size: 14 }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CalendarGridProps {
    #[props(default)]
    pub id: Option<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The styled date grid. Descendant selectors cover the semantic cells and
/// buttons generated by the primitive without duplicating its date behavior.
#[component]
pub fn CalendarGrid(props: CalendarGridProps) -> Element {
    let class = cn(&[
        "w-full table-fixed border-collapse text-sm [&_tr]:h-9 [&_th]:h-8 [&_th]:font-normal [&_th]:text-muted-foreground [&_td]:p-0.5 [&_td]:text-center [&_button]:inline-flex [&_button]:size-8 [&_button]:items-center [&_button]:justify-center [&_button]:rounded-md [&_button]:bg-transparent [&_button]:outline-none [&_button]:transition-colors [&_button:hover:not([data-disabled='true'])]:bg-accent [&_button:hover:not([data-disabled='true'])]:text-accent-foreground [&_button:focus-visible]:ring-2 [&_button:focus-visible]:ring-ring [&_button[data-selected='true']]:bg-primary [&_button[data-selected='true']]:text-primary-foreground [&_button[data-today='true']:not([data-selected='true'])]:ring-1 [&_button[data-today='true']:not([data-selected='true'])]:ring-border [&_button[data-month='last']]:text-muted-foreground [&_button[data-month='last']]:opacity-50 [&_button[data-month='next']]:text-muted-foreground [&_button[data-month='next']]:opacity-50 [&_button[data-disabled='true']]:pointer-events-none [&_button[data-disabled='true']]:opacity-50",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveCalendarGrid { id: props.id, class, attributes: props.attributes }
    }
}
