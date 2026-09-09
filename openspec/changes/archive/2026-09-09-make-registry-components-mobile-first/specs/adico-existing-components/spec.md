## ADDED Requirements

### Requirement: Registry components lay out without horizontal overflow at a 375px viewport
Every `registry:ui` item SHALL render its content within a 375 CSS-pixel-wide
viewport without any element extending past the viewport's horizontal bounds.
This applies to the component's default state and to any interactive state a
consumer can reach without JavaScript beyond what the component itself already
runs (opened menus, popovers, dialogs, expanded rows).

#### Scenario: In-flow content wider than the viewport
- **WHEN** a registry component's content would naturally exceed a 375px
  viewport width
- **THEN** the component wraps, scrolls within a bounded container, or clamps
  to the available width — it never extends past the viewport's horizontal
  bounds

#### Scenario: A fixed-position overlay at 375px
- **WHEN** a Dialog, AlertDialog, or other fixed-position overlay opens at a
  375px viewport
- **THEN** the overlay retains at least a 1rem gutter between its edges and
  each side of the viewport

#### Scenario: A horizontal control strip exceeds the viewport width
- **WHEN** a horizontally-laid-out control strip (for example Tabs, Menubar,
  NavigationMenu, ButtonGroup, or a DataTable's toolbar or pagination footer)
  has more content than fits a 375px viewport
- **THEN** the strip wraps its items or exposes horizontal scrolling, and never
  silently clips or pushes a sibling element off-screen

### Requirement: Responsive adaptation is mobile-first and preserves desktop rendering
Every responsive Tailwind class a registry component uses SHALL be authored
mobile-first: the unprefixed form of a utility SHALL express the layout for
viewports narrower than Tailwind's `sm` breakpoint (640px), and any `sm:`/`md:`
prefixed form SHALL restore the exact value the component rendered at that
breakpoint before this requirement existed. A component SHALL NOT rely on a
prefixed utility (e.g. `sm:text-center`) to express its narrow-viewport layout.

#### Scenario: A component gains a responsive override
- **WHEN** a registry component is updated to lay out correctly at a 375px
  viewport
- **THEN** its computed geometry at viewports ≥640px is unchanged from before
  the update

#### Scenario: A narrow-viewport-only class is added
- **WHEN** a component needs different behavior below 640px than at 640px and
  above
- **THEN** the narrow-viewport behavior is expressed as the unprefixed utility,
  and the ≥640px behavior is expressed with an `sm:` (or `md:`, where a second
  step is genuinely needed) prefix — never the reverse

#### Scenario: A component's layout must respond to its own rendered width, not the viewport
- **WHEN** a component is reusable enough to be embedded at a width unrelated
  to the viewport (for example a `Card` placed in a narrow sidebar on a wide
  desktop screen), and Tailwind cannot compile a viewport-prefixed form of the
  needed selector at all (verified by its absence from the compiled
  stylesheet, not merely assumed)
- **THEN** the component MAY use a `@container` query and its `@sm:`-style
  container variant instead of a viewport breakpoint, provided the container
  query's unprefixed/base classes still express the narrow layout and the
  `@sm:`-and-up form still restores the exact value the component rendered at
  that container width before this requirement existed

### Requirement: Sidebar's mobile presentation remains deferred, with a defensive clamp
Sidebar's viewport-driven mobile Sheet mode SHALL remain deferred, since it
depends on a JavaScript viewport-detection primitive this project does not yet
have a working implementation of (`document::eval`-based viewport detection has
been found non-functional in this Dioxus runtime; see `registry/ui/sidebar.rs`'s
module documentation). Pending that primitive, an open Sidebar panel SHALL be
constrained so it can never exceed the viewport's width, even though it does
not yet collapse or overlay based on viewport size.

#### Scenario: Sidebar is open at a narrow viewport
- **WHEN** a Sidebar with `collapsible: Offcanvas` or `Icon` is rendered open at
  a viewport narrower than its configured `--sidebar-width`
- **THEN** the open panel's width is capped to at most 85% of the viewport
  width, so it never exceeds the viewport, but it does not automatically
  collapse or switch to an overlay presentation based on viewport width alone

#### Scenario: A consumer needs viewport-driven mobile collapse today
- **WHEN** a consumer needs Sidebar to automatically collapse or overlay below
  a given viewport width
- **THEN** they must still implement that themselves (for example, driving the
  existing `open`/`on_open_change` props from their own viewport signal), since
  Sidebar does not provide a built-in viewport-driven mobile mode
