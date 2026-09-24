# Follow-ups discovered by this change

Two defects were found while implementing `redesign-web-visual-foundation`.
Neither is fixed here — both live in `packages/`, which this change deliberately
does not touch — and neither is blocking. They are recorded here rather than as
scaffolded change directories because `openspec new change` produces a
zero-delta change, and a zero-delta change without `skip_specs: true` makes
`openspec validate --all --strict` fail (verified: 16 passed / 1 failed with a
stub present, 16/0 with it removed). Leaving the repo's own validation gate red
to hold a placeholder is worse than writing them down.

---

## 1. The `dark` custom variant belongs in the CLI, not in each app

**Where:** `packages/adico-cli/src/css.rs`, `theme_region()` (css.rs:538-675).

**What:** The generated theme region emits `.dark { --… }` token overrides, and
`packages/adico-primitives/src/theme_mode.rs:165-176` applies the `dark` class
to `document.documentElement`. But nothing emits
`@custom-variant dark (&:is(.dark *))`, so Tailwind v4's *default* `dark`
variant applies and every literal `dark:` utility compiles to
`@media (prefers-color-scheme: dark)`.

**Consequence:** token-driven colors follow the app's theme control while
literal `dark:` utilities follow the operating system. For any user whose OS
theme differs from their chosen theme, the two disagree. Affected utilities are
the ones shipped in registry source: `dark:hover:bg-accent/50` (Button ghost),
`dark:aria-invalid:ring-destructive/40`, `dark:hover:bg-white/10`,
`dark:text-emerald-500`, and the `dark:focus-visible:ring-*/40` set.

**Evidence:** before this change,
`apps/web/assets/tailwind.css:3490-3492` read
`.dark\:hover\:bg-accent\/50 { @media (prefers-color-scheme: dark) { … } }`.

**Interim:** `apps/web/tailwind.css` declares the variant in its app-owned
prefix, which fixes the site but not any other consumer.

**Fix:** emit the `@custom-variant` from `theme_region()` so every project that
runs `adico add` inherits it, then drop the app-level declaration. Note this is
a **visible rendering change** for existing consumer projects, so it wants its
own proposal rather than a silent patch.

---

## 2. ~~`adico add --replace` collapses per-item `manifestDigest` values~~ — RETRACTED, not a defect

**This was a misdiagnosis. There is no bug here.** Recorded rather than deleted,
so the same wrong conclusion is not reached again.

The original report: `adico add button --replace` rewrote `apps/web/adico.lock`
so that four items which previously had three *distinct* `manifestDigest`
values all shared one, which looked like a per-item digest being computed over
the whole plan.

It is computed over the whole manifest, and that is the intent.
`packages/adico-registry-core/src/lib.rs:700` sets
`manifest_digest: sha256_hex(manifest_bytes)` on the *registry*, and
lib.rs:848/920 clone that single value onto every resolved item. The field
records **which registry manifest version an item was installed from**, not
anything about the item.

So items installed in the same command necessarily share a digest, and items
installed at different times differ because the manifest changed in between.
Verified directly while fixing the carousel/time-picker measurement races:
after re-installing 8 items, their shared new digest
`a4f7a2c9baaf677c19c6201c83dc7f0c33afb6fed355de4ba486e19f65a3bd15` is
byte-identical to `shasum -a 256 packages/adico-cli/embedded/registry.json`,
and the lock as a whole holds 13 distinct digests across its 71 items — one per
historical manifest version, exactly as designed.

The lockfile change that prompted this report was correct and should not have
been reverted. Nothing to fix.
