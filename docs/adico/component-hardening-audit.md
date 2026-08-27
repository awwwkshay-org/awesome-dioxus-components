# Existing component hardening audit

This record is the implementation ledger for `improve-existing-components`.
It covers the 21 `registry:ui` entries in `registry/registry.json`. Registry
source is the authority; every listed playground copy is refreshed with
`adico add --replace`, never edited as a second source of truth.

## Evidence and disposition keys

- **R** — registry-source remediation.
- **P** — `adico-primitives` remediation or retained primitive behavior.
- **D** — intentional Dioxus composition difference, documented rather than
  imitating a React-only renderer/slot API.
- **B** — named dependency block.

The comparison baseline is the current shadcn component documentation and the
owned Dioxus Components provenance recorded under `provenance/records/`.
Current primary references: [shadcn Button](https://ui.shadcn.com/docs/components/base/button),
[Pagination](https://ui.shadcn.com/docs/components/base/pagination),
[Select](https://ui.shadcn.com/docs/components/base/select),
[Combobox](https://ui.shadcn.com/docs/components/base/combobox),
[Calendar](https://ui.shadcn.com/docs/components/base/calendar),
[Date Picker](https://ui.shadcn.com/docs/components/date-picker), and
[Sidebar](https://ui.shadcn.com/docs/components/sidebar).

## Matrix

| Component | Source/API + variants/states | Theme / interaction / a11y | Responsive / docs / CLI / platforms | Disposition |
| --- | --- | --- | --- | --- |
| Button | Typed variants/sizes, composed children, native/global props | semantic hover/focus/disabled/link states | live variants/content controls; CLI/refreshed web+server copy | R |
| Badge | Default, secondary, destructive, outline, verified variants | semantic token/dark-mode-compatible status surfaces | variant/content controls; CLI/refreshed | R |
| Card | Attribute-forwarding structural header/content/footer parts | card semantic surfaces and composed Button actions | live title/description/action controls; CLI/refreshed | R |
| Input | typed value/type/placeholder/native invalid/disabled/read-only props | semantic invalid/focus/disabled/read-only states | live field controls; CLI/refreshed | R |
| Textarea | typed value/rows/placeholder/native invalid/disabled/read-only props | semantic invalid/focus/disabled/read-only states | live field controls; CLI/refreshed | R |
| Skeleton | default/circle shapes and reduced-motion treatment | decorative by default; opt out explicitly | shape/decorative controls; CLI/refreshed | R |
| Item | composed slots and default/muted/interactive/disabled states | semantic focus/disabled states | state controls; CLI/refreshed | R |
| Pagination | semantic landmark/current link, native link attributes, labels/compact mode | keyboard-native anchors and current-page ARIA | active/labels/compact controls; CLI/refreshed | R |
| Dialog | installed Button trigger, overlay/content/header parts | primitive focus, Escape, dismissal, ARIA; opaque layered overlay | controlled-open workbench route | P/R |
| Sheet | installed Button trigger and four-side content | primitive focus, Escape, dismissal, ARIA; opaque layered overlay | live side control | P/R |
| Select | styled parts; controlled single/multi values and open/invalid/disabled | primitive keyboard/typeahead/ARIA, opaque absolute popup | live single/multi/open controls; CLI/refreshed | P/R |
| Combobox | styled parts; controlled single/multi/query/open/disabled values | primitive filter/keyboard/ARIA, opaque absolute popup | live single/multi/open controls; CLI/refreshed | P/R |
| Tooltip | styled trigger/content placement surface | primitive hover/focus/Escape/ARIA | controlled-open/disabled route | P/R |
| Popover | styled trigger/content placement surface | primitive focus, outside dismissal, ARIA | open/alignment route | P/R |
| Hover Card | styled trigger/content placement surface | primitive pointer/focus/ARIA | controlled-open/disabled route | P/R |
| Dropdown Menu | styled root/trigger/content/item parts | primitive roving focus/Escape/dismissal/ARIA; opaque absolute popup | open/disabled route; CLI/refreshed | P/R |
| Context Menu | styled trigger/content/item parts | primitive pointer/keyboard/focus/dismissal/ARIA | open/disabled route | P/R |
| Menubar | styled bar/trigger/content/item parts | primitive roving-focus/keyboard/ARIA | canonical composition route | P/R |
| Calendar | styled single/range roots with all primitive parts retained | primitive keyboard/focus/ARIA; tokenised selected/unavailable states | selection/disabled/first-day controls; CLI/refreshed | R/P |
| Date Picker | styled typed single/range roots retaining primitive inputs/popovers | primitive constraints/focus/ARIA; tokenised selected, layered popup | disabled/read-only controls; CLI/refreshed | R/P |
| Sidebar | controlled provider/side/collapse, structural/menu parts, active/disabled menu item | semantic focus/pointer states | live state controls; CSS desktop disposition documented | R/D |

## First-wave control contract

Every route owns its state and declares the controls it deliberately exposes.
The shared controls render `ControlDefinition<T>` values, where `T` is the
route's concrete prop type. The renderer supports boolean, text, number, and
closed-option fields. A route may attach an unavailable reason for a supported
public prop that should not be demonstrated (for example, a callback whose
side effect would not be observable in the preview).

| Route | Live controls | Intentional Dioxus differences |
| --- | --- | --- |
| Button | variant, size, disabled, native type, text/icon composition | icons remain caller-composed children |
| Pagination | active page, previous/next labels, compact presentation | normal anchors/events replace a React renderer prop |
| Select | value, open, disabled, invalid presentation | primitive composition replaces item-array APIs |
| Combobox | value, open, disabled, query/default example | primitive composition replaces command/render APIs |
| Calendar | selection, disabled, first day of week, view month | typed `time::Date` state replaces JavaScript Date |
| Date Picker | selection, disabled, read-only, popover alignment | typed `time::Date` state replaces JavaScript Date |
| Sidebar | open, side, collapsible mode, active item | mobile Sheet behavior is unavailable pending viewport support |

Badge, Card, Input, Textarea, Skeleton, and Item have dedicated variation
controls. Dialog/Sheet and every popup/menu route exposes the safe live state
needed to inspect its primitive behavior. A menu’s check/radio/submenu API is
not fabricated in registry source where the owned primitive has no equivalent
part; that remains a documented primitive extension, rather than a misleading
visual-only prop.

## Completion evidence

Each first-wave row requires source API review, its declared live playground
controls, CLI-refreshed copied source and checksums, focused Rust/consumer
compile evidence, applicable keyboard/a11y checks, web and server feature
checks, plus a recorded reason for each unavailable browser, desktop, or
hydration check.
