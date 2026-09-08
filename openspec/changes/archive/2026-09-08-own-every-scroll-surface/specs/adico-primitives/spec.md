## ADDED Requirements

### Requirement: The scroll-area primitive exposes a mergeable, reusable scrollbar-styling contract
`ScrollArea` SHALL accept a caller-supplied `class` that is merged with, not replacing,
its internal visibility class, in both server-rendered and client-rendered output.
Scrollable regions across `adico-primitives` and `registry/ui` SHALL adopt this shared
contract instead of setting `overflow` directly on their own element, except for elements
that clip rather than scroll, native editing surfaces (`<textarea>`, contenteditable),
page/document-level scroll, and consumer-supplied virtualization boxes.

#### Scenario: A caller passes a custom class to ScrollArea
- **WHEN** a consumer renders `ScrollArea` with a `class` prop
- **THEN** the rendered element's `class` attribute contains both adico's internal
  visibility class and the caller's class, identically in server-rendered and
  client-rendered output

#### Scenario: A primitive-owned scroll element adopts the contract in place
- **WHEN** a component's own element already carries an ARIA role or existing scroll
  instrumentation (for example `SelectList`, `CommandList`, or `MessageScrollerViewport`)
- **THEN** the scroll-area styling contract is applied to that existing element rather
  than introducing a wrapping element that would change its structural position

#### Scenario: A non-scrolling clipping container is excluded
- **WHEN** an element uses `overflow: hidden` purely to clip content (for example
  rounded-corner or animation masking) rather than to scroll
- **THEN** it is not required to adopt the scroll-area contract

### Requirement: ScrollArea renders overlay scrollbars over a real native-overflow viewport
`ScrollArea` SHALL render its overlay scrollbar parts (viewport, scrollbar, thumb, corner)
such that the viewport remains a genuine native-overflow element receiving real browser
scroll events, and SHALL NOT use a `transform` to implement scrolling.

#### Scenario: An anchored popover continues tracking scroll
- **WHEN** a scroll-area viewport that has adopted the shared contract is scrolled while
  an anchored popover elsewhere on the page is open
- **THEN** the popover's position-tracking mechanism continues to receive native scroll
  events exactly as it did before adoption

#### Scenario: Server-rendered output has no measurement-dependent flash
- **WHEN** a page containing a `ScrollArea` is server-rendered and then hydrated in the
  browser
- **THEN** the overlay scrollbar thumb renders hidden until its first client-side
  measurement, producing no visible flash or jump between server output and hydrated
  output

### Requirement: ScrollArea's scrollbar-visibility props have observable, distinct effect
`ScrollType::Hidden` SHALL suppress the native scrollbar via an actual CSS declaration,
and `always_show_scrollbars` SHALL produce visually distinct styling from the default
auto-hide behavior.

#### Scenario: ScrollType::Hidden hides the native scrollbar
- **WHEN** a `ScrollArea` is rendered with `scroll_type: ScrollType::Hidden`
- **THEN** its markup contains a CSS style declaration suppressing the native scrollbar,
  not merely an inert HTML attribute of the same name

#### Scenario: always_show_scrollbars differs from the default
- **WHEN** a `ScrollArea` is rendered with `always_show_scrollbars: true` versus its
  default
- **THEN** the two renders apply distinguishably different CSS rather than identical
  rules under different class names

### Requirement: Keyboard navigation scrolls the active item into view inside a scrollable listbox or menu
Primitives that track an active or highlighted item inside a scrollable region
(listbox, select, combobox, command, menu, typeahead) SHALL scroll that item into view
when it changes via keyboard navigation and falls outside the currently visible area.

#### Scenario: Arrow-key navigation past the visible bottom of a long list
- **WHEN** a user presses the down-arrow key repeatedly in an open Select, Combobox, or
  Command list until the active option would fall below the visible viewport
- **THEN** the scroll container scrolls so the active option remains fully visible

#### Scenario: A short list needs no scrolling
- **WHEN** every item already fits within the scroll container's visible area
- **THEN** keyboard navigation does not trigger any scroll adjustment
