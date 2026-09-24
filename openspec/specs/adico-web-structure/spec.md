## Purpose
Serve adico's public landing page, documentation, and component playground
as one Dioxus app (`apps/web`) under one router, so a visitor moves between
`/`, `/docs`, and `/playground` without crossing an app boundary, while
keeping the same governing conventions the pre-merge apps already followed:
a dedicated routing module, CLI-installed component source, one page per
file, and a shell composed from real registry components rather than
app-specific reimplementations.

## Requirements

### Requirement: Router definitions live in routes.rs
`apps/web/src/routes.rs` SHALL define every `Route` enum variant and every
layout-shell component (`SiteLayout` wrapping the whole site,
`PlaygroundLayout` wrapping the `/playground/*` subtree). `apps/web/src/main.rs`
SHALL NOT define any `Route` enum variant, navigation list, or layout-shell
component directly; it retains only the application entrypoint, its asset
consts, the document head, the `Router` mount, and the CLI-managed
`adico:start`/`adico:end` module block.

#### Scenario: main.rs is read for routing logic
- **WHEN** a maintainer opens `apps/web/src/main.rs` looking for how routes,
  navigation, or layouts are defined
- **THEN** it contains no `#[derive(Routable)]` enum, no route-list
  function, and no layout-shell component — those live in `routes.rs`

#### Scenario: A route is added or removed
- **WHEN** a page is added to or removed from any section of the site
- **THEN** the corresponding `#[route(...)]` variant (and, for a playground
  page, its `nav_items()` entry) is edited in `routes.rs`, not in `main.rs`
  or scattered across page files

### Requirement: One page per file under pages/, named by route segment
Each routed page component SHALL live in its own file under
`apps/web/src/pages/`, aggregated by `pages/mod.rs`. The root route (`/`)
SHALL use the file name `pages/index.rs`. Playground pages SHALL live under
`pages/playground/`, named after their route's path segment (snake_case,
e.g. `pages/playground/hover_card.rs` for `/playground/hover-card`).
Documentation pages SHALL live under `pages/docs/`. No single file under
`pages/` SHALL define more than one page component.

#### Scenario: A new playground component page is added
- **WHEN** a maintainer adds a playground page for a newly-installed
  registry component
- **THEN** they create exactly one new file under `pages/playground/` named
  after the route's path segment, add its `pub mod`/re-export to
  `pages/playground/mod.rs`, and add its `#[route(...)]` variant to
  `routes.rs` — no existing page file is edited to make room

#### Scenario: The root route is located
- **WHEN** a maintainer looks for the component rendered at `/`
- **THEN** it is defined in `apps/web/src/pages/index.rs`

#### Scenario: pages.rs is searched for
- **WHEN** a maintainer or tool searches the repository for
  `apps/web/src/pages.rs`
- **THEN** no such file exists — page components live under the `pages/`
  directory tree established by this requirement

### Requirement: dioxus-router routes stay explicitly declared
Because dioxus-router has no file-system-based route generation, the
`pages/` file-naming convention SHALL NOT be treated as automatically
producing routes. Every route SHALL remain an explicit `#[route(...)]`
variant in `routes.rs`'s `Route` enum, kept in sync by hand with the files
under `pages/`. A generated `<Component>DemoState`/`<Component>Controls`/
`<Component>Preview` triad under
`apps/web/src/generated/controls/` likewise SHALL NOT be treated as
automatically producing a page or route — a page still explicitly wires
the generated panel in, and its route remains an explicit `routes.rs`
entry.

#### Scenario: A page file exists with no matching route
- **WHEN** a file exists under `pages/` whose page component has no
  corresponding `#[route(...)]` variant in `routes.rs`
- **THEN** this is a defect to fix by adding the missing route, not
  evidence that routes are derived from the file tree

#### Scenario: A generated control panel exists with no matching page
- **WHEN** `apps/web/src/generated/controls/` contains a
  `<Component>DemoState`/`<Component>Controls` pair for a component with
  no corresponding page under `pages/playground/`
- **THEN** this is not evidence that a page or route exists — a page must
  still be explicitly created under `pages/playground/` and registered in
  `routes.rs` to actually use the generated panel

### Requirement: Shell-free harness routes stay outside every layout
`/responsive/flow` and `/responsive/overlay` (with its `case` query
parameter) SHALL render outside both `SiteLayout` and `PlaygroundLayout` —
not merely outside the innermost shell — so the automated viewport test
harness measures each fixture's real, full-bleed geometry with no site
chrome affecting the measurement. These routes are additive: they SHALL NOT
change `SiteLayout`'s or `PlaygroundLayout`'s own behavior, and they are not
part of the site's normal browsing navigation.

