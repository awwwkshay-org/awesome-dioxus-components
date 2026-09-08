## Why

Two splits in the playground's shared UI are fixed-size today and cramped on
common window sizes: the live-component preview vs. the "Component controls"
panel (a hardcoded `grid-template-rows: minmax(0, 3fr) minmax(0, 1fr)` in
`Demo`), and the left nav vs. the main content (`Sidebar`'s width, fixed via
two hardcoded CSS variables on `SidebarProvider` with no override prop). Both
splits live in shared components (`Demo`, `Layout`), used by every one of the
69 playground pages, so making them user-draggable there covers the whole app
at once. The already-built, unmodified `Resizable` registry component
(`ResizablePanelGroup`/`ResizablePanel`/`ResizableHandle`, already demoed at
`/resizable`) is the natural fit — no new component needs to be built.

## What Changes

- `Demo`'s preview/controls split becomes a vertical `Resizable` group: the
  preview canvas and the controls `Card` are unchanged internally, only the
  outer `grid-template-rows` wrapper is replaced with
  `ResizablePanelGroup`/`ResizablePanel`/`ResizableHandle`.
- `Layout`'s nav/content split becomes a horizontal `Resizable` group.
  **BREAKING (playground-internal only, no public API)**: `SidebarProvider`,
  `Sidebar`, `SidebarTrigger`, and `SidebarRail` are dropped from `Layout` —
  their built-in fixed-width CSS variables have no way to cooperate with a
  drag-resize handle, and per this ecosystem's own rule, `registry/ui/sidebar.rs`
  is not modified to add one solely for playground's convenience. The nav
  column keeps using `SidebarHeader`/`SidebarContent`/`SidebarGroup`/
  `SidebarGroupContent`/`SidebarMenu`/`SidebarMenuItem`/`SidebarMenuButton`/
  `SidebarFooter` (all plain, prop-driven wrappers with no context dependency)
  inside a plain resizable panel instead. This loses `Sidebar`'s built-in
  icon-collapse/offcanvas animation and its collapse-toggle button — an
  accepted tradeoff for drag-resize (confirmed with the user).
- No change to `nav_items()`, route navigation, active-state highlighting,
  footer contents, or the playground logo/home link — only the structural
  wrapper and sizing mechanism around them.
- No registry, primitive, or generated-file changes: both `registry/ui/resizable.rs`
  and `registry/ui/sidebar.rs` are used exactly as they already exist.

## Capabilities

### New Capabilities

(none — this change adds requirements to an existing capability)

### Modified Capabilities

- `adico-playground-structure`:
  - **ADDED** requirement: the preview/controls split and the nav/content
    split are user-resizable via the installed `Resizable` component.
  - **MODIFIED** "Playground shell composes real registry components, not
    app-specific reimplementations" — clarifies that the nav column
    composes `sidebar`'s structural sub-components (not the top-level
    `Sidebar`/`SidebarProvider` orchestration, which has no way to
    cooperate with a drag-resize handle) plus the installed `resizable`
    component for the split itself.

## Impact

- `apps/playground/src/components/demo.rs` (`Demo`'s preview/controls split)
- `apps/playground/src/routes.rs` (`Layout`'s nav/content split)
- No change to any of the 69 playground pages, `registry/ui/resizable.rs`,
  `registry/ui/sidebar.rs`, any other registry item, `adico-primitives`, the
  CLI, `adico-registry-core`, the xtask generator, or any generated control
  file.
- No database, WebAssembly-target, or CLI-installation validation surface
  applies — playground-app UI/layout work only.
