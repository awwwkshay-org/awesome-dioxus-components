//! One page per currently migrated registry item, one file per page under
//! this directory. `index.rs` is the root route (`/`), named after
//! TanStack Start's file-based-routing convention for a directory's index
//! route (`routes.rs`'s `Route` enum still declares every path explicitly
//! -- dioxus-router has no file-system route generation).

mod badge;
mod button;
mod calendar;
mod card;
mod combobox;
mod context_menu;
mod date_picker;
mod dialog;
mod dropdown_menu;
mod hover_card;
mod index;
mod input;
mod item;
mod menubar;
mod pagination;
mod popover;
mod select;
mod sheet;
mod sidebar;
mod skeleton;
mod textarea;
mod tooltip;

pub use badge::BadgePage;
pub use button::ButtonPage;
pub use calendar::CalendarPage;
pub use card::CardPage;
pub use combobox::ComboboxPage;
pub use context_menu::ContextMenuPage;
pub use date_picker::DatePickerPage;
pub use dialog::DialogPage;
pub use dropdown_menu::DropdownMenuPage;
pub use hover_card::HoverCardPage;
pub use index::Home;
pub use input::InputPage;
pub use item::ItemPage;
pub use menubar::MenubarPage;
pub use pagination::PaginationPage;
pub use popover::PopoverPage;
pub use select::SelectPage;
pub use sheet::SheetPage;
pub use sidebar::SidebarPage;
pub use skeleton::SkeletonPage;
pub use textarea::TextareaPage;
pub use tooltip::TooltipPage;