#### Scenario: The viewport harness measures a component's real geometry
- **WHEN** the narrow-viewport test harness needs to measure a registry
  component's layout at 375px without any site or playground shell chrome
  affecting the measurement
- **THEN** it loads a shell-free route that renders that component directly
  at the browser's actual viewport width, with no header, nav, or footer
  present in the rendered output

#### Scenario: A shell-free route needs fixture content the demo page doesn't provide
- **WHEN** a component's existing playground demo page uses fixture content
  too minimal to exercise a real-world layout failure (for example, a Tabs
  demo with only two tabs, which cannot reproduce horizontal overflow with
  five)
- **THEN** the shell-free harness route renders that component with
  separately authored, realistic fixture content, rather than reusing the
  minimal demo fixture

### Requirement: apps/web is initialized and kept current through the real adico CLI
`apps/web` SHALL be initialized and kept current through the real `adico`
CLI (`adico init`, `adico add`) rather than hand-edited component source.
Its `components.json`, `adico.lock`, `src/components/ui/*`,
`src/adico_lib/*`, and the `adico:theme:start`/`adico:theme:end` block of
`tailwind.css` SHALL be produced by those commands, never authored or
edited by hand, consistent with CLAUDE.md's architecture rule that a
consumer-style app must exercise the CLI installation path rather than
import `registry/` source through a workspace path.

#### Scenario: apps/web is refreshed
- **WHEN** the CLI installer's public commands are re-run against
  `apps/web`
- **THEN** the app's installed component source, `adico.lock`, and
  `components.json` are produced by those commands, not manual edits

#### Scenario: A maintainer searches for a workspace-path import of registry/
- **WHEN** a maintainer or tool searches `apps/web/src/**` for a `use`
  statement or path referencing the workspace's `registry/` directory
  directly
- **THEN** no such import exists — installed component source under
  `src/components/ui/` and `src/adico_lib/` is the only source of UI code

### Requirement: apps/web has its own per-project Tailwind pipeline
`apps/web` SHALL have its own root `tailwind.css` compiled to its own
`apps/web/assets/tailwind.css`, linked via
`document::Stylesheet { href: asset!(...) }`, covering the landing, docs,
and playground route trees from one compiled stylesheet — since Tailwind
only emits the utility classes actually referenced in the project's own
`src/`, one project now needs exactly one such pipeline where three
previously needed three.

App-owned CSS — typeface declarations, font and type-scale tokens, and custom
variants — SHALL be written outside the `adico:theme:start`/`adico:theme:end`
marker region, because the `adico` CLI regenerates that region wholesale rather
than merging into it, and SHALL NOT be written inside it.

`apps/web` SHALL declare a `dark` custom variant bound to the `dark` class, so
that literal `dark:` utilities resolve against the theme class the app's own
theme control applies rather than against the operating system's colour-scheme
preference.

`apps/web` SHALL declare its own typeface: font families SHALL be served from
assets the project ships rather than fetched from a third-party origin at page
load, SHALL declare a fallback family so text remains legible before the
webfont loads, and SHALL be exposed as font tokens so pages reference them
through Tailwind utilities rather than per-element font declarations.

#### Scenario: apps/web is built
- **WHEN** `apps/web` is built or served
- **THEN** every Tailwind utility class referenced anywhere under its
  `src/` (landing, docs, or playground pages) is present in
  `apps/web/assets/tailwind.css`

#### Scenario: A registry item is installed after app-owned CSS exists
- **WHEN** `adico add <item>` runs against `apps/web` and regenerates the
  `adico:theme:start`/`adico:theme:end` region
- **THEN** the app's typeface declarations, font and type-scale tokens, and
  custom variants survive unchanged, and `adico css check` reports the
  compiled stylesheet up to date

#### Scenario: A visitor's OS theme disagrees with their chosen app theme
- **WHEN** a visitor whose operating system reports a light colour-scheme
  preference selects dark mode through the site's theme control (or the
  reverse)
- **THEN** elements styled with literal `dark:` utilities render according to
  the theme the visitor selected, not according to the operating system
  preference

#### Scenario: A page renders text
- **WHEN** any route under `apps/web` renders
- **THEN** its text is set in the project's declared typeface, served from the
  project's own assets, with no request to a third-party font origin

### Requirement: The site shell composes real registry components, not app-specific reimplementations
`SiteLayout` (the header/navigation and theme controls shared by every
route) SHALL be composed from installed registry components rather than
hand-rolled, app-specific reimplementations of the same behavior. Any
site-wide wiring needed to compose these components SHALL live under
`apps/web/src/components/`, SHALL NOT duplicate a registry component's own
logic, and SHALL NOT require any change to `registry/ui/*.rs` to exist.

