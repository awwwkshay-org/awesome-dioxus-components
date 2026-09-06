# Browser integration tests

Playwright suites for installed consumer applications live here. They cover
interaction, keyboard behavior, accessibility, and web-only runtime behavior.

`fullstack.spec.ts` exercises `examples/basic-ssr` (Button, Dialog, and Select
installed through the real `adico` CLI) for combined SSR and hydration: it
asserts the server-rendered HTML contains the installed markup, then confirms
the client wasm bundle hydrates and attaches interactivity with zero console
errors or warnings. Run `dx serve --platform web` from `examples/basic-ssr` in
one terminal, then in this directory:

```sh
ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:8080 npm run test:fullstack
```

`time-picker.spec.ts` exercises `tests/installation/time-picker-consumer`
(TimePicker and DateTimePicker installed through the real `adico` CLI). It
covers the segmented typed input, keyboard-only selection from the columns
view, the two pickers converging on one shared value, DateTimePicker combining
a date and a time, and an axe check scoped to the segmented input and columns.
Run `dx serve --web --port 5174` from `tests/installation/time-picker-consumer`
in one terminal, then in this directory:

```sh
ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:5174 npm run test:time-picker
```

`playground-time-picker.spec.ts` covers the analog clock dial — pointer drag
setting the hour, the hand tracking the pointer continuously rather than in
snapped steps, and the hand easing onto the snapped value on release. It runs
against the playground rather than the fixture above, because the dial relies
on `pointer-events-none` to stop its hour labels intercepting the pointer and
the fixture cannot compile that utility: `tests/installation/.gitignore`
ignores `*/src/components/`, and Tailwind v4 skips gitignored paths when
detecting sources, so none of a fixture's installed component classes reach its
stylesheet. Run `dx serve` from `apps/playground`, then:

```sh
ADICO_PLAYWRIGHT_BASE_URL=http://localhost:3000 npm run test:playground-time-picker
```
