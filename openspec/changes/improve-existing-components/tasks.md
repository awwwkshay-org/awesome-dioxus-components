## 1. Audit and playground control foundation

- [x] 1.1 Create a checked-in audit matrix for all 21 registry UI components; verify each has source/API, variants/states, theme, interaction, accessibility, responsive, docs/examples, CLI, and platform dispositions.
- [x] 1.2 Compare every audit entry with current shadcn and Dioxus Components evidence; verify each gap is classified as a registry-source fix, primitive fix, intentional Dioxus difference, or named block.
- [x] 1.3 Define the typed per-route playground control contract and shared field controls; require each route to consume its CLI-installed source and verify controls support explicit labels, closed options, values, types, default examples, and an unavailable-reason without runtime prop reflection.
- [x] 1.4 Complete the shared playground workbench—centered logical-size preview, independently scrolling bottom controls, semantic theme Dialog, and CSS export—and verify responsive representative routes from every component batch.

## 2. First-wave foundation and Button reference slice

- [x] 2.1 Replace Button's minimal source API with typed current-shadcn variant and size options, caller-composed children, native Dioxus button/global attributes and events, semantic states, and semantic-link styling; verify focused API and compile evidence.
- [x] 2.2 Give Button a complete playground workbench with selectable variant, size, disabled state, native button type, and text/icon composition examples; verify each control immediately changes the installed Button preview.
- [x] 2.3 Refresh Button registry metadata, checksums, generated data, and the playground/consumer fixtures through the CLI path; verify no copied Button source is hand-edited.
- [x] 2.4 Close Button's feature ledger: add documentation and proportionate Rust, consumer compile, browser, keyboard, accessibility, web, and SSR evidence; record unavailable checks explicitly.

## 3. Pagination reference slice

- [x] 3.1 Close Pagination's full feature ledger: semantic navigation composition, current-page state, native link/global attributes, custom labels, compact presentation, semantic themes, responsive/accessibility behavior, playground controls, and focused tests.
- [x] 3.2 Refresh Pagination metadata and the playground/consumer fixtures through the CLI; verify copied source and checksums match registry source.

## 4. Selection, date, and layout first-wave slice

- [x] 4.1 Close Select's feature ledger as a styled registry façade: all parts, single/multi controlled values, open/disabled/name/typeahead state, option/group/indicator behavior, keyboard/focus/ARIA, semantic themes, and route controls.
- [x] 4.2 Close Combobox's feature ledger as a styled registry façade: all parts, single/multi controlled values, open/disabled/query/filter state, option/empty/indicator behavior, keyboard/focus/ARIA, semantic themes, and route controls.
- [x] 4.3 Close Calendar's feature ledger as a styled registry façade: controlled single/range selection, disabled/read-only/first-day/view/navigation constraints, parts, keyboard/focus/ARIA, semantic themes, and route controls.
- [x] 4.4 Close Date Picker's feature ledger as a styled registry façade: typed value/range, input/read-only/disabled/popover state, constraints, keyboard/focus/ARIA, semantic themes, and route controls.
- [x] 4.5 Close Sidebar's feature ledger: controlled open state, side, collapsible modes, all structural/menu parts, active/disabled state, semantic tokens, keyboard/pointer behavior, and documented viewport disposition.
- [x] 4.6 Refresh first-wave registry metadata and the playground/consumer fixtures through the CLI; verify installation plus affected web/SSR builds.

## 5. Remaining native/static component batch

- [x] 5.1 Close Badge's feature ledger: documented variants, semantic theme/dark-mode states, composed content, responsive behavior, playground controls, and tests.
- [x] 5.2 Close Card's feature ledger: all structural parts, semantic surfaces, composed actions, responsive behavior, playground controls, and tests.
- [x] 5.3 Close Input and Textarea feature ledgers: native Dioxus attributes, value/default/placeholder/invalid/disabled/read-only/focus states, semantic themes, playground controls, and tests.
- [x] 5.4 Close Skeleton and Item feature ledgers: variants/slots, decorative or interactive accessibility dispositions, semantic themes, responsive behavior, playground controls, and tests.
- [x] 5.5 Refresh native/static metadata and consumer fixtures through the CLI; verify copied source and checksums match registry source.

## 6. Overlay and menu component batch

- [x] 6.1 Close Dialog and Sheet feature ledgers through shared primitive fixes before source styling changes; compose installed dependencies only through declared registry dependencies; verify layers, focus, Escape, outside interaction, ARIA, keyboard, pointer, side, and responsive behavior.
- [x] 6.2 Close Tooltip, Popover, and Hover Card feature ledgers through shared primitive fixes before source styling changes; verify trigger/content composition, delay/open/placement state, layers, focus, Escape, outside interaction, ARIA, keyboard, and pointer behavior.
- [x] 6.3 Close Dropdown Menu, Context Menu, and Menubar feature ledgers through shared primitive fixes before source styling changes; verify item/group/check/radio/submenu composition, layers, roving focus, Escape, dismissal, ARIA, keyboard, and pointer behavior.
- [x] 6.4 Add meaningful typed playground controls and close audited visual, semantic-token, dark-mode, responsive, and public-composition gaps for every overlay/menu item; verify each responds to live themes.
- [x] 6.5 Refresh overlay/menu metadata and consumer fixtures through the CLI; verify installation plus affected web/SSR builds.

## 7. Completion evidence

- [ ] 7.1 Update feature-ledger documentation, playground usage, and parity/hardening records for every remediated item; verify every intentional difference and skipped target has a rationale.
- [ ] 7.2 Run formatting, targeted locked tests, registry/CLI validation, consumer web/SSR checks, applicable browser/axe suites, and `git diff --check`; record unavailable desktop, hydration, or visual checks as skipped.
- [ ] 7.3 Run `openspec validate improve-existing-components --strict` and publish the hardening report; verify all 21 component ledgers are complete or visibly blocked before new component work resumes.
