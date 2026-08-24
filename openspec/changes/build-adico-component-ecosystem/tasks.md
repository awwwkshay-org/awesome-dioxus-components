## 1. M0 — Repository and architecture foundation

- [x] 1.1 Audit the current Cargo workspace, current application/template assets, and OpenSpec baseline; record the disposition of every pre-adico package and verify the audit is reviewed in the M0 decision record.
- [x] 1.2 Define the target Cargo workspace members and create the `apps`, `packages`, `examples`, `registry`, `tests`, and `scripts` boundaries without moving implementation prematurely; verify `cargo metadata --locked` reports the intended skeleton.
- [x] 1.3 Create package skeletons for `adico-cli`, `adico-primitives`, `adico-registry-core`, `adico-test-utils`, and `adico-xtask`; verify each has a deliberate public boundary and the workspace builds with `cargo check --workspace --locked`.
- [x] 1.4 Create docs and playground application skeletons plus the basic, web, desktop, fullstack, forms, dashboard, and kitchen-sink example manifests; verify workspace members do not import registry UI source as a crate.
- [x] 1.5 Establish the test directory layout, CI baseline, and command matrix for formatting, lint, unit, compile, installation, browser, SSR/hydration, desktop, and visual checks; verify CI/documentation distinguish required from optional runners.
- [x] 1.6 Investigate and pin the Dioxus 0.7-compatible Tailwind workflow, CSS entry conventions, Lucide icon crate, browser test runner, and visual test tool; verify the resulting compatibility decision names tested versions, licenses, and platform support.
- [x] 1.7 Add repository dual-license files, contribution/provenance policy, and `UPSTREAMS.md`/record schema; verify a sample provenance record passes schema validation.
- [x] 1.8 Define the parity manifest JSON schema, source-of-truth ownership, evidence vocabulary, and offline validation rules; verify a minimal fixture is accepted and an incomplete-complete fixture is rejected.
- [x] 1.9 Record M0 acceptance: workspace/package boundaries, toolchain decisions, licensing policy, test baseline, and parity schema are approved; verify the decision record links all artifacts and `cargo fmt --all --check` passes.

## 2. M1 — Upstream inventory and primitive ownership

- [x] 2.1 Fetch and pin the current `DioxusLabs/dioxus-components` revision; inventory every styled component, primitive, test, CSS/theme asset, dependency edge, and license source, and verify the inventory is reproducible from the pinned revision.
- [x] 2.2 Classify every upstream styled component as `EXISTING_SHADCN_EQUIVALENT`, `EXISTING_DIOXUS_EXTRA`, `NEEDS_PARITY_UPDATES`, `NEEDS_PRIMITIVE_FIX`, or `NOT_SUITABLE_FOR_REUSE`; verify all inventory entries have one classification and rationale.
- [x] 2.3 Map upstream styled components to the checked-in current shadcn catalog snapshot and record existing-shadcn matches, Dioxus-only extras, and tentative gap groups; verify the mapping has no unnamed upstream item.
- [x] 2.4 Record the primitive dependency graph for all reusable candidates and select the minimal Button/Dialog/third-component runtime slice; verify the chosen third component is Select, Combobox, or Calendar based on actual source suitability and interaction coverage.
- [x] 2.5 Import/fork the selected primitive modules into `adico-primitives` with immutable provenance, retained notices, and public-facade boundaries; verify `adico-xtask provenance check` succeeds for every imported file.
- [x] 2.6 Preserve required upstream internal module structure for the initial primitive slice and add independent unit/compile coverage before refactoring; verify `cargo test -p adico-primitives --locked` passes.
- [x] 2.7 Add target-gated runtime adapters for SSR-safe behavior and the selected web/desktop capabilities; verify the primitive crate compiles for native, server, and `wasm32-unknown-unknown` targets required by the selected slice.
- [x] 2.8 Port or adapt reusable upstream test-harness helpers into `adico-test-utils`; verify a primitive-focused keyboard/focus test runs through the common helper.
- [x] 2.9 Add `adico-xtask upstream dioxus-components` inventory refresh and diff reporting; verify it produces no live-network requirement for normal CI after its snapshot is checked in.
- [x] 2.10 Record M1 acceptance: pinned inventory, classifications, provenance, selected owned primitives, and independent target checks are complete; verify all M1 reports are linked from `UPSTREAMS.md`.

