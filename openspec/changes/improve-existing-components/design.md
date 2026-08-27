## Context

The registry exposes 21 source-installed UI components and the playground now
has an interactive preview workspace. However, the inherited component APIs
and examples have not all been compared to current shadcn and Dioxus
Components behavior, and the current playground controls only expose a small
subset of properties.

This is the existing-component hardening gate in the broader ecosystem change.
It improves the installed set only; it does not migrate or add a new component.

## Goals / Non-Goals

**Goals:**

- Audit every component across parity dimensions before declaring it hardened.
- Provide an explicit, typed playground control definition for each component.
- Close defects in dependency-coherent batches while preserving source
  ownership, semantic themes, and platform boundaries.
- Use Button as the reference implementation for a full native-wrapper API and
  its live demonstration.

**Non-Goals:**

- Creating a generic reflection system that attempts to discover Rust
  component props at runtime.
- Adding a 22nd registry UI component or claiming unexecuted platform parity.
- Rewriting primitives solely for stylistic consistency.

## Decisions

### 1. Audit by component and dimension

The initial artifact is a 21-row matrix. Each row records source/API,
variants/states, theme/dark mode, interaction/focus, accessibility, responsive
behavior, examples/docs, CLI refresh, and web/SSR/desktop evidence as a pass,
remediation, intentional Dioxus difference, or named dependency block.

The matrix is a feature ledger, not a high-level checklist. For every one of
the 21 installed items it records: the corresponding upstream Dioxus
Components parts and public properties; controlled/uncontrolled/default state;
visual variants, sizes, and semantic tokens; every focus, keyboard, pointer,
and ARIA state; responsive and target behavior; source-install dependencies;
playground controls; and behavior/consumer/platform evidence. A row cannot
state only that a component is “primitive-backed”; it must identify the exact
behavior the primitive supplies and the façade work still needed.

The catalog is a feature source, not a mandate to copy React-shaped or
upstream-internal APIs. An equivalent typed Dioxus composition counts only when
the ledger documents the mapping. A feature that cannot apply on a supported
target records a concrete block, owner, and validation disposition.

### 2. Define playground controls at each route

Each page owns typed signals for the component props that are useful to explore
and uses shared Boolean, text, number, and closed-option controls where they
fit. Component pages decide their own examples, permitted values, labels, and
types: Dioxus does not have safe runtime prop reflection. The controls panel is
bounded to the bottom quarter of the workspace and scrolls independently.

Button is the pattern: one page-owned state per variant, size, disabled state,
native type, and composition example. Other routes follow the same approach
only for options meaningful to their component (for example, alignment for a
popover, side/collapsible state for a sidebar, or selection for a calendar).

The Dioxus Components catalog is the interaction reference: component APIs
remain compositional, previews make meaningful states executable, and keyboard
interactions are part of the component contract rather than decoration. adico
retains its shadcn-style semantic CSS variables and source-install model.

### 3. Improve dependency-coherent batches

1. The first wave establishes the reusable patterns across Button, Pagination,
   Select, Combobox, Calendar, Date Picker, and Sidebar. It improves the
   specific public props and live controls described below, then refreshes
   every copied source file through the CLI.
2. Badge, Card, Input, Textarea, Skeleton, and Item share simpler semantic
   styling and native behavior.
3. Dialog, Sheet, Tooltip, Popover, Hover Card, Dropdown Menu, Context Menu,
   and Menubar share layer, focus, dismissal, and ARIA behavior.

Each batch finishes only after its individual component ledgers are closed;
completing a neighboring component does not carry evidence to another item.

### 3a. First-wave public contracts

- **Button** is a styled native `<button>` wrapper with typed
  `ButtonVariant` (`Default`, `Destructive`, `Outline`, `Secondary`, `Ghost`,
  `Link`) and `ButtonSize` (`Default`, `Xs`, `Sm`, `Lg`, `Icon`, `IconXs`,
  `IconSm`, `IconLg`). It forwards Dioxus button and global attributes/events
  and receives all visible content through `children`; icons are caller
  composition, not an icon-name prop.
- **Pagination** remains a semantic navigation composition. Its link parts
  preserve anchor semantics and accept normal link/global attributes; the
  previous/next presets expose caller-selected label text and a compact
  icon-only presentation instead of requiring a React renderer prop.
