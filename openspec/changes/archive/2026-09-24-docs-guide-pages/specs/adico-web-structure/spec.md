## MODIFIED Requirements

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