## 3. M2 — Registry and CLI vertical slice

- [x] 3.1 Define Rust types and JSON schema for source registry manifests, named embedded/local/static-HTTPS registry sources, namespaces, default-registry selection, item metadata, source checksums, file target roots, registry dependencies, Cargo dependencies, module exports, style requirements, compatibility, and provenance; verify valid and invalid official and Awwwkshay fixture manifests deserialize as expected.
- [x] 3.2 Choose and implement the canonical authored item layout plus generated normalized local index/payloads; verify `cargo xtask registry build` is deterministic across two clean runs.
- [x] 3.3 Implement registry source loading for the embedded official registry, a configured local path, and static HTTPS, then validate source paths, item names/types, compatibility ranges, duplicate target intent, dependency references, cycles, checksums, and HTTPS-only network endpoints; verify negative fixtures fail with actionable diagnostics before project mutation.
- [x] 3.4 Implement deterministic transitive registry dependency resolution and install-plan generation in `adico-registry-core`, preserving source namespace for every item and requiring explicit cross-registry dependencies; verify unit tests cover shared dependencies, missing items, cycles, stable ordering, default-registry lookup, and cross-registry resolution.
- [x] 3.5 Implement Cargo dependency requirement unification, including package aliases, features, default-feature policy, and target predicates; verify compatible requirements merge and incompatible requirements identify their origins.
- [x] 3.6 Define and implement versioned `components.json` parsing/validation and migration rules for named registries and `defaultRegistry`; verify invalid paths/URLs/namespaces, unsupported versions, and valid official and company defaults are covered by tests.
- [x] 3.7 Implement Dioxus project discovery using nearest Cargo.toml, Cargo metadata, Dioxus dependency inspection, entrypoint checks, and explicit ambiguity failures; verify single-package, workspace, non-Dioxus, and ambiguous fixtures.
- [x] 3.8 Implement `adico init` directory/config/module/CSS preparation and explicit default-registry selection with a reviewable plan; verify fresh official and Awwwkshay-registry fixtures obtain valid `components.json` and no unrelated file changes.
- [x] 3.9 Implement marker-region `mod.rs` creation and updater with deterministic declarations/re-exports, idempotency, and malformed/duplicate marker rejection; verify preservation fixtures retain all bytes outside the region.
- [x] 3.10 Implement structured Cargo.toml edits with `toml_edit` for ordinary package and unambiguous workspace dependencies; verify comments remain preserved where TOML permits and conflicts make no manifest edit.
- [x] 3.11 Implement idempotent CSS/theme token installation using the selected Tailwind/Dioxus workflow and explicit marker ownership; verify light/dark tokens and radius tokens are installed once without overwriting unrelated CSS.
- [x] 3.12 Implement `adico add <component...>` plan validation and transactional file apply with checksum-based unchanged/user-modified detection, source-namespace reporting, and `adico.lock` manifest/item pinning; verify planning failure makes no project changes and a modified file is reported without overwrite.
- [x] 3.13 Implement `adico add --all` using the same resolver/apply path and deterministic output; verify it installs every v1-supported local registry item once in a fixture.
- [x] 3.14 Add Button registry source/metadata, minimal source-installed class utility, theme requirements, and icon dependency declarations; verify `adico init && adico add button` yields a compiling consumer fixture.
- [ ] 3.15 Add Dialog registry source/metadata and its owned primitive/Cargo/theme dependencies; verify an installed Dialog compiles and has browser tests for open, Escape, focus, ARIA association, outside interaction, nesting, and scroll locking as applicable.
- [ ] 3.16 Add the M1-selected richer component registry source/metadata and dependencies; verify its installed fixture passes its critical keyboard/accessibility interaction suite.
- [ ] 3.17 Create the basic consumer-style fixture by invoking the locally built `adico` binary for init and vertical-slice add, without workspace-path source imports; verify Cargo build and Dioxus web build succeed.
- [ ] 3.18 Add CLI integration tests for multi-item add, repeated add, shared dependencies, Cargo conflicts, file conflicts, malformed modules, dry-plan output, incompatible registry sources, and source-lock refresh behavior; verify the installation test suite passes.
- [ ] 3.19 Create an Awwwkshay curated-registry fixture with a local and static-HTTPS manifest, select it as `defaultRegistry`, and install bare plus explicit `@adico` items; verify dependency namespaces, generated source provenance, and consumer build outcomes are correct.
- [ ] 3.20 Add vertical-slice SSR/hydration and desktop smoke validation with recorded target outcomes; verify `parity.json` contains evidence/skip reasons for each selected component.
- [ ] 3.21 Record M2 acceptance: registry source→metadata→resolution→CLI→Cargo/module/CSS→consumer build/runtime pipeline passes for Button, Dialog, and the selected richer component from the official registry and a configured organization registry; verify the end-to-end acceptance command is documented and reproducible.

