## Context

See [proposal.md](proposal.md) for motivation and the accompanying capability
specifications for behavioral contracts. The current repository is a Rust 2024
Dioxus 0.7 full-stack template with API/UI-specific packages that will be
superseded or repurposed as adico grows. The target is a shadcn-style
distribution system: installed components are Rust source in the consuming
application, while reusable headless behavior stays in a small runtime crate.

The current upstream `DioxusLabs/dioxus-components` repository already
describes itself as a shadcn-style Dioxus component library built over
`dioxus-primitives`; it has a `primitives` crate, styled `preview` application,
Playwright tests, and Apache-2.0/MIT licensing. That makes it a strong starting
point, but not a release dependency: its exact inventory, dependency graph,
source revision, and suitability must be audited at M1. The current shadcn
catalog is also intentionally a moving target, so the supplied name list is a
proposal baseline only.

## Goals / Non-Goals

**Goals:**

- Deliver a versioned, owned primitive runtime; a local registry; and a CLI
  that installs editable, production-quality source into Dioxus projects.
- Prove the complete flow using inherited components before developing any
  currently missing shadcn components.
- Keep consumer component APIs stable and idiomatic even while primitive
  internals evolve.
- Track and continuously close parity using evidence rather than names alone.

**Non-Goals:**

- Initial remote/authenticated/community registry fetching, MCP integrations,
  all future CLI commands, or every shadcn block.
- An opaque all-in-one styled component crate for consumers.
- Rewriting primitives merely to change ownership.
- Claiming universal mobile/desktop/browser parity before it is tested.

## Decisions

### 1. Workspace and dependency direction

Adopt the following conceptual structure, reconciling it with Cargo workspace
and Dioxus tooling during M0:

```text
apps/        docs and playground maintained by adico
packages/    adico-cli, adico-primitives, adico-registry-core,
             adico-test-utils, adico-xtask
registry/    authored distributable source and metadata
examples/    externally realistic Dioxus applications
tests/       playwright, visual, compile, installation fixtures
scripts/     non-Cargo helper entry points
statics/     generated compat/catalog/classification snapshots (§9)
```

Runtime dependency direction is `Dioxus -> adico-primitives -> installed
registry source -> consumer application`; the CLI reads configured registry
sources but is not a runtime dependency. The registry components are
intentionally not a public workspace crate. `adico-registry-core` is pure
installation/domain logic and may be reused by CLI, docs, tests, and xtask.

Alternative considered: retain the current API/UI template as the dominant
architecture. Rejected because adico is a distribution platform, not a
database-backed product. Existing apps may be repurposed only after M0 records
their disposition; no cross-boundary API dependency is introduced.

### 2. Primitive fork and provenance strategy

M1 begins by cloning/inspecting a pinned commit of
`DioxusLabs/dioxus-components`, producing a checked-in inventory with: commit
SHA, component/primitive paths, internal dependencies, tests, classifying
status, licenses, and source suitability. `adico-primitives` initially mirrors
the minimum upstream internal module arrangement needed to preserve behavior;
it exposes a deliberately smaller, documented public facade. Refactoring
internals waits until behavior is covered, avoiding a risky rewrite concurrent
with the fork.

Maintain `UPSTREAMS.md` and machine-readable provenance records (one record per
import/port) containing upstream URL, immutable revision, license, original
path, import date, local path, changed files, and port/cherry-pick notes.
Retain both upstream license texts/notices and add source headers where required
by the licenses. `adico-xtask provenance check` validates that imported paths
have records. Upstream contributions are represented as links or commit IDs,
not as a release dependency.

Alternative considered: depend on `dioxus-primitives` directly. Rejected
because it permits upstream changes and release cadence to block adico. A
wholesale first-pass redesign is also rejected because it increases behavioral
and licensing review risk.

### 3. Registry schema and build model

`registry/registry.json` is the official source manifest with `formatVersion`,
registry identity, supported adico CLI/runtime ranges, and item metadata.
Organizations such as Awwwkshay author the same manifest format in a separate
repository or directory; they do not fork the CLI, the official registry, or
the primitive crate. Individual items either live in their own JSON metadata
file beside source or are declared by the root manifest; M2 selects one
canonical authored form and supports a generated normalized index. The
generated index is the CLI's local built-in official registry payload, not the
source of truth.

The normalized item model contains:

```text
name, type, description, documentation metadata, source files,
target roots/paths, registryDependencies, cargoDependencies,
runtimeRequirement, css/theme requirements, module exports,
compatibility, provenance, checksums
```

Item types start with `registry:ui`, `registry:component`, `registry:hook`, and
`registry:lib`; the enum reserves `registry:block`, `registry:page`,
`registry:theme`, `registry:file`, and `registry:template`. File target intent
uses logical roots (`ui`, `components`, `lib`, `hooks`, `css`) resolved from
`components.json`, rather than hard-coded consumer paths. The root index and
per-item payloads include source checksums to distinguish unchanged installed
files from user-modified files.

`adico-xtask registry build` deterministically validates source paths,
metadata, dependency edges, checksums, and produces generated index/payloads.
`adico-xtask registry validate` runs without network and accepts a supplied
company registry manifest as an input. `adico-registry-core` exposes a registry
source abstraction with three M2 implementations: the embedded official
registry (`@adico`), a local path (for an organization repository or fixture),
and a static HTTPS manifest (for a self-hosted organizational registry).
Authenticated transport, registry discovery, and marketplaces remain later
extensions of that abstraction.

`components.json` maps stable namespace names to sources and selects
`defaultRegistry`. Bare names resolve only through that configured default;
namespaced addresses, such as `@adico/button` or `@awwwkshay/button`, preserve
source identity through the entire dependency graph. A company registry item
therefore names company dependencies explicitly rather than relying on an
installer-relative path. The resolver records the manifest digest and item
checksums in a project `adico.lock`; a changed source requires an explicit plan
refresh before it can alter installed files. Static network sources require
HTTPS, while local sources are restricted to paths explicitly configured by the
consumer project.

