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
parity.json  checked-in first-party shadcn component parity state
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
from one API — so `web` and `desktop` features share the same detection call
rather than needing two divergent adapters. `dark-light::detect()` is a
one-shot synchronous read; its `subscribe()`/`stream()` APIs give live
OS-theme-change notification, and are used where the underlying target
supports a listener without adding an async runtime dependency the crate
doesn't already pull in. Native SSR/server builds (no `web` or `desktop`
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
used elsewhere for class-driven dark-mode toggling; on `desktop`, via a small
JSON preferences file colocated with the consumer's Dioxus desktop data
directory. Native SSR/server builds have no persistence target and read the
deterministic default on every render, which is the same limitation every
other stateful client primitive in this crate already accepts.

Alternative considered: detect system preference via a pure
`@media (prefers-color-scheme: dark)` CSS rule with no Rust-side detection.
Rejected because it cannot drive a `desktop` (native window, no browser CSS
engine for OS-level media queries in all cases) or Rust-side `ThemeMode`
value that other primitives or consumer code might want to read, and it
cannot support live-updating `subscribe()`-style reactivity uniformly across
web and desktop.

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

### 9. Parity manifest, upstream snapshots, and synchronization

`parity.json` keys canonical shadcn component identifiers and contains catalog
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

## Open Questions

- The exact Dioxus-compatible Tailwind integration, Lucide crate, and browser
  test runner versions are intentionally deferred to M0 research because they
  are version-sensitive, but that research is a gating task before M2 choices
  are implemented.
- The upstream audit will select the third vertical-slice component from the
  actually reusable rich components (prefer Select, then Combobox, then
  Calendar based on implementation and test coverage). This does not change
  the vertical-slice contract.