#### Scenario: A new UI need arises in the site shell
- **WHEN** `SiteLayout` needs a button, card, navigation, or theme-toggle
  element
- **THEN** an existing installed registry component is composed to provide
  it, or a new generic registry component is proposed and added through the
  normal registry-item process — the site shell SHALL NOT gain a parallel,
  app-specific implementation of behavior a registry component already
  provides

### Requirement: Playground shell composes real registry components, not app-specific reimplementations
`PlaygroundLayout`'s navigation shell and theme controls SHALL be composed
from installed registry components (`sidebar`, `sheet`, `mode-toggle`,
`theme-switcher`, `theme-builder`, `resizable`) rather than hand-rolled,
app-specific reimplementations of the same behavior. Any playground-specific
wiring needed to compose these components (e.g. a launcher that opens a
dialog containing an installed component) SHALL live under
`apps/web/src/components/`, SHALL NOT duplicate a registry component's own
logic, and SHALL NOT require any change to `registry/ui/*.rs` to exist.

`PlaygroundLayout` SHALL NOT re-render chrome that `SiteLayout` already
provides for every route — specifically the adico logo and wordmark, the
`ThemeSwitcher`, and the `ModeToggle`. Controls with no equivalent in the site
header, such as the theme-builder launcher, SHALL remain. This applies to both
the `>= md` and `< md` presentations.

The navigation SHALL offer a filter that narrows the visible component
entries by typed text, composed from an installed registry component.

At viewports `>= md`, the navigation shell's nav column SHALL be composed
from `sidebar`'s structural sub-components (`SidebarHeader`,
`SidebarContent`, `SidebarGroup`, `SidebarGroupContent`, `SidebarMenu`,
`SidebarMenuItem`, `SidebarMenuButton`, `SidebarFooter`) rather than the
top-level `Sidebar`/`SidebarProvider` orchestration components, since the
latter's width is controlled by fixed CSS variables with no way to
cooperate with a drag-resize handle; the nav column's sizing is instead
composed from the installed `resizable` component (see "The
preview/controls split and the nav/content split are user-resizable",
below). This is not a departure from this requirement — the nav column
still renders no hand-rolled reimplementation of any of these
sub-components' own behavior.

At viewports `< md`, the same nav content (the same shared nav-list
rendering, not a reimplementation of it) SHALL instead be composed inside
the installed `sheet` component, opened by an ordinary click-driven
control — never by measuring or reacting to the viewport at runtime (see
the "Navigation is reachable and usable below the `md` breakpoint"
requirement, below, for the full behavior contract).

#### Scenario: A new theme or navigation need arises in the playground
- **WHEN** the playground needs new theme-editing or navigation behavior
- **THEN** an existing installed registry component is composed to provide
  it, or a new generic registry component is proposed and added through
  the normal registry-item process — the playground SHALL NOT gain a
  parallel, app-specific implementation of behavior a registry component
  already provides

#### Scenario: A registry component appears unused in playground source
- **WHEN** a registry component is installed in
  `apps/web/src/components/ui/` but referenced nowhere in playground source
- **THEN** this is a defect to fix (either wire it in or determine it's
  genuinely unneeded and stop installing it) — not a state to leave
  indefinitely, since it signals the playground has drifted from the
  components it's meant to demonstrate

#### Scenario: The nav column's sizing mechanism is located
- **WHEN** a maintainer looks for what controls the left nav column's
  width in `PlaygroundLayout` at a viewport `>= md`
- **THEN** it is the installed `resizable` component's `ResizablePanel`,
  not `Sidebar`'s own CSS-variable-driven width, and not a
  playground-specific width calculation

#### Scenario: The nav's presentation mechanism is located, at `< md`
- **WHEN** a maintainer looks for how navigation is presented at a
  viewport `< md`
- **THEN** it is the installed `sheet` component opened from a hamburger
  control, not a narrowed copy of the `>= md` `ResizablePanel`, and not a
  JS-driven viewport check