Alternative considered: embed component strings in CLI code. Rejected because
it hides the distribution source, makes review difficult, and prevents docs and
tooling reuse. Directly copy shadcn's JSON schema is not used because Rust
module/Cargo requirements need first-class representation, though its
source-versus-built registry separation is adopted.

### 4. Resolution and plan-then-apply installation

`adico-registry-core` resolves requested names to a transitive DAG before any
write. It validates item/format compatibility, detects cycles/missing entries,
topologically orders dependencies deterministically, and deduplicates Cargo
and CSS requirements. It emits an immutable `InstallPlan` containing every
file write, module edit, Cargo edit, CSS/theme edit, and conflict precondition.

The CLI validates the complete plan against the target project, prints a
summary (and later supports a formal `--dry-run`), then performs writes through
staged temporary files and atomic renames where the platform permits. Any
precondition failure happens before writes; an unexpected apply failure reports
completed actions and preserves recovery material rather than silently
continuing. M2 defines the exact transaction/recovery behavior and tests it.

Registry dependencies are resolved by stable `(source namespace, item identity)`,
never relative paths. Bare dependencies inside a manifest resolve to that
manifest's namespace; cross-registry dependencies must be explicitly
namespaced. Cargo dependencies carry package name, semver requirement, feature
set, optional/default-feature policy, and target/feature predicates. Requirement
unification uses Cargo-compatible intersections; incompatible declarations are
reported with their registry and manifest origin.

### 4a. Registry discovery is configuration-aware and read-only

`adico list` loads the same configured registry catalog as `add`, listing the
default registry by default or one explicitly configured namespace when
selected. `adico view <item>` resolves bare and namespaced addresses by the
same default-registry rules and emits deterministic human-readable metadata:
the fully-qualified item address, description, type, files and logical targets,
registry and Cargo dependencies, style requirements, compatibility, and
provenance. These commands perform no installation planning or consumer-file
mutation, so they are safe for discovery in projects using an organizational
default such as `@awwwkshay`. Machine-readable output and remote search remain
future extensions; v1 keeps the contract clear, offline-friendly where the
configured registry is local or embedded, and aligned with `add` resolution.

### 5. `components.json` and Dioxus project detection

`components.json` is the consumer-owned, shadcn-compatible-in-spirit JSON
configuration. Its v1 shape includes:

```json
{
  "$schema": "https://adico.dev/schema/components.json/v1",
  "version": 1,
  "style": "default",
  "theme": { "tokens": "shadcn", "darkMode": "class" },
  "paths": {
    "components": "src/components",
    "ui": "src/components/ui",
    "lib": "src/adico_lib",
    "hooks": "src/hooks"
  },
  "css": { "entry": "assets/tailwind.css", "framework": "tailwind" },
  "registries": {
    "@adico": { "kind": "embedded" },
    "@awwwkshay": {
      "kind": "https",
      "url": "https://ui.awwwkshay.example/registry.json"
    }
  },
  "defaultRegistry": "@awwwkshay"
}
```

M2 validates final Dioxus/Tailwind integration against the pinned Dioxus
version. `init` finds the nearest Cargo.toml, uses Cargo metadata to identify a
package, then checks for an explicit Dioxus dependency and supported entrypoint
layout. Ambiguous workspaces require a package/path selection rather than a
guess. Existing configuration is read, migrated only through versioned logic,
and never silently reset.

Alternative considered: convention-only `src/components/ui` discovery.
Rejected because Dioxus apps vary in entry/CSS layouts and a predictable config
is required for safe updates. The name remains `components.json` to honor the
intended shadcn experience.

### 6. Module and Cargo mutation

Rust module registration uses marker-based managed regions in generated or
adico-managed `mod.rs` files:

```rust
// adico:start
pub mod button;
pub use button::*;
// adico:end
```

The updater parses only a single well-formed region, maintains deterministic
alphabetical declarations/re-exports, and otherwise leaves the file unchanged.
It creates the module path when configuration permits. It refuses duplicate or
malformed marker regions and does not implement remove/rename until a tracked
ownership manifest and explicit command exist. A Rust AST editor was considered
but rejected initially: module declarations/re-exports are a constrained,
marker-owned grammar and AST reformatting would make user-code preservation
less predictable.

Cargo.toml edits use `toml_edit`, targeting the selected consumer package
manifest; M2 documents how workspace-level dependencies are detected and when
the CLI offers/uses workspace edits. Existing compatible declarations remain;
incompatible versions, table types, aliases, and unclear workspace ownership
are planning conflicts, never silent rewrites. The plan includes each precise
manifest edit and tests comments/format preservation.

### 7. Style, class composition, and icons

V1 uses the current Dioxus-supported Tailwind workflow verified in M0/M2,
CSS variables for shadcn semantic tokens and radii, a class-driven dark mode,
and an idempotently installed/adopted CSS entry. The registry owns a small,
source-installed `cn` utility that accepts Dioxus/Rust-friendly class inputs,
eliminates empty values, and permits later Tailwind conflict resolution only if
the chosen ecosystem capability demonstrates a real need. It does not emulate
TypeScript `clsx` mechanically or hide component classes in runtime magic.

Registry items use the current Dioxus-compatible Lucide implementation through
a declared Cargo dependency and a small replaceable icon adapter/type boundary
when components need icons. They do not embed per-item SVG copies. M0 records
the selected crate's Dioxus/version/platform compatibility and licensing.

### 7a. Playground theme-combination validation

The playground is a consumer-realistic visual validation surface, so it owns a
small runtime-only customization launcher and modal rather than introducing a
new registry runtime or changing copied component source. The launcher remains
at the bottom of the navigation surface; the modal reuses the installed Dialog
component so focus, Escape, and outside dismissal retain the same behavior
that consumers receive. The modal provides independently
selectable primary, secondary, and tertiary palette presets as a quick starting
point, plus an advanced editor for the complete shadcn-style semantic contract:
page/card/popover surfaces and foregrounds; primary, secondary, muted, accent,
and destructive role pairs; border, input, ring, and radius; and every sidebar
role. Light and dark appearances retain separate values, so a user can edit a
combination without overwriting its counterpart.