- **Select** and **Combobox** retain the owned primitive APIs for controlled
  value/open/disabled state, typeahead, and ARIA. Both provide explicit
  single- and multi-select root APIs with controlled and uncontrolled values;
  registry source supplies styled exported parts and documents the primitive's
  composition as the Dioxus alternative to React's item-array/render APIs.
- **Calendar** and **Date Picker** retain their primitive selection and
  keyboard models. Their registry façades supply a semantic default visual
  composition and allow the relevant controlled value, disabled/read-only,
  first-day, range, and navigation inputs to pass through as native primitive
  props.
- **Sidebar** retains its controlled/uncontrolled provider state, side, and
  collapsible model. The documented responsive disposition remains desktop CSS
  collapse until a real Dioxus viewport-to-Sheet primitive is available; it is
  not represented by a misleading `mobile` prop.

Each playground route owns strongly typed signals for these props and lists
only controls which change a meaningful part of the rendered example. The
route states the Dioxus alternative when a current shadcn API is React-only.
The route imports the CLI-installed component copy from
`apps/playground/src/components/ui`; it MUST NOT import registry source
directly. Consequently, every source change first gains a lock-verified CLI
refresh path, then refreshes the playground copy before its route/control work
is considered complete.

### 4. Preserve source ownership

Registry source is authored in `registry/ui`; reusable behavior belongs in
`adico-primitives`. Registry metadata and consumer copies refresh through the
CLI. No installed file under a consumer's `components/ui` directory is edited
as a source of truth.

### 5. Compose installed registry components deliberately

Registry components MAY compose another installed registry component when that
creates one consistent user-facing control rather than duplicating its visual
or native semantics. The composed component MUST be declared in
`registryDependencies`, and the CLI MUST install or refresh that dependency
before the consumer copy is compiled.

For example, `DialogTrigger` and `SheetTrigger` compose the installed `Button`
and expose its applicable `variant` and `size` props. They do not render a
second ad-hoc native button. The primitive remains the owner of dialog/sheet
state, focus, dismissal, and ARIA behavior.

### 6. Component completion contract

Each registry component is complete only when all applicable items below are
present and evidenced:

| Component | Required feature focus |
| --- | --- |
| Button | variants, sizes, composed children, native attributes/events, disabled/focus/link states |
| Badge | upstream status variants, semantic tokens, composed content, responsive wrapping |
| Card | structural parts, semantic surfaces, composed actions, responsive layout |
| Input / Textarea | native input attributes, value/default/invalid/disabled/read-only/focus states |
| Skeleton | shape/animation/reduced-motion and decorative accessibility disposition |
| Item | all compositional slots, interaction/disabled affordances, semantic states |
| Pagination | landmark, current page, link semantics, labels, compact/pointer/keyboard behavior |
| Dialog / Sheet | trigger composition, modal/non-modal layers, focus, Escape, outside dismissal, side/size/responsive behavior |
| Select / Combobox | parts, single/multi controlled values, disabled/open/query/typeahead/filter states, option/group/empty/indicator ARIA |
| Tooltip / Popover / Hover Card | trigger/content parts, delays/open state, placement, layers, keyboard/focus/pointer dismissal |
| Dropdown Menu / Context Menu / Menubar | nested item/group/separator/check/radio/submenu composition, roving focus, keyboard and pointer interaction |
| Calendar / Date Picker | single/range/controlled selection, constraints, navigation, first-day/month-year controls, typed date inputs, keyboard/ARIA |
| Sidebar | provider/open state, side/collapse modes, all structural/menu parts, active/disabled state, documented viewport disposition |

The playground is the executable catalog entry for this contract. It gives a
user a meaningful control for every safe visible property and an explicit
unavailable reason for properties that cannot be meaningfully demonstrated.

## Risks / Trade-offs

- [An audit reveals broad primitive gaps] → Repair the shared primitive first
  and record affected components as blocked.
- [A route exposes too many confusing controls] → Limit it to documented,
  meaningful properties and group related options; retain a canonical default
  example.
- [shadcn React APIs do not map exactly to Dioxus] → Document the Dioxus-safe
  alternative in public source and playground notes.
- [Visual refinements hide accessibility regressions] → Require keyboard and
  ARIA evidence before completing a component batch.

## Migration Plan

1. Build/review the component audit matrix and the per-route control plan.
2. Implement Button and its full interactive playground model.
3. Harden the three remaining dependency-coherent batches, refreshing consumer
   fixtures and evidence after each.
4. Publish the final hardening report before resuming new component work.