## 4. M3 — Existing component migration

- [ ] 4.1 Turn the M1 inventory into a migration queue ordered by reusable primitive availability and classify each entry’s registry item type, target source files, theme assets, Cargo dependencies, and platform limitations; verify every suitable upstream item has a migration decision.
- [ ] 4.2 Extend `adico-primitives` only with the audited upstream dependencies necessary for the first migration batch, preserving provenance and public facades; verify focused primitive tests and required target checks pass.
- [ ] 4.3 Migrate the first independent batch of existing styled components into `registry/` with source metadata, docs metadata, and installation fixtures; verify each can be installed by `adico add` into a clean consumer fixture.
- [ ] 4.4 Migrate overlay/layer-dependent existing components as a batch after their shared primitive behavior is available; verify browser keyboard, focus, and accessibility coverage for every migrated interactive item.
- [ ] 4.5 Migrate collection/selection/navigation-dependent existing components as a batch after their shared primitive behavior is available; verify representative keyboard/typeahead/selection tests and consumer builds.
- [ ] 4.6 Migrate existing Dioxus-only extras that meet adico quality and provenance rules, marking them explicitly as extras rather than shadcn parity; verify registry/docs labels prevent false parity credit.
- [ ] 4.7 For every migrated item, add parity-manifest entries, source provenance, registry validation, installation test coverage, documentation metadata, and kitchen-sink registration; verify `cargo xtask parity` reports no unclassified migrated entry.
- [ ] 4.8 Refresh the basic/example installation fixtures through the actual CLI for the current migrated set; verify they have no direct registry workspace imports.
- [ ] 4.9 Record M3 acceptance: every upstream item classified suitable for current reuse is installable through adico or has a documented blocking primitive/parity exception; verify the migration report and `cargo xtask registry validate` pass.

## 5. M4 — Existing component parity hardening

- [ ] 5.1 Refresh the checked-in first-party shadcn snapshot and compare each inherited equivalent against current composition/API, visuals, variants, states, accessibility, keyboard, theme, RTL, responsive, docs, and platform expectations; verify the audit creates evidence-backed parity gaps rather than name-only matches.
- [ ] 5.2 Close inherited public composition/API deviations in small dependency-coherent batches and document intentional Dioxus idiom differences; verify affected installed consumer examples compile without exposing primitive internals.
- [ ] 5.3 Close inherited visual, variant, state, semantic-token, dark-mode, RTL, and responsive gaps in small component batches; verify visual and targeted layout fixtures update with approved baselines.
- [ ] 5.4 Close inherited keyboard, focus, ARIA, layering, and pointer interaction gaps by strengthening shared primitives before dependent UI source; verify browser/accessibility tests cover the declared behavior.
- [ ] 5.5 Complete per-component documentation, composition examples, accessibility/keyboard notes, and parity evidence for each hardened batch; verify no component is marked complete with an omitted required dimension.
- [ ] 5.6 Record M4 acceptance: all M3 shadcn-equivalent components have either complete required parity dimensions or visible, justified in-progress status with a dependency-group task; verify `cargo xtask parity` and all targeted checks pass.