The modal can export the active appearance as a paste-ready `:root` or `.dark`
CSS block containing the canonical shadcn variables. This export deliberately
omits derived Tailwind `--color-*` aliases: consumers retain their ordinary
`@theme` aliases, which resolve the copied canonical values in their own CSS
scope.

The selected mode is applied at the playground shell, together with the shared
custom properties, so all routed component pages respond without component
specific class or source changes. This keeps the theme contract identical to a
consumer's installed CSS-variable contract and makes the controls useful for
parity inspection. The initial configuration is deterministic and retained
only for the current page session; persistence, system-preference negotiation,
and consumer-facing theme generation remain outside this M3 playground task.

Tailwind v4's generated `--color-*` utility aliases must be overridden at that
same shell boundary as the lower-level shadcn variables. A `--color-primary`
alias inherited from `:root` has already resolved its `var(--primary)` value,
so changing only `--primary` on a descendant would not reliably recolor
`bg-primary` and related utility classes. The tray therefore supplies each
semantic source value and its Tailwind color alias together, keeping installed
component utilities live without altering their source.

The tray also provides a generate-theme action that selects a fresh palette
combination and applies the resulting role defaults to both appearances. It
uses an internal deterministic generator rather than browser randomness, so it
does not add a client-only dependency or make SSR/hydration behavior depend on
ambient platform state. Generated values are ordinary tray values and remain
individually editable afterward.

Alternative considered: create one stylesheet for every primary/secondary/
tertiary combination. Rejected because it grows combinatorially, obscures the
semantic-token contract, and makes adding a palette needlessly expensive.
Alternative considered: alter each installed component to accept palette
props. Rejected because themes must remain a shared consumer concern rather
than a divergent copied-component API.

### 7b. Cross-platform theme mode and consumer-facing theme switcher

Section 7a's playground launcher intentionally stayed playground-only, with
"persistence, system-preference negotiation, and consumer-facing theme
generation" explicitly out of that task's scope. This section supersedes that
exclusion for a new, real, consumer-installable feature: a `mode-toggle`
registry component (Light/Dark/System) and a `theme-switcher` registry
component (the palette presets already prototyped in the playground tray),
both installable through `adico add` like any other registry item.

