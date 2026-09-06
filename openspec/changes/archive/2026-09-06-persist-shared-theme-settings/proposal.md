## Why

`theme-switcher`'s coordinated palette preset (Slate/Blue/Violet/Emerald/
Rose/Amber) lives only in an in-memory `GlobalSignal`: every mounted
instance stays in sync with every other instance on the same page load, but
the choice is lost on every reload, unlike `theme_mode`'s light/dark/system
setting, which already persists via a hand-rolled, ThemeMode-specific
mechanism. The user asked for the general fix rather than a second one-off:
a reusable, hook-based persisted-store primitive ("like React's Context/
Provider... if it changes from anywhere it updates in the store... saved in
the localStorage... reflects everywhere"), explicitly not a Context/Provider
wrapper component (this ecosystem's registry items are drop-in, zero-setup
components; a required root wrapper would be a new install step for every
consumer app).

## What Changes

- Add `adico_primitives::persisted_state::use_persisted_global`, a generic
  hook generalizing `theme_mode`'s existing `GlobalSignal` +
  `localStorage`/native-preferences-file pattern so any future shared,
  persisted setting can reuse it instead of hand-rolling the same
  three-target-gated load/persist machinery again.
- Refactor `adico_primitives::theme_mode::use_persisted_theme_mode` to be a
  thin wrapper over the new primitive, with no behavioral change (same
  public signature, same accepted async-load limitation, same tests).
  **BREAKING (internal, non-durable)**: a pre-existing native
  `adico-theme-mode.json` preferences file written by a build from before
  this change (`{"mode":"..."}`) is no longer read; it is simply rewritten
  in the new generic `{"value":"..."}` shape on the next set. This is a
  one-time reset of a non-durable OS-temp-directory file, not a real data
  migration.
- `theme-switcher`'s palette preset now persists across reloads through the
  same primitive (`adico-theme-palette` key), replacing its previous
  DOM-read-back hydration mechanism (which only ever recovered a value
  already applied earlier in the same page session, never across a reload).
- **Accepted, named tradeoff**: the removed DOM-read-back was specifically
  gated so `theme-switcher` would *adopt* a still-mounted `theme-builder`'s
  live per-token edits rather than overwrite them. The new, simpler
  apply-on-mount effect always writes the persisted preset's colors
  immediately, so mounting a `theme-switcher` while `theme-builder` has live
  edits applied now overwrites the primary/secondary/accent subset of those
  edits. This is not a new regression -- it is what the removed mechanism
  already did whenever the live values didn't match one of the six presets
  -- and the direction that matters (`theme-builder` reading back whatever
  `theme-switcher` last applied) is unaffected.

## Capabilities

### New Capabilities

(none -- `persisted_state` is new primitive-crate machinery, not a new
user-facing capability in its own right; its behavior contract is folded
into the modified `adico-primitives` capability below.)

### Modified Capabilities

- `adico-primitives`: adds a requirement that a shared, reload-surviving
  setting is expressed through one reusable, `GlobalSignal`-backed persisted
  primitive rather than each consumer hand-rolling its own storage
  read/write.
- `adico-existing-components`: adds a requirement that `theme-switcher`'s
  palette preset selection persists across a reload, in addition to staying
  synced across every simultaneously-mounted instance.

## Impact

- `packages/adico-primitives/src/persisted_state.rs` (new), `theme_mode.rs`
  (refactored), `lib.rs` (module registration).
- `registry/ui/theme_switcher.rs`, propagated to its three installed copies
  (`apps/playground`, `examples/basic-spa`, `examples/basic-ssr`),
  `registry/registry.json` (checksum + composition note),
  `registry/generated/items/theme-switcher.json` (regenerated),
  `statics/primitive_usage/theme-switcher.json` (regenerated: now also
  depends on the `persisted_state` primitive module).
- No public API break for any consumer: `ThemeSwitcher {}`'s own props are
  unchanged, and `use_persisted_theme_mode`'s signature is unchanged.
- `theme-builder` is explicitly untouched by this change; its per-token
  edits stay transient and unpersisted, by design.