### Requirement: Navigation is reachable and usable below the `md` breakpoint
Below the `md` breakpoint, `PlaygroundLayout` SHALL present a mobile top bar
(at minimum: the playground logo/home link and a hamburger control) and
SHALL NOT render the `>= md` `ResizablePanelGroup` nav column, since that
column's minimum width truncates every nav label to an unreadable
fragment at phone widths. Activating the hamburger control SHALL reveal
the full nav list (every entry `nav_items()` produces, with the current
route's active state) in a dismissible overlay. The nav list content
itself SHALL be rendered by the same underlying nav-list rendering used
by the `>= md` tree — not a second, independently maintained copy — so a
route added to `nav_items()` appears in both presentations automatically.
Selecting a nav entry from the overlay SHALL both navigate to that route
and dismiss the overlay. The mechanism selecting between the `< md` and
`>= md` presentations SHALL be pure CSS (breakpoint-scoped utility
classes) — no JavaScript viewport or media-query detection SHALL be used,
consistent with the documented non-functional `document::eval`
viewport-detection pattern this project avoids elsewhere.

#### Scenario: A phone-width viewer opens the playground
- **WHEN** `/playground` is loaded at a viewport narrower than `md`
- **THEN** the `>= md` resizable nav column is not rendered, a top bar
  with a hamburger control is visible, and no element on the page causes
  horizontal document overflow

#### Scenario: A phone-width viewer opens navigation
- **WHEN** the viewer activates the hamburger control
- **THEN** every entry from `nav_items()` becomes visible, each fully
  legible (not truncated to a fragment of its label), with the current
  route visually indicated as active

#### Scenario: A phone-width viewer picks a page from the overlay
- **WHEN** the viewer selects a nav entry while the overlay is open
- **THEN** the router navigates to that entry's route and the overlay is
  no longer visible

#### Scenario: A route is added to nav_items()
- **WHEN** a maintainer adds a new entry to `nav_items()` for a newly
  routed page
- **THEN** the entry appears in both the `>= md` nav column and the
  `< md` overlay with no additional per-breakpoint wiring, because both
  render the same shared nav-list rendering

### Requirement: The `>= md` playground layout is unaffected by mobile support
Adding the `< md` mobile presentation SHALL NOT change the markup,
classes, sizing, or behavior of the existing `>= md`
`ResizablePanelGroup` layout (nav column via `resizable`, content panel,
resize handle) in `PlaygroundLayout`. The `>= md` tree remains exactly the
composition already required by "Playground shell composes real registry
components, not app-specific reimplementations" and "The preview/controls
split and the nav/content split are user-resizable".

#### Scenario: A desktop-width viewer opens the playground after this change
- **WHEN** `/playground` is loaded at a viewport `>= md`
- **THEN** the rendered nav column, content panel, and resize handle are
  unchanged from their pre-mobile-support geometry and behavior

### Requirement: Registry components are never modified solely for an app-section's convenience
A confirmed rendering or behavioral defect in a registry component,
discovered while composing it anywhere in `apps/web` (landing, docs, or
playground), SHALL be fixed in the registry source directly. A registry
component SHALL NOT be modified, extended, or given a section-specific
escape hatch (e.g. an `as_child` prop added only so playground can nest a
router `Link`) solely because a section's current composition approach
would otherwise be inconvenient.

#### Scenario: A registry component lacks a feature a section wants
- **WHEN** a registry component's existing API makes a composition awkward
  (e.g. `SidebarMenuButton` always rendering a native `<button>` with no
  way to substitute an `<a>`)
- **THEN** that section SHALL compose around the existing API using
  ordinary Dioxus patterns (e.g. `onclick` plus programmatic navigation, or
  an app-specific wiring component under `apps/web/src/components/`) rather
  than the registry component being changed to accommodate it

#### Scenario: A genuine rendering defect is found while using a real component
- **WHEN** composing an installed registry component anywhere in `apps/web`
  surfaces behavior that is wrong independent of that section (a real bug,
  not a section-specific inconvenience)
- **THEN** the fix lands in `registry/ui/*.rs` (or the relevant
  `adico-primitives` module) as its own cited change, verified against a
  live render, not worked around in that section's composition

### Requirement: Popup-family components are demoed via trigger and popup composition
A playground route for a component whose registry facade composes a trigger
and a floating or overlay content part (e.g. Popover, Dialog, Menu, Calendar's
popover-composing siblings) SHALL render that component activated through its
trigger, not pre-opened or rendered as a bare, always-visible surface, so the
demo exercises the same anchoring, focus, and dismissal behavior a consumer
gets. Where a trigger has a natural current-value display (a selected date, a
selected color, an edited theme), the trigger SHALL reflect that value rather
than a static label. A route MAY additionally render the component's
always-visible form alongside the trigger composition when doing so is needed
to demonstrate props that are only meaningful while the surface is visible
(e.g. a calendar's month/year navigation controls).

#### Scenario: A component with only a flat form is converted
- **WHEN** a playground route renders an overlay-family component's content
  unconditionally, with no trigger
- **THEN** this is a defect to fix — the route is updated to open that content
  from a trigger, consistent with every other overlay-family route

#### Scenario: A component needs both forms to demo its own props
- **WHEN** a component (e.g. Calendar) has props that only make sense while
  its surface is visible, and hiding it behind a trigger would prevent
  demoing them
- **THEN** the route MAY keep an always-visible instance for those props and
  additionally add a trigger-driven instance whose trigger shows the current
  value, rather than omitting the trigger composition entirely

#### Scenario: Every popup-family route can be forced open from its controls
- **WHEN** a playground route demos a popup-family component whose underlying
  registry item exposes controllable open state (an `open`/`default_open`/
  `on_open_change` prop, as Popover, Dialog, Sheet, Drawer, AlertDialog,
  DropdownMenu, ContextMenu, Select, Combobox, HoverCard, and Tooltip already
  do)
- **THEN** its "Component controls" panel includes a control to force the
  popup open, consistent with the other such routes

#### Scenario: A component has no controllable open state to expose
- **WHEN** a popup-family component's registry item manages its own open
  state entirely internally, with no `open`/`default_open`/`on_open_change`
  prop to control (for example, `menubar`, whose primitive tracks which of
  its menus is open as private context state shared across sibling menus)
- **THEN** the route is not required to add a non-functional control for it,
  and SHALL NOT gain a controlled-open prop added to the registry item solely
  to satisfy this requirement — that would be a playground-convenience-only
  registry change, which the existing "Registry components are never
  modified solely for an app-section's convenience" requirement already
  forbids

### Requirement: Playground demo defaults reflect the local timezone of the running device
Any playground demo default value derived from "now" (e.g. a calendar's
initially visible month, a time picker's initial value) SHALL be computed in
the timezone of the device running the UI, not UTC, using the crate's existing
local-time resolution helpers.

#### Scenario: A user views the Calendar demo from a non-UTC timezone
- **WHEN** the Calendar playground route loads with no date already selected
- **THEN** the calendar's initially visible month is the current month in the
  device's local timezone, not UTC

### Requirement: Navigation lists components in flat alphabetical order
The playground navigation list (`nav_items()`) SHALL present all component
entries as a single flat list ordered alphabetically ascending by displayed
label, with no thematic batches or restarted alphabetical runs. A newly
added component page SHALL be inserted at its alphabetical position.

When a filter is applied, the entries that remain SHALL keep that same flat
alphabetical order, and clearing the filter SHALL restore the complete list.
A filter SHALL never be the only route to a page.

#### Scenario: A user scans the navigation for a component
- **WHEN** a user opens `/playground` and scans the navigation sidebar
- **THEN** every component entry appears in one continuous A→Z sequence by
  its displayed label

#### Scenario: A user filters the navigation
- **WHEN** a user types into the navigation filter
- **THEN** only entries matching the typed text remain, still in one flat
  A→Z sequence, and clearing the filter restores every entry

#### Scenario: A new component page is registered
- **WHEN** a maintainer adds a navigation entry for a new component page
- **THEN** the entry is inserted at its alphabetical position in
  `nav_items()`, not appended at the end

### Requirement: The demo preview zone is pannable with a reset control
The shared demo preview zone SHALL render the demoed component centered by
default and SHALL let the user reposition it by dragging the preview
background. A drag that begins on the demoed component (or its immediate
wrapper) SHALL NOT start a pan, so the component's own pointer interactions
are never intercepted. The preview zone SHALL render a "Center" reset
control at the bottom of the preview area, composed from the installed
Button registry component, that restores the component to the centered
position.

The pannable area SHALL be visually distinguishable from inert page
background, so a user can tell the space is a workspace rather than empty
margin, even when the demoed component fills very little of it.

#### Scenario: Dragging the background moves the component
- **WHEN** a user presses on the preview zone's background and drags
- **THEN** the rendered component follows the drag offset and remains at the
  released position

#### Scenario: A preview zone holds a component much smaller than itself
- **WHEN** a demoed component occupies a small fraction of the preview zone
- **THEN** the pannable workspace is still identifiable as a workspace rather
  than reading as blank page background

#### Scenario: Dragging the component itself does not pan
- **WHEN** a user begins a drag on the demoed component (e.g. a slider thumb
  or a carousel track)
- **THEN** the preview does not pan and the component receives the pointer
  interaction unchanged

#### Scenario: Center resets the position
- **WHEN** a user activates the Center control after panning
- **THEN** the component returns to the centered default position

### Requirement: Playground demo tooling is composed from installed registry components
The playground's shared demo chrome and control widgets (boolean, text,
number, select, and optional-boolean controls) SHALL be composed from
installed registry components (e.g. Switch, Input, NativeSelect, Label,
Card, Button) rather than raw HTML form elements, to the extent an installed
component provides the needed behavior. The control widgets' public
signatures SHALL remain compatible with the generated control panels that
call them.

#### Scenario: A control widget renders
- **WHEN** a demo page renders a boolean, text, number, or select control
- **THEN** the widget's interactive element is an installed registry
  component, not a hand-styled raw HTML input

#### Scenario: Generated panels keep working
- **WHEN** the control widgets' internals are rebuilt on installed components
- **THEN** every generated `<Component>Controls` panel compiles and behaves
  unchanged, without regenerating or modifying the generator

### Requirement: Demo pages render only the demoed component's real composition
A playground demo page SHALL NOT render extra elements that exist only to
bind a generated control panel and are not part of a realistic composition
of the demoed component (e.g. a standalone pagination link rendered beside a
real pagination row solely to exercise its demo-state panel). When a
generated panel cannot be bound to the realistic composition, the page SHALL
omit that panel rather than invent a demo element for it. The Input page
SHALL render a single `Input` field rather than a second field added only to
demonstrate a hand-composed password-reveal pattern, once that pattern is
built into the `Input` registry component itself. The Drag And Drop List
page's use of `use_drag_and_drop_list_items()` SHALL be called from a scope
that is a descendant of `DragAndDropList`'s context provider, not from an
ancestor scope where the context does not resolve.

#### Scenario: A generated panel has no realistic binding target
- **WHEN** a generated control panel's props cannot be bound to any element
  of the page's realistic composition
- **THEN** the page omits that panel and renders only the realistic
  composition — no placeholder element is added to host the panel's bindings

#### Scenario: The Input page renders one field
- **WHEN** a user opens the `/playground/input` page
- **THEN** exactly one `Input` field is rendered, and its password-reveal
  behavior (when `r#type` is `"password"`) comes from the `Input` component
  itself, not from a second, separately composed field

#### Scenario: The Drag And Drop List page renders without hanging
- **WHEN** a user opens the `/playground/drag-and-drop-list` page
- **THEN** the page renders successfully and remains interactive, because
  `use_drag_and_drop_list_items()` is called from a scope that can resolve
  `DragAndDropList`'s context

### Requirement: A page's controls are grouped by the component that declares them
A playground page's "Component controls" panel SHALL present every control —
whether rendered by a generated `<Component>Controls` panel or hand-written
directly on the page — grouped under a label naming the exact component whose
prop that control edits, rendered as space-separated Title Case (e.g. a
hand-written group for `SelectList` SHALL be labeled "Select List", not
"SelectList"). A control SHALL NOT appear ungrouped, or grouped under a
component that does not actually declare the prop it edits, so that a viewer
can never infer that a subcomponent's prop belongs to its parent component or
to an unrelated sibling.

#### Scenario: A hand-written control for a subcomponent is grouped correctly
- **WHEN** a page hand-writes a control for a prop declared on a subcomponent
  (for example `SelectList`'s `align`)
- **THEN** that control is grouped under a label naming that subcomponent,
  rendered as "Select List" (not the unspaced `SelectList`), not under the
  root component's group and not left ungrouped

#### Scenario: A demo-scenario control that is not a real prop
- **WHEN** a page includes a control that toggles between two sibling root
  compositions rather than editing a real prop (for example Accordion's
  "Allow multiple open" switch between `Accordion` and `AccordionMulti`, or
  Select's "Multi-select" switch between `Select` and `SelectMulti`)
- **THEN** that control is grouped under the root component's own group,
  not given a separate unlabeled group and not merged into a subcomponent's
  group

#### Scenario: A multi-part page's controls read as distinct components
- **WHEN** a user views a converted multi-part component's page (for example
  `/playground/select`, `/playground/accordion`, `/playground/sidebar`,
  `/playground/dialog`)
- **THEN** the control panel's groups correspond one-to-one with the real
  components in that composition, and no group's label implies a component
  accepts a prop it does not declare

#### Scenario: A page is not required to display every component its registry file exposes
- **WHEN** a registry file exposes a component (locally or via a resolved
  re-export) that the page's own composition never renders (for example
  `select.rs`'s re-exported `SelectGroup`, which the `/playground/select`
  page never composes)
- **THEN** the page is not required to render a group for it — this
  requirement governs how a control that IS shown must be grouped, not which
  of a file's available components a page must display; the existing
  requirement that a page renders only its real composition and omits a
  generated panel with no realistic binding target still governs that
  question

### Requirement: The preview/controls split and the nav/content split are user-resizable
The shared demo preview/controls split (`Demo`) and the shared nav/content
split (`PlaygroundLayout`) SHALL each be composed from the installed
`resizable` registry component (`ResizablePanelGroup`/`ResizablePanel`/
`ResizableHandle`), letting a user drag to adjust the relative size of each
pair. Each split's two sides SHALL enforce a minimum and maximum size so
neither side can be dragged to zero or to fully consume the other; for the
preview/controls split these bounds are preview 60–80% and controls 20–40%
(default 70/30), and for the nav/content split they are nav 12–30% and
content 70–88% (default 18/82). Each panel's seeded size, minimum, and
maximum SHALL come from that panel's own configuration regardless of the
order in which sibling panels within the same group finish mounting — no
panel's initial state may be overwritten by another panel's configuration.
A resize handle's pointer hit target SHALL extend beyond its visible
divider line, so a user can grab it without pixel-precise pointer placement.
Resized sizes are NOT required to persist across a full page reload.
Because `PlaygroundLayout` is a persistent router layout (mounted once, not
re-mounted by in-app navigation within `/playground/*`) while `Demo` is
re-mounted fresh by every page navigation, the nav/content split's resized
size MAY persist across in-app navigation within the same session, while
the preview/controls split's resized size is reset by every page
navigation — neither behavior is a defect.

#### Scenario: A user resizes the preview/controls split
- **WHEN** a user drags the handle between the live component preview and
  the "Component controls" panel
- **THEN** the two areas' relative heights change accordingly, within their
  configured minimum/maximum bounds, on every playground page

#### Scenario: A user resizes the nav/content split
- **WHEN** a user drags the handle between the left navigation column and
  the main content area
- **THEN** the two areas' relative widths change accordingly, within their
  configured minimum/maximum bounds

#### Scenario: A resize handle is dragged to its bound
- **WHEN** a user drags a resize handle past its configured minimum or
  maximum size for either side
- **THEN** that side's size stops at the configured bound rather than
  shrinking to zero or growing to consume all available space

#### Scenario: A split renders at its intended default on every page, regardless of mount order
- **WHEN** a playground page mounts a `Demo` (or `PlaygroundLayout`'s)
  resizable split for the first time
- **THEN** each panel renders at its own coded default size — not a size
  copied from a sibling panel — independent of which panel's setup happens
  to complete first

#### Scenario: A resize handle is grabbable without precise pointer placement
- **WHEN** a user positions the pointer near, but not exactly on, a resize
  handle's visible divider line and presses down
- **THEN** the drag starts, because the handle's pointer hit target is wider
  than its rendered line

#### Scenario: A full page reload resets both splits
- **WHEN** a user resizes either split and then reloads the browser page
- **THEN** both splits render at their original default sizes

#### Scenario: In-app navigation may preserve the nav/content split but not the preview/controls split
- **WHEN** a user resizes the nav/content split, then clicks a different
  nav item without reloading the browser
- **THEN** the nav/content split MAY keep its resized size
  (`PlaygroundLayout` is not re-mounted by in-app navigation), while the new
  page's preview/controls split renders at its own default sizes (`Demo` is
  re-mounted fresh per page) — neither outcome is a defect

### Requirement: The docs route tree renders through Tailwind and installed registry components, not hardcoded CSS
`/docs` and `/docs/components/:name` SHALL be styled through `apps/web`'s
Tailwind pipeline and composed from installed registry components (e.g.
`Card`, `Table`, `Badge`) rather than a hardcoded, app-specific stylesheet.
Neither route SHALL inject CSS that overrides `body`-level or other
global styling outside its own rendered subtree, so light and dark mode
render correctly on every other route.

Registry-authored prose rendered by these routes SHALL present markdown inline
code spans as code rather than as literal backtick characters. The component
that performs this rendering SHALL live under `apps/web/src/components/` as
app-level wiring, and SHALL NOT require any change to `registry/ui/*.rs`.

Registry-authored text rendered by these routes SHALL wrap within the bounds of
the element containing it, for every item in the registry, with no horizontal
overflow past that element's visible edge at any supported viewport width.

`/docs/components/:name` SHALL render the component itself, live, in addition to
its prose and props table. Where a component has more than one variant, size, or
supported composition, the page SHALL show them rather than describing them.

For each such example the page SHALL display the example's source, and that
displayed source SHALL be the same source that produced the rendered example —
not a separately maintained copy of it. The mechanism SHALL make divergence
impossible rather than detectable after the fact.

`/docs/components/:name` SHALL show the command that installs the component, and
SHALL link to that component's `/playground/<name>` page.
`/docs` SHALL additionally provide conceptual guide routes covering, at minimum,
installation, theming, typography, spacing, light/dark mode, and how the
Tailwind pipeline is wired into a Dioxus application. Each guide SHALL be its
own route and its own file, consistent with the one-page-per-file rule.

Where a guide documents the theme tokens a project has installed, it SHALL
derive them from that project's own stylesheet rather than restating them in
prose, so the documented set cannot diverge from the set `adico add` actually
installed.

The docs route tree SHALL have a navigation shell listing its guides and its
components, composed from installed registry components. Adding that shell SHALL
NOT change the markup, classes, sizing, or behavior of `SiteLayout`, of the
landing page, or of `PlaygroundLayout`, and SHALL NOT introduce document-level
scrolling — the application scrolls inside `main`, not the document.

Every guide route SHALL be reachable at every supported viewport width,
including widths at which the docs navigation shell is not displayed.

A component for which no examples are authored SHALL still render its prose,
props table, install command, and playground link, so that adding examples is
incremental and never regresses a page.

The components that present examples, source, and install commands SHALL live
under `apps/web/src/components/` as app-level wiring composed from installed
registry components, and SHALL NOT require any change to `registry/ui/*.rs`.

#### Scenario: A user switches to light mode on a docs page
- **WHEN** a user toggles light mode while viewing `/docs` or
  `/docs/components/:name`
- **THEN** the page renders with the site's light theme tokens, with no
  forced dark background left over from a global style override

#### Scenario: A user navigates from docs to another site section
- **WHEN** a user navigates from `/docs/components/:name` to `/` or
  `/playground`
- **THEN** the destination route's own theme (light or dark, per the
  user's current selection) renders correctly, unaffected by any styling
  docs previously applied

#### Scenario: A registry item's prose contains markdown inline code
- **WHEN** `/docs/components/:name` renders a registry item whose description,
  composition note, accessibility text, or keyboard text contains a
  backtick-delimited span
- **THEN** that span renders as styled inline code, and no literal backtick
  character appears in the rendered text

#### Scenario: A registry item has a long description
- **WHEN** `/docs` renders the registry item with the longest description
- **THEN** that description wraps inside its card and no text crosses the
  card's visible border

#### Scenario: A reader opens the docs page for a component with variants
- **WHEN** a reader opens `/docs/components/:name` for a component that has
  authored examples
- **THEN** each example renders as a live, interactive instance of the real
  installed component, with its title and its own source available

#### Scenario: An example's source is changed
- **WHEN** the code of an authored example is edited
- **THEN** the source displayed for that example on the docs page changes with
  it in the same edit, with no separate regeneration, sync command, or
  verification step required to keep the two in agreement

#### Scenario: A reader wants to install the component they are reading about
- **WHEN** a reader views `/docs/components/:name`
- **THEN** the page shows that component's install command in copyable form and
  offers a link to its `/playground/<name>` page

#### Scenario: A component has no authored examples yet
- **WHEN** a reader opens `/docs/components/:name` for a component with no
  example module
- **THEN** the page renders its prose, props table, install command, and
  playground link without error and without an empty examples section

#### Scenario: A reader needs to wire Tailwind into a fresh project
- **WHEN** a reader opens the Tailwind guide
- **THEN** it documents the root stylesheet input, the compiled output and how
  it is linked, and the region of that file which is regenerated by the CLI and
  must not be hand-edited

#### Scenario: A project installs a registry item that adds a theme token
- **WHEN** the set of theme tokens in the project's stylesheet changes
- **THEN** the theming guide reflects the new set without any edit to the guide

#### Scenario: A reader wants to try a theme change while reading about it
- **WHEN** a reader opens the theming guide
- **THEN** they can change theme values from that page and see the page respond

#### Scenario: A reader opens a guide at a narrow viewport
- **WHEN** a reader opens any guide route at a width where the docs navigation
  shell is not displayed
- **THEN** the guide renders in full and remains reachable from `/docs`

#### Scenario: The docs shell is added
- **WHEN** the docs navigation shell is present
- **THEN** the playground's `>= md` resizable panel geometry and the landing
  page's shell are unchanged, and no scrollbar appears on the document itself

### Requirement: The landing page shell composes real registry components, not app-specific reimplementations
The landing page (`/`) content SHALL be composed from installed registry
components (e.g. `Badge`, `Card`, `CopyButton`) rather than hand-rolled,
app-specific reimplementations of the same behavior. Any app-specific
wiring needed to compose these components (e.g. a link component usable
where neither `Button` nor `NavigationMenuLink` can serve) SHALL live under
`apps/web/src/components/`, SHALL NOT duplicate a registry component's own
logic, and SHALL NOT require any change to `registry/ui/*.rs` to exist.

#### Scenario: A new UI need arises on the landing page
- **WHEN** the landing page needs a button, card, or install-command display
  element
- **THEN** an existing installed registry component is composed to provide
  it, or a new generic registry component is proposed and added through the
  normal registry-item process — the landing page SHALL NOT gain a
  parallel, app-specific implementation of behavior a registry component
  already provides

The landing page SHALL show real, rendered registry components, not only prose
describing them.

#### Scenario: A visitor lands on the site
- **WHEN** a first-time visitor opens `/`
- **THEN** they can see actual registry components rendered on the page,
  without navigating to another route