**Theme-mode primitive.** `adico-primitives` gains a `theme_mode` module
exposing `ThemeMode::{Light, Dark, System}` and a `use_theme_mode()` hook that
resolves `System` to a concrete `Light`/`Dark` value. Unlike this repo's
existing web-only `document::eval` JS-bridge pattern (used for focus-trap,
portals, and pointer capture), OS/browser theme-preference detection is
delegated to the `dark-light` crate (MIT/Apache-2.0, `rust-version = "1.78"`,
well under this repo's pinned `1.96.1` toolchain), which supports macOS,
Windows, Linux/BSD (via the XDG Desktop Portal D-Bus API), and WebAssembly
from one API — so `web` and `native` features share the same detection call
rather than needing two divergent adapters. `dark-light::detect()` is a
one-shot synchronous read; its `subscribe()`/`stream()` APIs give live
OS-theme-change notification, and are used where the underlying target
supports a listener without adding an async runtime dependency the crate
doesn't already pull in. SSR/server builds (no `web` or `native`
feature) get a deterministic `Light` fallback rather than calling `detect()`,
matching the SSR-safety convention already established for every other
target-gated primitive in this crate (dialog's focus trap, portal, etc.) —
`System` is a real, selectable mode in that context, it simply resolves to
`Light` until a client feature is active.

**Registry components and classification.** `mode-toggle` composes an
existing primitive (dropdown-menu or select — chosen during implementation
based on the actual keyboard/ARIA fit, matching this repo's existing
composition-first rule for registry source) and is classified
`EXISTING_SHADCN_EQUIVALENT`-adjacent: shadcn's own theming documentation
ships an equivalent Light/Dark/System dropdown pattern, so `mode-toggle`
earns a real `parity.json` entry once implemented, following the same
honest `applicable: true, passed: false` + task-referencing-note convention
used for the 34 entries added in task 4.7. `theme-switcher` (the palette
picker) has no shadcn equivalent — ui.shadcn.com's own theme customizer is a
docs-site feature, not a shipped component — so it is classified
`EXISTING_DIOXUS_EXTRA`-style: no `parity.json` entry, and its registry
`description`/`compositionNote` avoid shadcn-equivalence framing, per the
labeling mechanism already established and validated in the Wave 5 extras
migration (task 4.6).

**Persistence.** The selected mode and palette persist across reloads: on
`web`, via `localStorage` through the same `document::eval` mechanism already
used elsewhere for class-driven dark-mode toggling; on `native`, via a small
JSON preferences file colocated with the consumer's Dioxus desktop/mobile data
directory. SSR/server builds have no persistence target and read the
deterministic default on every render, which is the same limitation every
other stateful client primitive in this crate already accepts.

Alternative considered: detect system preference via a pure
`@media (prefers-color-scheme: dark)` CSS rule with no Rust-side detection.
Rejected because it cannot drive a `native` (desktop/mobile window, no
browser CSS engine for OS-level media queries in all cases) or Rust-side
`ThemeMode` value that other primitives or consumer code might want to read,
and it cannot support live-updating `subscribe()`-style reactivity uniformly
across web and native.

### 7c. `adico css build`: a real, Node-free compile step for a shadcn-like DX

Task 4.8h identified that `adico init`/`adico add` never guarantee a consumer
project's Tailwind pipeline actually produces styled output — `examples/basic-ssr`
proved this by shipping fully unstyled while every existing check
(`registry validate`, `provenance check`, `parity`, text-content Playwright
specs) passed. Investigating that gap during M4 surfaced a second, related
problem: `assets/tailwind.css` (the *compiled* Tailwind output an installed
app's `document::Stylesheet` links) is a generated artifact currently
committed to source control with no tool that regenerates or checks its
staleness — unlike `registry/generated/*`, which has exactly that pair
(`cargo xtask registry build`/`registry validate`). This section specifies the
fix: a real `adico` subcommand that compiles a consumer's `tailwind.css` input
without requiring Node/npm anywhere in the chain, matching the "one CSS file
with your theme tokens, the tool handles compiling it" experience shadcn/JS
developers already expect.

**Why a Node dependency was the wrong assumption.** `docs/adico/m0-toolchain-decisions.md`
pins `@tailwindcss/cli` — the npm-distributed wrapper — but `dx serve`/`dx build`
were never actually invoking it: Dioxus's CLI downloads and caches Tailwind's
own official **standalone native CLI** (a self-contained executable per
platform, confirmed present at `~/.dx/tools/tailwindcss-v4.1.5/tailwindcss` on
a machine that has only ever run `dx serve`, never `npm install`). Tailwind
v4's engine (Oxide) is native; the standalone CLI is Tailwind's own supported,
Node-free distribution of it. `m0-toolchain-decisions.md`'s pin is corrected
by this section to the standalone binary — the npm package is not adopted
anywhere in the `adico` toolchain.

**The command.** `adico-cli` gains `adico css build [--check]` (name chosen
during implementation; folding into a broader `adico build` is acceptable if
it stays a distinct, individually invocable mode):

- Resolves a cached copy of Tailwind's standalone CLI for the host platform,
  downloading and verifying it (checksum/signature per Tailwind's published
  release artifacts) into a versioned local cache
  (`~/.adico/tools/tailwindcss-<version>/`, mirroring `dx`'s own
  `~/.dx/tools/` convention so the two tools' caches don't collide) on first
  use. The version is pinned in `adico-cli` the same way other toolchain
  versions are pinned in this repo, and is independently upgradable from the
  `dx`-managed copy — the two happen to agree today but are not assumed to
  stay in lockstep.
- Invokes it against the consumer's project-root `tailwind.css` (the same
  input `plan_theme_install` already manages) and writes `assets/tailwind.css`.
- `--check` (or an equivalent `adico css check`) compiles into a temporary
  location and diffs against the checked-in `assets/tailwind.css`, exiting
  non-zero on drift — the `registry validate`-equivalent staleness gate for
  this artifact, runnable in CI and by `cargo xtask` for this repo's own
  examples/playground.
- `adico init` and `adico add` are updated to close the actual 4.8h gap
  precisely because this command now exists to call: `init` scaffolds
  `Dioxus.toml` and the project-root `tailwind.css` if absent and invokes
  `adico css build` once so a fresh project already renders styled; `add`
  invokes it after every successful install so a consumer never has to
  remember a separate compile step, matching `npx shadcn add`'s expectation
  that the CSS is simply correct after the command returns.

**Consequence for the committed-artifact question.** With `adico css build`/
`--check` in place, `assets/tailwind.css` can be treated exactly like
`registry/generated/*`: committed, but with a real regeneration command and a
real staleness check, rather than a silently-trusted file nobody's tooling
touches. Whether this repository additionally gitignores its own examples'
`assets/tailwind.css` and relies on CI running `adico css build`/`--check` is
an implementation-time call for whoever picks up this task, not a design
constraint — either choice is now safe because a real command exists to keep
the file correct, which is the actual property that was missing.

**No consumer Node dependency.** Because the standalone binary is fetched and
run entirely by `adico-cli` (a single Rust executable), a downstream project
created via `adico init` never needs Node, npm, or any JS tooling installed
to get correct, current CSS — only `adico` and `cargo`. This preserves the
"pure Rust/Cargo Dioxus app" pitch that a per-consumer `build.rs` shelling out
to the npm-based CLI would have quietly broken.

Alternative considered: a per-consumer `build.rs` invoking the Tailwind CLI
directly at `cargo build` time, so plain `cargo check` is self-sufficient with
no separate `adico css build` step. Not rejected outright — it remains a
reasonable follow-on once `adico css build`'s binary-management logic exists,
since `build.rs` could shell out to the same cached binary — but sequenced
after the CLI command because it changes every consumer's build graph
(a network fetch or cache-miss inside `cargo build` itself) and deserves its
own scrutiny once the simpler explicit-command version is proven.

### 7d. `theme-builder`: productizing the playground's advanced theme editor

§7a explicitly kept `apps/playground/src/theme.rs`'s full 28-token editor,
independent light/dark values, deterministic "generate theme," and CSS
export as playground-only "parity inspection" tooling, reasoning that the
playground is "a consumer-realistic visual validation surface" that "owns a
small runtime-only customization launcher and modal rather than introducing
a new registry runtime." §7b then partially superseded that scope
boundary once — for mode switching and primary-color presets — by shipping
`mode-toggle`/`theme-switcher` as real registry components. This section
completes that supersession for the remaining piece §7a kept playground-only:
none of it (full token coverage, per-appearance independence, a generator,
CSS export) is actually playground-specific in nature. A real consumer
building a theme customization feature for their own app wants exactly this,
the same way they want `mode-toggle`/`theme-switcher`. Keeping it
playground-only was a scope decision made before `theme-mode`/`mode-toggle`/
`theme-switcher` existed to build it on top of, not a permanent architectural
one — and it left playground carrying two problems: a `ThemeMode` enum
duplicating and *weaker than* the registry's own (`Light`/`Dark` only, no
`System`, no persistence, no OS detection), and a primary-palette HSL table
byte-identical to `theme-switcher`'s.

