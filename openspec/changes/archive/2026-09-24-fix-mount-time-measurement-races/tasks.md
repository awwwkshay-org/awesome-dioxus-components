# Tasks

Design decisions referenced below are `D1`–`D4` in `design.md`.

## 1. Reproduce

- [x] 1.1 With `dx serve` fully rebuilt, confirm the three failures still
  reproduce: `playground-enriched-demos.spec.ts`'s two Carousel tests and
  `playground-time-picker.spec.ts`'s dial-tracking test

  — **reproduced: all 3 failed exactly as task 9.8 described.**

## 2. Fix the registry source (D1, D2, D3)

- [x] 2.1 Add an `onresize` handler to `registry/ui/carousel.rs`'s scroll
  container, mirroring `scroll_area_viewport_onresize`: set viewport size from
  the observed content box, re-read `get_scroll_size()` for content size, and
  ignore zero-size readings
  — **done: `onresize` reads the observed content box for viewport size and re-reads `get_scroll_size()` for content size, ignoring zero boxes (a hidden element reports zero, and adopting that would restore the stuck state).**
- [x] 2.2 Add an `onresize` handler to `registry/ui/time_picker.rs`'s dial
  face, updating `face_size` only — no position, preserving the file's existing
  documented rationale
  — **done: size only, no position — the file's existing comment explains why a position captured at mount is wrong, and that rationale is preserved.**
- [x] 2.3 Rebuild the registry (`cargo run -p adico-xtask -- registry build`)
  and verify `registry validate` and `registry build --check` pass

  — **done. `registry build` initially refused on a checksum mismatch, which is the gate working as designed; `registry checksums --write` updated the two changed files, then `registry build`, `registry validate`, and `registry build --check` all pass (71 items).**

## 3. Refresh the installed copies (D4)

- [x] 3.1 Refresh `apps/web`'s installed `carousel` and `time-picker` through
  the real CLI, and verify the installed files match the registry source
  — **done via `adico add carousel time-picker --replace`. Verified with `diff`: both installed copies are byte-identical to the registry source.**
- [x] 3.2 Inspect `apps/web/adico.lock` for the known `--replace` digest-collapse
  defect; if per-item digests collapse, restore the lock and record it rather
  than committing a wrong lockfile

  — **inspected, and the concern was wrong.** The lock rewrote `manifestDigest` for all 8 items in the plan to one shared value — which the previous change had reported as a defect. It is not: `manifestDigest` is `sha256` of the *whole registry manifest* (`adico-registry-core/src/lib.rs:700`, cloned per item at lib.rs:848/920). Confirmed the new shared value is byte-identical to `shasum -a 256 packages/adico-cli/embedded/registry.json`, and that the lock holds **13** distinct digests across its 71 items — one per historical manifest version. The lockfile change is correct and kept; the earlier report is retracted in `redesign-web-visual-foundation/FOLLOWUPS.md`.

## 4. Verify the fix

- [x] 4.1 The three previously-failing tests now pass
  — **all 3 now pass.**
- [x] 4.2 The rest of the playground suite is unaffected:
  `playground-enriched-demos.spec.ts` in full,
  `playground-resizable-split.spec.ts`, `playground-drag-and-drop-list.spec.ts`,
  `--project=mobile`, `--project=desktop-invariance`
  — **no regressions: `playground-enriched-demos` 7/7 (was 5/7), `playground-resizable-split` 3/3, `playground-drag-and-drop-list` 1/1, `--project=mobile` 21/21, `--project=desktop-invariance` 14/14.**
- [x] 4.3 Repo baseline: `cargo fmt --all --check`, `cargo check --locked
  --workspace`, clippy over the five core crates with `-D warnings`, core-crate
  tests, `cargo test --locked -p adico-web`, `openspec validate --all --strict`
  — **all pass:** fmt clean, workspace check, clippy `-D warnings` exit 0, core tests exit 0, `cargo test -p adico-web` 142/0, `openspec validate --all --strict` 19/0.
- [x] 4.4 Registry staleness gates, since registry source changed:
  `registry validate`, `registry build --check`, `provenance check`,
  `primitive-usage check`, `styling-usage check`, `component-props check`,
  `prop-parity check`, `primitive-compat check`, `component-compat check`
  — **8 of 9 pass:** `registry validate`, `registry build --check`, `provenance check`, `primitive-usage check`, `styling-usage check`, `prop-parity check`, `primitive-compat check`, `component-compat check`.
  **`component-props check` fails as stale — pre-existing, not this change.** Proven twice: it fails identically with this change's registry/app edits stashed, and a `sync` diff shows the affected items are `badge`, `button`, `color_picker`, `input_otp`, `tag_group` — while `carousel` and `time_picker` are unchanged. `statics/component_props.json` was restored rather than regenerated, so the pre-existing staleness is left exactly as found instead of being hidden inside this change's diff.
- [x] 4.5 `cargo check --locked -p adico-web --target wasm32-unknown-unknown`
  and `adico css check`
  — **both pass.**
- [x] 4.6 Report which checks ran and which did not, with reasons

  — see 4.3/4.4. **Not run:** `playground-controls check` (pre-existing failure on `apps/playground/src/components/ui`, deleted by `c55c500`) and bare `npm test` (shared `baseURL` across specs targeting different dev servers).

## 5. Archive the stack

- [ ] 5.1 Archive this change, verifying `adico-existing-components` in
  `openspec/specs/` gains both modified requirements
- [ ] 5.2 Complete and archive `2026-09-24-merge-apps-into-web` (its task 9.8
  is now resolvable), verifying `adico-web-structure` lands in `openspec/specs/`
- [ ] 5.3 Archive `redesign-web-visual-foundation`, then
  `docs-component-examples`, then `docs-guide-pages` — in that order, since
  each modifies the same requirement
- [ ] 5.4 Verify `openspec validate --all --strict` passes and no active change
  remains that should have been archived
