import { expect, test, type Page } from "@playwright/test";

// `make-registry-components-mobile-first`: asserts no registry component
// extends past a 375px viewport's horizontal bounds. Runs against the
// playground's shell-free `/responsive/...` harness routes (see
// `apps/playground/src/pages/responsive_flow.rs` / `responsive_overlay.rs`),
// never the normal demo pages -- the demo pages sit inside a shell whose
// root is `overflow-hidden` and whose demo canvas is deliberately
// `overflow-visible`, either of which would make a naive `scrollWidth` check
// pass even when a component visibly overflows. This spec instead walks
// each fixture's rendered descendants and asserts every element's bounding
// rect stays within [0, viewport width] -- see design.md's D7.
//
// This spec is EXPECTED TO FAIL on the known-broken list (dialog,
// alert-dialog, calendar, toast, tabs) until Waves 1-6 land their fixes --
// that failure is the point: a check that has never been seen to fail is
// not trusted to cover the other ~54 components in Wave 6.

const OVERLAY_CASES = [
  "dialog",
  "alert-dialog",
  "sheet",
  "drawer",
  "toast",
  "popover",
  "hover-card",
  "command",
  "time-picker"
];

// Default scope is `[data-responsive-case]` (the first match) rather than
// `body`: `dx serve` injects its own devtools reconnect/hot-reload toast
// directly into `document.body`, outside the app's own DOM -- scoping to
// `body` sweeps that unrelated UI into the walk. Callers that need to check
// every in-flow case on one page pass an explicit wrapper selector instead
// (see `responsive-flow-root` below), never `body`.
async function assertNoHorizontalOverflow(page: Page, viewportWidth: number, scopeSelector = "[data-responsive-case]") {
  const offenders = await page.evaluate(
    ({ width, scopeSelector }) => {
    const found: string[] = [];
    const root = document.querySelector(scopeSelector);
    if (!root) return found;

    // A horizontally-scrollable container (e.g. TabsList) legitimately has
    // children positioned outside the viewport -- they're clipped by the
    // container's own `overflow-x: auto`, not spilling onto the page. Check
    // the scroll container itself for viewport bounds, but don't descend
    // into it: its children being individually "outside the viewport" is
    // the intended behavior of a horizontal scroller, not an overflow bug.
    function isHorizontalScrollContainer(el: HTMLElement): boolean {
      const style = getComputedStyle(el);
      return (
        (style.overflowX === "auto" || style.overflowX === "scroll") &&
        el.scrollWidth > el.clientWidth
      );
    }

    function walk(el: HTMLElement) {
      if (el.getAttribute("aria-hidden") === "true") return;
      const rect = el.getBoundingClientRect();
      if (rect.width > 0 && rect.height > 0 && (rect.right > width + 1 || rect.left < -1)) {
        found.push(
          `${el.tagName.toLowerCase()}${el.className ? "." + String(el.className).split(" ").join(".") : ""} ` +
            `(left=${rect.left.toFixed(1)}, right=${rect.right.toFixed(1)}, viewport=${width})`
        );
      }
      if (isHorizontalScrollContainer(el)) return; // don't flag clipped, scrollable children
      for (const child of Array.from(el.children)) {
        walk(child as HTMLElement);
      }
    }
    walk(root as HTMLElement);
    return found;
    },
    { width: viewportWidth, scopeSelector }
  );

  expect(offenders, `overflowing element(s): ${offenders.join(" | ")}`).toEqual([]);
}