## 6. M5 — Low-complexity current shadcn gaps

- [ ] 6.1 Refresh the upstream catalog snapshot and derive the actual low-complexity missing set after M4; verify the proposed batch excludes components blocked by new foundational primitive work.
- [ ] 6.2 Implement the first audited compositional batch (for example Alert, Breadcrumb, Button Group, Empty, Input Group, Kbd, Native Select, Spinner, Table, and Typography only if still missing) as source-owned registry items; verify each installs, compiles, is documented, and has parity entries.
- [ ] 6.3 Implement any shared low-complexity utility/theme requirements as source-installed libraries rather than hidden styled runtime APIs; verify installed source remains understandable and independently editable.
- [ ] 6.4 Add focused visual/accessibility tests and kitchen-sink/examples coverage for each low-complexity batch; verify each item’s required parity dimensions are recorded.
- [ ] 6.5 Record M5 acceptance: the actual low-complexity gap batch is either complete or reclassified with a documented primitive dependency; verify the refreshed parity report has no silent omissions.

## 7. M6 — Shared primitive expansion

- [ ] 7.1 Use the current gap dependency graph to prioritize direction/RTL context, field semantics, focus/layering, positioning, roving focus, typeahead, drag gestures, observers, and scroll utilities; verify every planned primitive names the components it unblocks.
- [ ] 7.2 Implement and test direction/RTL, controllable state, collection management, selection, roving focus, and typeahead primitives in isolated public modules; verify unit and browser keyboard fixtures cover LTR/RTL and controlled/uncontrolled behavior.
- [ ] 7.3 Implement and test focus scopes/guards, dismissable layers, presence, overlay stack, portal behavior, and scroll locking; verify nested overlay, focus restoration, Escape, and outside-interaction suites.
- [ ] 7.4 Implement and test positioning, DOM measurement, pointer tracking/capture, and observer bridges behind platform-gated Rust APIs; verify SSR safety and web behavior in target-specific checks.
- [ ] 7.5 Implement and test drag/snap and scroll anchoring/scroll utilities required by current missing components; verify deterministic interaction fixtures and desktop compatibility outcomes.
- [ ] 7.6 Record M6 acceptance: every new primitive has provenance (if ported), documented public APIs, consumer-independent tests, target applicability, and a mapped dependent component; verify primitive target checks and browser suites pass.

## 8. M7 — Complex current shadcn gaps

- [ ] 8.1 Refresh and group the actual complex missing catalog by M6 primitive dependencies; verify each planned component has an unblocked foundation and a parity checklist.
- [ ] 8.2 Implement the audited command/combobox/navigation family (including Command and Navigation Menu only when still missing) as composition-first registry source; verify keyboard navigation, typeahead, accessibility, installation, and consumer build coverage.
- [ ] 8.3 Implement the audited overlay/gesture family (including Drawer and Carousel only when still missing) on shared layers/drag primitives; verify mobile-responsive, focus, pointer, and platform behavior coverage.
- [ ] 8.4 Implement the audited input/resize family (including Input OTP and Resizable only when still missing) using reusable selection/drag primitives; verify keyboard, paste/selection, accessibility, and interaction coverage.
- [ ] 8.5 Add every completed complex component to appropriate focused examples and kitchen-sink through the CLI refresh path; verify visual and SSR/hydration validation is recorded for each applicable item.
- [ ] 8.6 Record M7 acceptance: all current complex gaps are complete or explicitly blocked by a named, scheduled primitive/compatibility issue; verify parity output and focused suites pass.

