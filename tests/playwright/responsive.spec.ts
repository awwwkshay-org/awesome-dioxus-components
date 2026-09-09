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

async function assertNoHorizontalOverflow(page: Page, viewportWidth: number) {
  const offenders = await page.evaluate((width) => {
    const found: string[] = [];
    const root = document.querySelector("[data-responsive-case]");
    if (!root) return found;
    const elements = root.querySelectorAll<HTMLElement>("*");
    for (const el of Array.from(elements)) {
      if (el.getAttribute("aria-hidden") === "true") continue;
      const rect = el.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) continue;
      if (rect.right > width + 1 || rect.left < -1) {
        found.push(
          `${el.tagName.toLowerCase()}${el.className ? "." + String(el.className).split(" ").join(".") : ""} ` +
            `(left=${rect.left.toFixed(1)}, right=${rect.right.toFixed(1)}, viewport=${width})`
        );
      }
    }
    return found;
  }, viewportWidth);

  expect(offenders, `overflowing element(s): ${offenders.join(" | ")}`).toEqual([]);
}

test.describe("in-flow components at 375px", () => {
  test("no rendered element extends past the viewport", async ({ page }) => {
    await page.goto("/responsive/flow");
    await page.waitForTimeout(300); // let mount-time transitions settle
    await assertNoHorizontalOverflow(page, 375);
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
    await assertNoHorizontalOverflow(page, 375);
  });

  test("card renders a single column at 375px", async ({ page }) => {
    await page.goto("/responsive/flow");
    const header = page.locator('[data-responsive-case="card"] > div').first();
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
