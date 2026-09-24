# Vendored fonts

These are redistributed, unmodified font binaries — not ported source — so they
carry a license notice and this origin record rather than a
`provenance/records/*.json` entry (that schema is for imported/ported *source*,
and its checker requires each recorded local path to be a readable text file
containing the upstream revision, which a `.woff2` cannot be). See
`UPSTREAMS.md` § "Vendored assets".

| File | Family | Axis | Subset |
| --- | --- | --- | --- |
| `geist-latin-wght-normal.woff2` | Geist | `wght` 100–900, normal | latin |
| `geist-mono-latin-wght-normal.woff2` | Geist Mono | `wght` 100–900, normal | latin |

- **Typeface source:** <https://github.com/vercel/geist-font>
- **Obtained from:** `@fontsource-variable/geist@5.3.0` and
  `@fontsource-variable/geist-mono` (Fontsource's latin-subset variable `woff2`
  builds), fetched via jsDelivr on 2026-09-24.
- **License:** SIL Open Font License 1.1 — full text in `OFL.txt`.
- **Copyright:** Copyright 2024 The Geist Project Authors
  (<https://github.com/vercel/geist-font>).

No glyph, metric, or axis modification was made locally; the files are byte
copies of Fontsource's published subset builds. Italic, cyrillic, latin-ext,
and vietnamese subsets are deliberately not vendored — nothing in `apps/web`
references them, and each would add weight to every page load.

## Replacing or updating

Re-download both files at the same package version, keep the filenames, and
update the version and date above. The `@font-face` rules that consume them
live in the app-owned prefix block of `apps/web/tailwind.css`, **above**
`/* adico:theme:start */` — never inside the marker region, which `adico add`
regenerates wholesale.
