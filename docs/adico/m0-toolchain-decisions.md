# M0 UI and test toolchain decisions

Status: accepted for the M2 vertical slice

## Pinned compatibility baseline

| Concern | Selected version | License | Supported adico use | Decision |
| --- | --- | --- | --- | --- |
| Dioxus | `=0.7.9` | MIT OR Apache-2.0 | Web, desktop, SSR, fullstack | Existing repository pin; all adico application manifests use its documented platform feature model. |
| Dioxus CLI | `=0.7.9` | MIT OR Apache-2.0 | Web serving/building, Tailwind detection | Existing development baseline. |
| Tailwind CLI | Tailwind's standalone native CLI release `v4.1.5` (fetched from `tailwindlabs/tailwindcss` GitHub releases, not the npm-distributed `@tailwindcss/cli` package) | MIT | CSS generation for web and CSS-capable Dioxus renderers; `adico css build`/`adico css check` | Corrected in M4 task 4.8j: `dx serve`/`dx build` never actually used the npm package this row previously pinned -- they already fetch and cache this exact standalone binary into `~/.dx/tools/`. `adico-cli` does the same into a separate `~/.adico/tools/` cache (verified against Tailwind's published `sha256sums.txt`), so `adico init`/`adico add` can compile a consumer's CSS without requiring Node/npm anywhere in the chain. |
| Dioxus Lucide icons | `dioxus-icons =0.1.0` | MIT AND ISC | Dioxus 0.7.x web, desktop, mobile, server/fullstack apps | Preferred registry icon dependency. It exposes one Dioxus component per Lucide icon, allowing the linker to retain only imported icons. |
| Browser and visual tests | `@playwright/test =1.62.1` | Apache-2.0 | Web/Chromium CI and local browser interaction tests | Use one runner for interaction, accessibility snapshots, and visual screenshots. |
| Automated accessibility | `@axe-core/playwright =4.13.0` | MIT | Web browser tests | Run alongside role/keyboard assertions; it supplements rather than replaces manual accessibility assessment. |

## Styling contract

`adico init` creates a project-root `tailwind.css` (`components.json`'s
`css.entry`) containing Tailwind v4's `@import "tailwindcss"`, an explicit
Rust source directive, and the managed semantic-token/animation-utility
region, then compiles it to `assets/tailwind.css` via `adico css build` (task
4.8j) -- `adico init`/`adico add` call this automatically, and it is also
available standalone (`adico css build`/`adico css check`) for manual
recompiles or CI staleness gating. Installed application roots include that
asset with `document::Stylesheet { href: asset!("/assets/tailwind.css") }`
(added by hand today; `adico` warns if it is missing rather than editing an
arbitrary entrypoint automatically).

Dioxus 0.7 detects a root Tailwind input during `dx serve` and can configure
different input/output locations through `Dioxus.toml`; adico retains the
conventional root input plus `assets/` output so copied source remains easy to
understand. Semantic colors, radii, and dark-mode classes are CSS variables in
that consumer-owned entry point. Rust component code keeps Tailwind class names
literal or uses sibling `class` attributes so the Tailwind scanner can see
conditional classes.

## Test contract

M2 creates a Node package under `tests/playwright` pinned to the selected
Playwright and axe adapter versions. The runner starts Dioxus with `dx serve`,
uses Chromium in a fixed CI container for visual baselines, and stores approved
screenshots in version control. It uses `toHaveScreenshot` for visual
regression and role/keyboard assertions plus axe scans for accessibility.

Desktop checks remain Rust/Dioxus smoke checks on a supported platform runner;
Playwright does not claim desktop renderer coverage. SSR/hydration uses Dioxus
server and browser fixtures separately.

## Evidence

- [Dioxus 0.7 Tailwind guide](https://dioxuslabs.com/learn/0.7/guides/utilities/tailwind/)
- [Dioxus CSS and asset guidance](https://dioxuslabs.com/learn/0.7/essentials/ui/styling/)
- [Dioxus web testing guide](https://dioxuslabs.com/learn/0.7/guides/testing/web/)
- [dioxus-icons 0.1.0 documentation](https://docs.rs/dioxus-icons/0.1.0/dioxus_icons/)
- [Playwright visual comparisons](https://playwright.dev/docs/test-snapshots)
- [Playwright accessibility testing](https://playwright.dev/docs/accessibility-testing)

## M2 verification gate

The versions above are selected from their published compatibility information.
Before a component is marked installable, M2 must execute: a Tailwind-generated
consumer web build, a `dioxus-icons` component compile in installed source,
Playwright Chromium interaction/axe tests, and a deterministic screenshot
comparison in the configured CI environment.