**Component.** `registry/ui/theme_builder.rs` adds `ThemeBuilder`, a
self-contained component (no required props, matching `ModeToggle`/
`ThemeSwitcher`'s shape) that ports `theme.rs`'s `ThemeSelection`/
`ThemeVariables`/`ThemeToken`/`generate_theme`/`css_export` logic verbatim
where it still applies. It owns its own local editing state (all 28 tokens,
independently for light and dark, exactly as `theme.rs` does today) — this
state is intentionally *not* `theme_mode`'s persisted global signal, since
`ThemeBuilder` is an editing surface a consumer mounts occasionally (e.g.
behind a settings dialog), not an always-active mode switch.

**Fixing the mechanism conflict, not inheriting it.** `theme.rs` applies its
whole result via an inline `style`/class on a shell `<div>` wrapping the
entire app — a mechanism nothing else in the registry uses, and one that
actively conflicts with `theme-switcher`'s `:root`-level custom properties
(an inline style on a descendant always wins). `ThemeBuilder` instead applies
its edited tokens live through the *existing*
`adico_primitives::theme_mode::apply_root_properties` (already fully generic
— `&[(&str, String)]` pairs, no primitive-layer change required), the exact
mechanism `theme-switcher` already uses for its 4 properties, just extended
to the full 28-token set. This means `ThemeBuilder`, `ThemeSwitcher`, and
`ModeToggle` can all be mounted on the same page and compose correctly —
whichever was touched last wins for the specific properties it wrote, same
as any other CSS custom-property cascade a consumer would expect, with no
special-cased shell wrapper anywhere.

**Output for "designing a new theme."** Reuses `theme.rs`'s `css_export()`
logic verbatim to generate the same paste-ready `:root {}`/`.dark {}` CSS
block (this is what lets a consumer actually walk away with a new theme, not
just preview one live) — but *not* `theme.rs`'s `copy_theme_css`, which calls
`web_sys`/`wasm_bindgen_futures` clipboard APIs directly. That would leak a
browser-interop detail into registry UI source (prohibited by this repo's
architecture rules: "Browser-only interop belongs behind target-aware
primitive adapters") and require adding `web-sys`/`wasm-bindgen-futures` as
new `cargoDependencies` for this one item, a pattern no other registry item
uses. `ThemeBuilder` instead renders the generated CSS in a read-only,
selectable `<textarea>` — still fully "paste-ready," with zero new browser
API surface or cargo dependencies; the consumer selects and copies manually.
Additionally exposes an optional `on_theme_change: Callback<...>` prop so a
consuming app can persist or react to edits programmatically instead of only
copy-paste — the exact callback payload shape is an implementation detail
decided while porting (reusing the existing `ThemeVariables`-shaped token
table rather than inventing a new representation).

**Classification and provenance.** `EXISTING_DIOXUS_EXTRA`, same as
`theme-switcher` — no shadcn upstream has an equivalent (ui.shadcn.com's own
theme customizer is a docs-site feature, not a shipped component). Added to
`packages/adico-xtask/src/main.rs`'s `DIOXUS_ONLY_EXTRAS`; no `parity.json`
entry, per the existing extras-labeling convention (task 4.6).

**What happens to `theme.rs`.** A separate, already-scoped follow-up change
(`2026-08-30-playground-uses-registry-theme-and-sidebar`) deletes `theme.rs`
and its duplicated `ThemeMode`/`Palette` outright once `ThemeBuilder` exists,
and rewires `apps/playground` to compose `ModeToggle` + `ThemeSwitcher` +
`ThemeBuilder` + the real `Sidebar` family instead of any hand-rolled
equivalent. That rewrite is out of this section's/task 4.8k's scope — 4.8k
only adds the registry component.

Alternative considered: keep `theme.rs` playground-only permanently, as §7a
originally decided, and treat the duplication with `mode-toggle`/
`theme-switcher` as an acceptable, isolated exception. Rejected per this
session's explicit direction: registry components should be used to their
full extent inside the playground, and duplicated app-specific logic that
isn't actually app-specific in nature should become a real component instead
of a permanent carve-out.

### 8. Stable copied-component APIs and platform features

Installed components expose idiomatic Dioxus composition (for example,
`Dialog`, `DialogTrigger`, and `DialogContent`) and avoid exposing primitive
state internals. Each registry item documents intentional deviations from
current shadcn React composition. `adico-primitives` uses capability-oriented
features (base, DOM/web interop, desktop integration, SSR-safe stubs as
needed); copied components depend on stable public capabilities rather than
private modules. SSR paths render deterministic markup and defer DOM work until
client mount. Browser bridges (observers, measurement, scrolling, pointer
capture) are internal runtime adapters with no consumer JS surface.

### 8a. A named, public, Base-UI-shaped shared primitive layer

The 2026-08-30 shadcn props parity audit (`parity.json`, `api` dimension,
per-component evidence) found 29 of 38 tracked components with real API gaps,
and the gaps cluster around missing *shared behavior*, not styling: three
independent flat menu implementations with no submenu, checkbox-item, or
radio-item support (context-menu, dropdown-menu, menubar); accordion's
self-acknowledged missing controlled `value`/`on_value_change`; and a
`sideOffset`/open-close-delay gap repeated across popover, hover-card, and
tooltip because each reimplements positioning independently. Base UI
(`base-ui.com`) demonstrates the fix: every anchored-overlay component shares
one anatomy, and one `Menu` primitive backs every menu-shaped component.

Adopt that anatomy and inventory as the target shape for `adico-primitives`:

- A shared anchored-overlay anatomy of named, reusable parts: `Portal`,
  `Backdrop`, `Positioner`, `Popup`, `Viewport`, `Arrow`. `Positioner` owns
  collision-aware placement (side/align/offset/collision boundary/padding/
  sticky/anchor tracking); every popup-shaped component (popover, hover-card,
  tooltip, select, combobox, dropdown-menu, context-menu, menubar) composes it
  instead of reimplementing placement.
- A single `Menu` primitive with `SubmenuRoot`/`SubmenuTrigger` (arbitrarily
  nested), `CheckboxItem`, `RadioGroup`/`RadioItem`, `Group`/`GroupLabel`, and
  `Separator`, composed by the context-menu, dropdown-menu, and menubar
  registry items rather than each hand-rolling menu behavior.
- Existing crate-private shared behavior is promoted to a documented public
  surface rather than re-invented: `use_controlled` is already public and
  becomes the uniform controlled/uncontrolled pattern every primitive follows;
  `use_unique_id`/`use_id_or`, `use_animated_open` (presence), `use_focus_trap`,
  `use_outside_dismiss`/`use_global_escape_listener` (dismissable layer), and
  the `collection`, `selection`/`selectable`/`listbox`, `portal`, and
  `pointer`/`move_interaction` modules are all real, working primitives today,
  but are `fn`/`pub(crate)`/private-`mod` and therefore unusable by more than
  one component internally — which is precisely why the three menu components
  above were each written flat. Promotion is not sufficient by itself:
  several of these are also incomplete and need extending, not just exporting
  — the existing `portal` module is a logical VDOM relay that cannot escape
  `overflow`/`transform`/stacking contexts (a real DOM portal is needed for
  dialog/popover/tooltip/menus to layer correctly), `collection`'s roving focus
  is one-dimensional with no orientation or RTL flip (blocking calendar,
  menubar, and toolbar), and `use_focus_trap`'s underlying `js/focus-trap.js`
  only recognizes `A/INPUT/BUTTON/SELECT/TEXTAREA` as focusable, silently
  excluding the `[tabindex]` roving-focus items most adico components rely on.
- Net-new primitives genuinely absent today: the `Positioner`/`Arrow` engine
  itself, the unified `Menu`, a direction/RTL context (Base UI's Direction
  Provider equivalent), a real DOM portal, z-order overlay/layer stacking
  (today only Escape-key dismissal is stack-ordered, via `EscapeListenerStack`;
  there is no shared z-index registry), shared scroll locking (today only a
  private, file-local helper inside `context_menu.rs`; dialog and alert-dialog
  do not lock scroll at all), a unified pointer/gesture primitive (today split
  across three overlapping helpers, with `drag_and_drop_list` and
  `context_menu`'s long-press each reimplementing their own), a shared
  `use_typeahead` (today welded into `select`'s own context), and
  resize/intersection/mutation observer adapters.
- Base UI's inventory extends past current shadcn-mapped components; track
  Field/Fieldset/Form (field semantics), Number Field, OTP Field, Meter,
  Autocomplete, Navigation Menu, Preview Card, and Checkbox Group as primitive
  targets for shadcn components adico does not implement yet. Two Base UI
  concepts have no Dioxus analogue and are recorded here rather than silently
  dropped: `mergeProps`/`useRender` correspond to the `#[props(extends =
  GlobalAttributes)]` + `Element` composition adico already uses throughout,
  and `CSP Provider` is a React inline-style injection concern with no
  Dioxus/Tailwind equivalent.

This section does not introduce new milestone scope: task 5.2 (closing
inherited API deviations) and M6 (§7 of `tasks.md`) already own primitive
expansion — this decision replaces M6's prose enumeration with this named,
per-module inventory and the migration obligation for existing components.

**Standing rule once a primitive exists: registry components compose it,
they do not reimplement it.** As of task 7.6a, the net-new primitives listed
above are no longer all hypothetical — `direction` (7.3a), `typeahead`
(7.3c), `layer` (7.4a/7.4b, dismissable-layer stacking with `z_index()`),
`scroll_lock` (7.4c), `use_escape_key` (7.4d), `positioner` (7.5a/7.5b,
`Positioner`/`Arrow`), and `menu` (7.6a, the full `Menu` anatomy) are real,
public, independently-tested modules in `adico-primitives` today. This
obligation is not limited to task 7.8's migration of *existing* components:
it applies equally to every *new* registry component built for the rest of
this change, most immediately the task 7.9 Base-UI-parity tier —
`Autocomplete` and `Navigation Menu` are anchored-overlay components and
SHALL compose `Positioner`/`Arrow` rather than inventing their own placement
math, `Autocomplete` SHALL compose `typeahead` for type-to-select, and
`Preview Card` SHALL compose `Positioner` for its hover-anchored popup. A
component proposed after this point that reimplements behavior an existing
public primitive already provides is a design defect, not a style
preference, and should be caught at review the same way a duplicate
`Positioner` implementation would be today. `context_menu`/`dropdown_menu`/
`menubar` remain the one deliberate, temporary exception until 7.8 migrates
them onto `menu`, `positioner`, and `layer` — not a precedent for new work.

**Correction (2026-09-01):** 7.8e (closed) found this only held for
`dropdown_menu`, which is now a straight re-export of `menu`'s components.
`context_menu` and `menubar` turned out to be **permanent**, not temporary,
exceptions, for two distinct, verified reasons: `context_menu` anchors to an
arbitrary click point, which `positioner::Positioner` has no anchor-element
to key off; `menubar`'s current plain-CSS placement was live-verified to
stay glued to its trigger through a scroll of their shared containing block,
which `Positioner`'s one-shot, non-repositioning measurement (blocked on
7.5c's still-unimplemented observer bridges) would regress. Both still
compose `use_escape_key` for dismissal (added in the same 7.8e pass). See
`tasks.md`'s 7.8e note and this change's `specs/adico-primitives/spec.md`
delta for the full evidence. This is not a precedent against future work
composing `positioner`/`menu` — it is a recorded architectural boundary for
these two specific anchoring/placement models.

Target-gating for these primitives follows two real interop tiers, not four:
**web** (wasm/browser DOM) and **native** (desktop and mobile together) —
plus SSR/server as the no-interop fallback every gated hook already has.
Dioxus's desktop and mobile renderers both embed a platform WebView (`wry`/
`tao`: WebView2 on Windows, WebKitGTK on Linux, WKWebView on macOS and iOS,
Android's system WebView) showing the same HTML/CSS the web target renders —
Dioxus does not render native OS widgets on any platform — so a WebView
answers `document::eval`-style JS calls the same way a real browser does.
This is why every existing browser-interop hook (dismissable layer, focus
trap, presence, pointer tracking) is gated
`#[cfg(any(feature = "web", feature = "native"))]` with an SSR-safe no-op
fallback: a WebView answers those `document::eval`-style JS calls the same
way a real browser does, so one gate correctly covers desktop and mobile
together. `adico-primitives`' Cargo feature for this tier is named `native`
(renamed from its earlier `desktop` name) precisely so it isn't confused with
Dioxus's own `dioxus`-crate `desktop`/`web`/`mobile` platform features, which
select the *renderer* an application enables and are a separate concern from
this crate's own interop-capability flag. `docs/validation.md`'s platform
matrix row for this tier explicitly covers Android and iOS alongside desktop
for the same reason.

The rename was made before `adico-primitives` reaches its first published
release (`version = "0.1.0"`, not yet on crates.io — every registry item's
`cargoDependencies` entry pins it by path/version, not a released crate), and
no registry item or generated consumer `Cargo.toml` ever referenced the old
`desktop` feature name (every installed component is unconditionally pinned
to `features = ["web"]` today, a separate, already-tracked gap — see
`parity.json`'s `desktop` dimension notes on `dialog`/`accordion`/etc.), so
the rename has no compatibility cost to any real consumer.

Alternative considered: keep positioning and menu behavior implemented
per-component, as today. Rejected because it is the direct, evidenced cause of
the audit's largest gap (three divergent, incomplete menu implementations) and
guarantees the same class of gap recurs for every future anchored-overlay
component. Also considered: leave the existing shared hooks/modules
crate-private and only add the missing pieces. Rejected because it does not
fix the root cause named above — components would keep reimplementing behavior
that already exists one file away, and it contradicts this section's own goal
of one implementation per concern. Exposing raw primitive internals directly to
consumers (bypassing registry composition) was also considered and rejected,
consistent with §8's existing constraint that copied components depend on
stable public capabilities, not private modules.

### 9. Parity manifest, upstream snapshots, and synchronization

**Originally specified as follows; see "Removed 2026-08-31" below — none of
this describes a current command.** `parity.json` keys canonical shadcn component identifiers and contains catalog
snapshot reference, adico item mapping, status (`missing`, `in_progress`,
`complete`, `not_applicable`), intentional deviations, evidence links, and the
following dimensions: source, api, visual, variants, states, keyboard,
accessibility, darkMode, rtl, responsive, examples, cli, cargo, web, desktop,
ssrHydration, docs. Applicability is explicit; inapplicable does not silently
count as passing.

`adico-xtask upstream dioxus-components` refreshes the immutable upstream
inventory/fork-point records. `adico-xtask upstream shadcn-catalog` fetches the
official catalog only on explicit maintainer request, records source revision,
date and raw snapshot under `upstreams/`, then compares it with the prior
snapshot. `cargo xtask parity` operates offline on those checked-in inputs and
fails CI on malformed or incomplete records, reporting counts and gaps. It
does not hard-code catalog totals.

**Removed 2026-08-31:** `parity.json`, `parity.schema.json`,
`docs/adico/parity.md`, the `adico-component-parity` capability spec, and
`cargo xtask parity` (and its `DIOXUS_ONLY_EXTRAS` classification list) are all
deleted — per an explicit user instruction that this repo only needs to track
what is actually built, right now, not maintain a separate 17-dimension
completion-tracking manifest. `upstreams/dioxus-components/inventory.md` is
also removed for the same reason. Every remaining mention of `parity.json`/
`cargo xtask parity`/the inventory table elsewhere in this document (and in
`tasks.md`'s completed-task notes) describes a system that existed and was
used at the time, not a current command — left as an honest historical record
rather than rewritten. In its place, `packages/adico-xtask` gained
`baseui-compat` and `shadcn-compat` subcommands (Rust, using `syn` for source
introspection and `reqwest::blocking` for the Base UI live-drift check,
matching this crate's existing `reqwest::blocking` usage in `adico-cli`/
`adico-registry-core`) that regenerate `packages/adico-primitives/
baseui_compatibility.json` and `registry/shadcn_compatibility.json` — a
current-state snapshot of what's built vs. Base UI's/shadcn's own component
inventories, with prop/hook detail introspected from the live source, rather
than a hand-maintained, multi-dimension completion ledger.

**Restructured 2026-08-31:** `baseui-compat`/`shadcn-compat` are replaced by
`primitive-compat`/`component-compat` (same Rust/`syn`/`reqwest::blocking`
approach) because adico has two upstreams per layer, not one: the primitive
layer forked from `DioxusLabs/dioxus-components`' primitives as well as
following Base UI's architecture, and the registry models both shadcn and
that repo's styled components. `cargo xtask primitive-compat sync|check|diff`
now covers both the Base UI axis (unchanged: hand-maintained inventory + live
drift check) and a dioxus-primitives axis derived entirely from
`upstreams/dioxus-components/catalog.json`'s `primitiveSourcePaths` (no hand
table beyond a short exceptions list). `cargo xtask component-compat
sync|check` covers both a shadcn axis (derived from
`upstreams/shadcn/catalog.json`) and a dioxus-components axis (from the same
catalog's `styledComponents`) alongside the existing per-registry-item
props/hooks introspection. Both commands now write to a repo-root `statics/`
directory (`statics/primitive_compatibility.json`,
`statics/component_compatibility.json`) instead of living inside
`packages/adico-primitives/` or `registry/`, since a generated tracking
snapshot isn't shippable package or registry content. There is no live
refresh for `upstreams/shadcn/catalog.json` yet (only
`upstreams/dioxus-components/catalog.json` has one, via `cargo xtask upstream
dioxus-components`) — a known gap, not solved by this restructure.

**Replaced 2026-08-31 (catalog-fetch-tooling):** the `upstreams/` directory,
its hand-maintained/single-axis `cargo xtask upstream dioxus-components`
refresh command, and the Base UI live-drift GET embedded in
`primitive-compat sync`/`diff` are all replaced by a single `cargo xtask
catalog fetch <axis|all>` command (`packages/adico-xtask/src/catalog/`)
covering all four upstream axes (`shadcn`, `base-ui`, `dioxus-components`,
`dioxus-primitives`) under one shared schema, writing revision-pinned
snapshots to `statics/catalogs/<axis>.json`. This closes the "no live refresh
for `upstreams/shadcn/catalog.json`" gap noted just above, and adds
per-component prop/composition detail (with an explicit `props_source`
discriminator, since the four axes are not symmetric in what prop data is
even obtainable) that neither `upstreams/` catalog ever carried. `catalog
fetch` is now the only network-touching adico-xtask command; `primitive-
compat`/`component-compat` `sync`/`check`/`diff` read only the committed
`statics/catalogs/*.json` snapshots. See
`openspec/changes/catalog-fetch-tooling/` for the full proposal/design.

### 10. Examples, testing, and rollout

Examples are product fixtures, not workspace-source shortcuts. `examples/`
holds exactly two CLI-installed fixtures — `basic-spa` (web-only) and
`basic-ssr` (server + web, SSR/hydration) — per the `consolidate-examples`
change (2026-08-30) and its `adico-example-fixtures` spec; both, plus
`tests/installation/*`, are initialized/updated through the CLI against a
locally built/installed adico binary and compile with normal published-style
dependencies. New components migrated by later milestones get their
installation/parity evidence from `tests/installation/*` fixtures and
`examples/basic-spa`/`examples/basic-ssr` refreshes, not from progressively
growing the example set — a native-desktop fixture and an unwired
kitchen-sink/dashboard/forms/web gallery are explicitly out of scope unless a
future change reintroduces them.

Testing layers are: unit/property tests in primitives and registry core;
compile fixtures; CLI installation/conflict tests; Playwright interaction,
keyboard, and accessibility tests; SSR/hydration tests; desktop smoke/behavior
tests; and visual regression snapshots. Test helpers live in `adico-test-utils`
and browser suites in `tests/playwright`. A parity evidence record names the
executed checks. The existing upstream test harness is reused where compatible.

The rollout is M0–M10 as detailed in tasks: foundational workspace; audited
primitive ownership; a vertical slice that includes official and
organization-registry switching; existing migration; incremental parity; then
full validation. Docs/playground publish only components whose parity record
clearly exposes their maturity. Company-registry items have their own
provenance/quality metadata and are never counted toward official shadcn parity
unless explicitly mapped.

### 10a. Current-component hardening gate

Before another component migration wave begins, adico pauses on the currently
installed registry set and makes its consumer proof surface trustworthy. The
gate audits each installed component for public composition/API, all referenced
semantic tokens in light and dark themes, visual states and variants,
keyboard/pointer behavior, accessibility, responsive layout, and playground
coverage. Findings are closed through shared primitives or copied source as
appropriate, with focused tests and real consumer routes. This is a bounded
hardening pass for already migrated components, not a claim of final catalog
parity; the full M4 parity program remains the follow-on milestone.

## Risks / Trade-offs

- **Upstream source relies on unstable/private Dioxus APIs** → inventory and
  compile the selected primitive slice before porting styles; expose an adico
  facade and document replacements where a port is unsuitable.
- **Company registry content is untrusted or changes unexpectedly** → require
  HTTPS for network sources, schema/compatibility/checksum validation, explicit
  source namespaces, and a reviewed lock refresh before installation changes.
- **Cargo workspaces vary substantially** → support only unambiguous layouts
  initially, show precise conflicts, and add fixture coverage before widening
  project detection.
- **Marker management cannot safely merge arbitrary user module code** → scope
  it to an explicit generated region, preserve other content, and defer removal
  and rename rather than attempt heuristic edits.
- **Tailwind/Dioxus tooling changes** → pin and record tested versions, make
  CSS entry/configuration versioned, and validate through an installed fixture.
- **Parity scope grows with shadcn** → snapshot upstream explicitly, derive
  work from actual diffs, and let new items become visible missing work rather
  than silently changing the target.
- **Cross-platform UI behavior differs** → keep DOM code in primitives,
  declare per-dimension applicability, and never count skipped validations as
  passed.
- **Fork licensing records drift** → automate provenance validation and require
  records as part of registry/primitive review.

## Migration Plan

1. M0 introduces the workspace boundaries and no consumer installer behavior.
2. M1 pins/audits upstream and lands the independently compiling primitive
   foundation plus provenance records.
3. M2 adds the registry/CLI vertical slice, including default-registry swapping
   with a curated company fixture, and validates it in an external-style fixture
   before public documentation or broader migration.
4. M3 installs the suitable inherited catalog; M4 hardens its parity before
   M5+ add missing components by dependency group.
5. Package releases use semver; `components.json` and registry format versions
   are independently versioned. Breaking installed-source API changes require a
   migration note and explicit overwrite/diff flow, never an automatic rewrite.

Rollback is by ordinary package/version rollback and by retaining prior
registry build artifacts/snapshots. Consumer projects are never rolled back
automatically; failed CLI applies preserve files and report recovery actions.

This section predates the M4+ scope split recorded in `proposal.md`'s
2026-08-31 note and stops at M5; M6–M10 phase descriptions are not
duplicated here — see `tasks.md` sections 6–11 for the current milestone
breakdown of the remaining work.

## Open Questions

Both items below were open at proposal time and have since been resolved by
completed, archived milestones; kept for the historical record rather than
deleted.

- ~~The exact Dioxus-compatible Tailwind integration, Lucide crate, and
  browser test runner versions are intentionally deferred to M0 research~~ —
  resolved: Tailwind is compiled via its standalone native CLI, not an npm
  package (§7c); the icon crate and browser test runner were pinned during
  M0/M2 (`docs/adico/m0-toolchain-decisions.md`, `docs/adico/m2-vertical-slice.md`).
- ~~The upstream audit will select the third vertical-slice component~~ —
  resolved: Select was chosen (`docs/adico/m2-vertical-slice.md`: "Button,
  Dialog, and Select").

**Note (2026-09-01):** two changes proposed, implemented, and archived since
this design was last touched are relevant completed dependencies for the
still-open M6 primitive work below:
`reauthor-primitives-from-independent-spec` re-authored essentially every
file in `packages/adico-primitives/src/` against independent WAI-ARIA
APG/Base-UI specs rather than the original DioxusLabs port (several file
paths named in §8a and in `tasks.md`'s M6 tasks have since changed
substance, not just location); `enforce-registry-facade-standards` added
mechanically-verified behavior-ownership and styling classification rules
for `registry/ui/*.rs`, relevant to any future registry-facade migration
work under M6/M7.
