## ADDED Requirements

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
  `/select`, `/accordion`, `/sidebar`, `/dialog`)
- **THEN** the control panel's groups correspond one-to-one with the real
  components in that composition, and no group's label implies a component
  accepts a prop it does not declare

#### Scenario: A component has both a generated panel and page-level hand-written controls
- **WHEN** a page hand-writes a control for a component (for example
  Accordion's "Allow multiple open" demo-scenario toggle, grouped under
  `Accordion`) and that same component also has its own generated panel
  (once `Accordion` is discovered as a re-exported component)
- **THEN** the page MAY render these as two separate groups both labeled with
  that component's name, rather than merging them into one — every control
  under either group still correctly names the component it belongs to, so
  this is not a violation of "grouped by the component that declares them"

#### Scenario: A page is not required to display every component its registry file exposes
- **WHEN** a registry file exposes a component (locally or via a resolved
  re-export) that the page's own composition never renders (for example
  `select.rs`'s re-exported `SelectGroup`, which the `/select` page never
  composes)
- **THEN** the page is not required to render a group for it — this
  requirement governs how a control that IS shown must be grouped, not which
  of a file's available components a page must display; the existing
  requirement that a page renders only its real composition and omits a
  generated panel with no realistic binding target still governs that
  question