## 9. M8 — Data and application components

- [ ] 9.1 Audit current shadcn Data Table and Chart source/composition requirements against completed adico primitives and registry items; verify the design record identifies replaceable headless data/chart dependencies and avoids an opaque all-purpose widget.
- [ ] 9.2 Implement missing Table-related source composition and the Data Table registry pattern using source-owned Table, Checkbox, Dropdown Menu, Button, Input, and Pagination composition; verify a dashboard fixture supports sorting/selection/pagination behavior covered by tests.
- [ ] 9.3 Implement the Chart registry pattern with shadcn-style configurable source and replaceable chart integration; verify examples cover tokens, tooltip/legend states, dark mode, and accessible fallback/documentation.
- [ ] 9.4 Add installation, Cargo conflict, docs, dashboard, kitchen-sink, visual, and applicable SSR/platform validation for each application component; verify complete items satisfy all relevant parity dimensions.
- [ ] 9.5 Record M8 acceptance: Data Table and Chart (if present in the refreshed upstream catalog) are source-owned compositional registry offerings with documented dependencies and parity evidence; verify dashboard integration checks pass.

## 10. M9 — Newer current shadcn components

- [ ] 10.1 Refresh the upstream catalog and derive the actual newer chat/agent component family after M8, including Attachment, Bubble, Marker, Message, Message Scroller, and Questionnaire only when present/missing; verify each has a dependency and platform assessment.
- [ ] 10.2 Implement generic message/scroll anchoring primitives before dependent chat source items; verify browser tests cover append/prepend anchoring, user scroll intent, observers, SSR safety, and declared desktop behavior.
- [ ] 10.3 Implement the audited chat/agent component batch as source-owned registry components with semantic, accessible composition; verify installation, keyboard/accessibility, responsive, and theme tests for each item.
- [ ] 10.4 Add focused chat/agent examples, kitchen-sink coverage, documentation, and parity evidence; verify unimplemented upstream additions remain visibly missing rather than omitted.
- [ ] 10.5 Record M9 acceptance: every current newer component is complete, explicitly in progress, or scheduled with an identified primitive dependency; verify parity reporting and focused integration suites pass.

## 11. M10 — Full parity validation and ongoing synchronization

- [ ] 11.1 Perform an explicit shadcn catalog refresh and reconcile every snapshot item with `parity.json`; verify there are zero untracked entries and zero missing first-party components before declaring full parity.
- [ ] 11.2 Run and remediate the complete required parity checklist for every component: source, API, visual/variants/states, keyboard, accessibility, dark mode, applicable RTL/responsive behavior, examples, CLI, Cargo, docs, web, desktop, and SSR/hydration; verify evidence links are machine-validated.
- [ ] 11.3 Validate official and configured organization registry sources, provenance records, installation fixtures, basic/web/desktop/fullstack/forms/dashboard/kitchen-sink examples, browser tests, accessibility tests, visual suite, and platform builds; verify skipped checks are reported with reasons and none are counted as passes.
- [ ] 11.4 Add scheduled/documented maintainer workflows for `cargo xtask registry build`, `cargo xtask registry validate`, company-registry validation, `cargo xtask parity`, upstream Dioxus inventory refresh, shadcn snapshot refresh, and provenance checks; verify normal CI works offline from checked-in snapshots.
- [ ] 11.5 Publish docs/playground parity status and CLI/registry/provenance maintenance documentation, including an organization-registry authoring and default-switching guide; verify examples and docs only advertise completed component dimensions accurately.
- [ ] 11.6 Record M10 acceptance: the refreshed first-party shadcn catalog has zero missing components and every complete record passes its applicable parity dimensions; verify the full release validation matrix is attached to the milestone report.
