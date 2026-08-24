# M0 UI and test toolchain decisions

Status: accepted for the M2 vertical slice

## Pinned compatibility baseline

| Concern | Selected version | License | Supported adico use | Decision |
| --- | --- | --- | --- | --- |
| Dioxus | `=0.7.9` | MIT OR Apache-2.0 | Web, desktop, SSR, fullstack | Existing repository pin; all adico application manifests use its documented platform feature model. |
| Dioxus CLI | `=0.7.9` | MIT OR Apache-2.0 | Web serving/building, Tailwind detection | Existing development baseline. |
| Tailwind CLI | `@tailwindcss/cli =4.3.3` | MIT | CSS generation for web and CSS-capable Dioxus renderers | Use Tailwind v4 input syntax and Dioxus asset output. |
| Dioxus Lucide icons | `dioxus-icons =0.1.0` | MIT AND ISC | Dioxus 0.7.x web, desktop, mobile, server/fullstack apps | Preferred registry icon dependency. It exposes one Dioxus component per Lucide icon, allowing the linker to retain only imported icons. |
| Browser and visual tests | `@playwright/test =1.62.1` | Apache-2.0 | Web/Chromium CI and local browser interaction tests | Use one runner for interaction, accessibility snapshots, and visual screenshots. |
| Automated accessibility | `@axe-core/playwright =4.13.0` | MIT | Web browser tests | Run alongside role/keyboard assertions; it supplements rather than replaces manual accessibility assessment. |

## Styling contract

The M2 `adico init` implementation will create/adopt a project-root
`tailwind.css` containing Tailwind v4's `@import "tailwindcss"` and an explicit
Rust source directive, then write its generated file to `assets/tailwind.css`.
Installed application roots include that asset with
`document::Stylesheet { href: asset!("/assets/tailwind.css") }`.

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
