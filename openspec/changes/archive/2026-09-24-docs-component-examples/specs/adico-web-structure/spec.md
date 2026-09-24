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
