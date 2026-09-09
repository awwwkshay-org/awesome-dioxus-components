import { expect, test } from "@playwright/test";

// `make-registry-components-mobile-first`: companion to responsive.spec.ts.
// Asserts today's actual desktop-viewport (1280px) computed geometry, so a
// responsive override that unintentionally also changes desktop rendering
// is caught automatically. Every value below was captured LIVE against the
// unmodified registry, BEFORE Wave 1 touched any component -- see
// design.md's D8 for why the baseline must be captured before, not written
// against already-changed code. When a later wave intentionally changes one
// of these values (the mobile-first rewrite forms in
// docs/adico/mobile-first-rules.md restore the ORIGINAL value at `sm:` and
// above, so in the common case nothing here should change), that wave's own
// commit updates the specific assertion, with a comment explaining why.

test("dialog keeps its max-width at desktop", async ({ page }) => {
  await page.goto("/responsive/overlay?case=dialog");
  const content = page.getByRole("dialog");
  await expect(content).toBeVisible();
  const box = await content.evaluate((el) => ({
    width: Math.round(el.getBoundingClientRect().width),
    maxWidth: getComputedStyle(el).maxWidth
  }));
  expect(box.maxWidth).toBe("512px"); // max-w-lg
});

test("alert dialog keeps its max-width at desktop", async ({ page }) => {
  await page.goto("/responsive/overlay?case=alert-dialog");
  const content = page.getByRole("alertdialog");
  await expect(content).toBeVisible();
  const maxWidth = await content.evaluate((el) => getComputedStyle(el).maxWidth);
  expect(maxWidth).toBe("512px"); // AlertDialogContentSize::Default => max-w-lg
});

test("popover keeps its width at desktop", async ({ page }) => {
  await page.goto("/responsive/overlay?case=popover");
  // Popover's `zoom-in-95` entrance animation means an immediate measurement
  // reads ~95% of the settled width -- wait for the animation to finish.
  await page.waitForTimeout(300);
  const width = await page.evaluate(() => {
    const el = document.querySelector('[class*="w-72"]');
    return el ? Math.round(el.getBoundingClientRect().width) : null;
  });
  expect(width).toBe(288); // w-72
});

test("hover card keeps its width at desktop", async ({ page }) => {
  await page.goto("/responsive/overlay?case=hover-card");
  await page.waitForTimeout(300); // same zoom-in-95 entrance animation as popover
  const width = await page.evaluate(() => {
    const el = document.querySelector('[class*="w-64"]');
    return el ? Math.round(el.getBoundingClientRect().width) : null;
  });
  expect(width).toBe(256); // w-64
});

test("toast keeps its max-width at desktop", async ({ page }) => {
  await page.goto("/responsive/overlay?case=toast");
  await expect(page.getByText("Saved")).toBeVisible();
  const maxWidth = await page.evaluate(() => {
    const el = document.querySelector('[class*="max-w-\\[420px\\]"]');
    return el ? getComputedStyle(el).maxWidth : null;
  });
  expect(maxWidth).toBe("420px");
});

test("calendar keeps its fixed dimensions at desktop", async ({ page }) => {
  await page.goto("/responsive/flow");
  const box = await page.evaluate(() => {
    const el = document.querySelector('[data-responsive-case="calendar"] [class*="w-[18rem]"]');
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return { width: Math.round(r.width), height: Math.round(r.height) };
  });
  expect(box).toEqual({ width: 288, height: 320 }); // w-[18rem] h-[20rem]
});

test("carousel vertical orientation keeps its height at desktop", async ({ page }) => {
  await page.goto("/responsive/flow");
  const height = await page.evaluate(() => {
    const el = document.querySelector('[data-responsive-case="carousel"] [class*="h-[24rem]"]');
    return el ? Math.round(el.getBoundingClientRect().height) : null;
  });
  expect(height).toBe(384); // h-[24rem]
});

test("card header grid keeps two tracks at desktop", async ({ page }) => {
  await page.goto("/responsive/flow");
  const trackCount = await page.evaluate(() => {
    const el = document.querySelector('[data-responsive-case="card"] [class*="grid-cols-"]') as HTMLElement | null;
    if (!el) return null;
    const cols = getComputedStyle(el).gridTemplateColumns;
    return cols === "none" ? 1 : cols.trim().split(/\s+/).length;
  });
  expect(trackCount).toBe(2); // has-[[data-slot=card-action]]:grid-cols-[1fr_auto]
});

test("tabs list scrolls rather than clips, but is visually unaffected at desktop", async ({ page }) => {
  await page.goto("/responsive/flow");
  const result = await page.evaluate(() => {
    const el = document.querySelector('[data-responsive-case="tabs"] [role="tablist"]') as HTMLElement | null;
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return {
      overflowX: getComputedStyle(el).overflowX,
      scrolls: el.scrollWidth > el.clientWidth,
      width: Math.round(r.width)
    };
  });
  // Wave 4 added `overflow-x-auto` to TabsList (was "visible") -- an
  // intentional computed-style change, not a regression: at desktop all 5
  // tabs fit without scrolling (scrollWidth === clientWidth), so the visible
  // layout is unchanged even though the CSS property differs.
  expect(result?.overflowX).toBe("auto");
  expect(result?.scrolls, "5 tabs should fit without scrolling at desktop width").toBe(false);
});

test("sidebar open width is the intended 256px at desktop", async ({ page }) => {
  await page.goto("/responsive/flow");
  const width = await page.evaluate(() => {
    const el = document.querySelector('[data-responsive-case="sidebar"] aside');
    return el ? Math.round(el.getBoundingClientRect().width) : null;
  });
  // Wave 5 fixed `registry/ui/sidebar.rs`'s `w-[--sidebar-width]` -- which
  // compiled under Tailwind v4.1.5 to the INVALID declaration
  // `width: --sidebar-width` (missing `var(...)`), silently dropped by the
  // browser, so the open Sidebar rendered at its shrink-to-fit content width
  // (~139-143px) instead of the intended 16rem -- to `w-(--sidebar-width)`,
  // v4's own var-reference syntax. Confirmed fixed: this now measures the
  // correct 256px. (This assertion originally read `toBeLessThan(200)` to
  // record the pre-fix baseline; flipped here per that comment's own plan.)
  expect(width).toBe(256);
});
