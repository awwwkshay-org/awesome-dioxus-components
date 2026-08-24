# Browser integration tests

Playwright suites for installed consumer applications live here. They cover
interaction, keyboard behavior, accessibility, and web-only runtime behavior.

`fullstack.spec.ts` exercises `examples/fullstack` (Button, Dialog, and Select
installed through the real `adico` CLI) for combined SSR and hydration: it
asserts the server-rendered HTML contains the installed markup, then confirms
the client wasm bundle hydrates and attaches interactivity with zero console
errors or warnings. Run `dx serve --platform web` from `examples/fullstack` in
one terminal, then in this directory:

```sh
ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:8080 npm run test:fullstack
```
