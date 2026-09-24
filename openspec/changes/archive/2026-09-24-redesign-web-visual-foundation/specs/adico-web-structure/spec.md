## MODIFIED Requirements

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
