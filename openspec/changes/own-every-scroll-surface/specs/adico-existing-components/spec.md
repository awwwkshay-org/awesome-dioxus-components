## ADDED Requirements

### Requirement: Registry components with a scrollable region compose the shared scroll-area contract
Select, Combobox, Command, Sidebar, Table, Resizable, TimePicker, and MessageScroller
SHALL render their scrollable regions through the shared scroll-area contract (the
`adico-primitives` `ScrollArea`/overlay parts specified in `adico-primitives`) rather than
a bare Tailwind `overflow-*` class. Carousel SHALL remain excluded from this requirement:
its slide tracks combine `scroll-snap-type` with a pointer-drag gesture, and converting
them risks regressing both drag and snap behavior for a purely cosmetic gain.

#### Scenario: A themed scrollbar appears wherever content overflows
- **WHEN** Select's option list, Combobox's option list, Command's list, Sidebar's content
  region, Table's horizontal scroller, a Resizable panel, TimePicker's column, or
  MessageScroller's viewport contains more content than its visible area
- **THEN** it renders the shared themed overlay scrollbar rather than the browser's
  default scrollbar

#### Scenario: Carousel keeps native scrolling
- **WHEN** a Carousel's slide track overflows its visible area
- **THEN** it continues using native browser scrolling, not the shared scroll-area
  contract

### Requirement: Anchored menu surfaces cap their height and scroll rather than clipping
DropdownMenu, ContextMenu, Menubar, NavigationMenu, Popover, and HoverCard content SHALL
cap their height to the space actually available between their anchor and the viewport
edge, and SHALL scroll rather than clip or silently overflow the viewport when their
content exceeds that height.

#### Scenario: Menu content exceeds available space
- **WHEN** an open DropdownMenu, ContextMenu, Menubar, NavigationMenu, Popover, or
  HoverCard's content is taller than the space available on its placed side
- **THEN** the content scrolls within a height-capped container instead of being clipped
  or extending past the viewport

#### Scenario: A submenu opens from inside a scrolled menu
- **WHEN** a DropdownMenu submenu is opened from an item inside an already-scrolled parent
  menu
- **THEN** the submenu renders fully visible, not clipped by its newly-scrolling parent

### Requirement: Modal and drawer surfaces scroll their body rather than clipping
Drawer, Dialog, AlertDialog, and Sheet SHALL cap their content height and scroll their
body when content exceeds the visible viewport, rather than clipping content or requiring
caller-side workarounds.

#### Scenario: Dialog content exceeds the viewport
- **WHEN** a Dialog, AlertDialog, Sheet, or Drawer's content is taller than the available
  viewport height
- **THEN** its body scrolls within a height-capped container instead of clipping or
  overflowing the viewport

#### Scenario: A caller no longer needs its own scroll workaround
- **WHEN** a consumer previously added its own `overflow-y-auto`/`max-h-*` classes to a
  `DialogContent` to work around missing built-in scrolling
- **THEN** that workaround is no longer necessary, since `DialogContent` scrolls by
  default
