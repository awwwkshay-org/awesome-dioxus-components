## Context

`packages/adico-primitives/src/theme_mode.rs`'s `use_persisted_theme_mode`
was, before this change, the only example in this codebase of a shared
app-wide setting that survives a reload: a module-level `static MODE:
GlobalSignal<ThemeMode>`, persisted to `localStorage` on `web` and a
hand-rolled preferences file on `native`, with a lowercase string-token
round-trip (`mode_token`/`mode_from_token`) since `adico-primitives`
deliberately carries zero serde dependency. `registry/ui/theme_switcher.rs`
separately held its own `static PALETTE: GlobalSignal<ThemePalette>`, added
earlier in the same work session, shared across mounted instances but never
persisted -- it re-derived its value each mount by reading the live
`--primary`/`--secondary`/`--accent` CSS custom properties back off the
document and reverse-matching them against its own preset tables, which only
ever recovered a value already applied earlier in the same page session.

## Goals / Non-Goals

**Goals:**
- One reusable, generic hook other persisted settings can compose, instead
  of a second hand-rolled copy of `theme_mode`'s storage machinery.
- Zero behavioral change to `ThemeMode`'s existing persistence.
- `theme-switcher`'s palette preset survives a reload and stays synced
  across every mounted instance, matching `ThemeMode`'s own guarantees.

**Non-Goals:**
- `theme-builder`'s full 28-token, per-role customization is explicitly not
  persisted by this change; it stays a transient live-preview/CSS-export
  surface, cleaned up on unmount as it already was.
- Cross-tab live sync (a `storage` event listener) is not provided. A
  `GlobalSignal` is per-document; a second tab picks up a persisted value on
  its own next load, not live.
- Real JSON serialization (`serde`) is not introduced; the native store
  keeps the existing hand-rolled single-field record shape.

## Decisions

**GlobalSignal, not Context/Provider.** Every registry item here is a
drop-in component a consumer mounts anywhere with zero setup (`ModeToggle
{}`, `ThemeSwitcher {}`, ...). A Provider component would force every
consumer app to wrap its root just to use one of these -- a new required
install step this shadcn-style, copy-paste-source ecosystem does not
otherwise ask of anyone. `GlobalSignal` already gives "read and write from
anywhere, observed everywhere" for free, matching the existing `MODE`/
`POINTERS` convention. This was the user's own explicit direction.

**A 4-argument function, not a trait.** `use_persisted_global<T: Copy +
PartialEq + 'static>(global: &'static GlobalSignal<T>, storage_key: &'static
str, to_token: fn(T) -> &'static str, from_token: fn(&str) -> Option<T>)`.
Only two call sites exist (`ThemeMode`, `ThemePalette`); a `StorageToken`
trait would add indirection this scale doesn't justify, and `ThemeMode`'s
existing `mode_token`/`mode_from_token` free functions already coerce
directly to the fn-pointer parameters with no restructuring.

**`T: Copy + PartialEq + 'static`, verified against the vendored
`dioxus-hooks` source, not assumed**: `use_memo` needs `PartialEq +
'static`; `Callback`/`use_callback` need `'static`; reading `*global.read()`
out of the `Deref` guard needs `Copy`. No `Send`/`Sync` bound is needed --
`Global<T, R>`'s `static` storage is `PhantomData`-backed and already `Sync`
regardless of `T`.

**Native store: one preferences file per key, hand-rolled `{"value":
"<token>"}`, no serde.** `adico-primitives` deliberately carries zero
serde/serde_json dependency (confirmed against its `Cargo.toml`). Generalizing
`theme_mode`'s existing single-file, single-field, string-split approach to
`{storage_key}.json` / `{"value":"<token>"}` needs no new dependency and
keeps every call site symmetric. The pure parsing/formatting helpers are
gated `#[cfg(any(feature = "native", test))]` rather than plain `#[cfg(feature
= "native")]`, since nothing in this workspace enables `native` and a plain
gate would mean they never compile or run in CI; `any(..., test)` exercises
them as real unit tests under the default profile while keeping them out of
the non-test lib build.

**No `loaded` third return value to suppress the first-paint flash.** The
persisted value still loads asynchronously after first mount on `web` (one
`dioxus_document::eval` round trip), so a render can briefly show the
default before the stored value lands -- the same accepted limitation
`ThemeMode` already had. A synchronous, hydration-matching read would need
an inline pre-hydration script; adding a third tuple element to the hook's
return type to let a caller suppress-render-until-loaded would grow the
primitive's API surface to work around a limitation already named and
accepted elsewhere. Not worth it for a strictly-better-than-before behavior
(a reload previously lost the palette selection entirely).

**Delete the DOM-read-back hydration outright, no migration path.** It only
ever recovered a value already applied in the same page session; a reload
wipes inline CSS custom properties, so on the very first load after this
ships there is nothing to read back anyway -- it would just "migrate" to
Slate, exactly what the plain default already does. Keeping it around as a
fallback would be dead weight, not a safety net.

## Risks / Trade-offs

**[Risk] A `theme-switcher` mount now overwrites live `theme-builder` edits
unconditionally, where the old code sometimes adopted them instead.** → The
old DOM-read-back was gated so `theme-switcher` would *adopt* a still-mounted
`theme-builder`'s live colors when they happened not to match one of the six
presets, rather than overwrite them; the new apply-on-mount effect always
writes the persisted preset immediately. **Mitigation**: this is an accepted
trade, not a new failure mode -- the old code already overwrote to Slate in
the (more common) case where `theme-builder`'s edits didn't match a preset
exactly, and the direction that matters -- `theme-builder` reading back
whatever `theme-switcher` last applied, via its own separate, unmodified DOM
read-back -- is unaffected. Verified live, not just asserted (see tasks.md).

**[Risk] A pre-existing native `adico-theme-mode.json` (old `{"mode":
"..."}` shape) is silently unreadable after this ships.** → **Mitigation**:
named explicitly, not silent. It's a one-time reset of a non-durable
OS-temp-directory file (already documented as non-durable before this
change), not a real data-loss concern; the file is simply rewritten in the
new shape on the next `set`.

**[Risk] No cross-tab sync.** → Named as a non-goal above; a second tab
picks up a persisted value on its own next load. Not attempting to solve
this now avoids a `storage`-event-listener feature that has no current
requester.

## Migration Plan

No database or deployed-service migration; this is a client-side registry
component and primitives-crate change, installed by copying source. Existing
consumers who re-run `adico add theme-switcher` (or otherwise refresh their
installed copy) pick up the new behavior automatically. No rollback beyond
reverting the source change is needed -- there is no persisted-data shape
that must be rolled back, since the "loss" on downgrade is limited to the
same one-time reset described above.