test.describe("in-flow components at 375px", () => {
  test("no rendered element extends past the viewport", async ({ page }) => {
    await page.goto("/responsive/flow");
    await page.waitForTimeout(300); // let mount-time transitions settle
    await assertNoHorizontalOverflow(page, 375, "#responsive-flow-root");
  });

  test("tabs list exposes horizontal scroll rather than clipping", async ({ page }) => {
    await page.goto("/responsive/flow");
    const list = page.locator('[data-responsive-case="tabs"] [role="tablist"]');
    const overflowX = await list.evaluate((el) => getComputedStyle(el).overflowX);
    const isScrollable = await list.evaluate((el) => el.scrollWidth > el.clientWidth);
    expect(isScrollable, "5 real tab labels should overflow a 375px list").toBe(true);
    expect(overflowX, "TabsList should scroll, not clip, its overflow").toBe("auto");
  });

  test("data table toolbar and pagination footer do not overflow", async ({ page }) => {
    await page.goto("/responsive/flow");
    const table = page.locator('[data-responsive-case="data-table"]');
    await expect(table.getByText(/of 7 row/)).toBeVisible();
    await assertNoHorizontalOverflow(page, 375, '[data-responsive-case="data-table"]');
  });

  test("card renders a single column at 375px", async ({ page }) => {
    await page.goto("/responsive/flow");
    // `CardHeader` renders a real `<header>` element, not a `<div>` -- a
    // `> div` selector here never matched anything and silently timed out
    // instead of asserting (a bug present since this spec was first
    // written; caught only once Card's own fix made this test the last one
    // still failing). `[class*="grid-cols-"]` is unambiguous: it's the one
    // R6 class this wave's `@sm:has-[[data-slot=card-action]]:*` compound
    // touches.
    const header = page.locator('[data-responsive-case="card"] [class*="grid-cols-"]').first();
    const columns = await header.evaluate((el) => getComputedStyle(el).gridTemplateColumns);
    const trackCount = columns === "none" ? 1 : columns.trim().split(/\s+/).length;
    expect(trackCount, `card header grid should collapse to 1 column at 375px, got "${columns}"`).toBe(1);
  });
});

for (const overlayCase of OVERLAY_CASES) {
  test(`overlay case "${overlayCase}" does not extend past the viewport`, async ({ page }) => {
    await page.goto(`/responsive/overlay?case=${overlayCase}`);
    await page.waitForTimeout(300);
    await assertNoHorizontalOverflow(page, 375);
  });
}

test("dialog keeps a viewport gutter at 375px", async ({ page }) => {
  await page.goto("/responsive/overlay?case=dialog");
  const content = page.getByRole("dialog");
  await expect(content).toBeVisible();
  const rect = await content.evaluate((el) => el.getBoundingClientRect());
  expect(rect.left, "dialog should have at least a 1rem (16px) left gutter").toBeGreaterThanOrEqual(15);
  expect(rect.right, "dialog should have at least a 1rem (16px) right gutter").toBeLessThanOrEqual(360);
});

test("alert dialog keeps a viewport gutter at 375px", async ({ page }) => {
  await page.goto("/responsive/overlay?case=alert-dialog");
  const content = page.getByRole("alertdialog");
  await expect(content).toBeVisible();
  const rect = await content.evaluate((el) => el.getBoundingClientRect());
  expect(rect.left, "alert dialog should have at least a 1rem (16px) left gutter").toBeGreaterThanOrEqual(15);
  expect(rect.right, "alert dialog should have at least a 1rem (16px) right gutter").toBeLessThanOrEqual(360);
});

test("toast stays on-screen at its left edge at 375px", async ({ page }) => {
  await page.goto("/responsive/overlay?case=toast");
  const toast = page.getByText("Saved");
  await expect(toast).toBeVisible();
  const rect = await toast.evaluate((el) => el.closest("li")?.getBoundingClientRect() ?? el.getBoundingClientRect());
  expect(rect.left, "toast's left edge should not be pushed off-screen").toBeGreaterThanOrEqual(-1);
});

test("calendar fits within the viewport at 375px", async ({ page }) => {
  await page.goto("/responsive/flow");
  const calendar = page.locator('[data-responsive-case="calendar"] [role="grid"]').first();
  const rect = await calendar.evaluate((el) => el.closest('[class*="border"]')?.getBoundingClientRect() ?? el.getBoundingClientRect());
  expect(rect.width, "calendar should fit within a 375px viewport with room for gutters").toBeLessThanOrEqual(343);
});
