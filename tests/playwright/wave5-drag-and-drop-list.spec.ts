import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed DragAndDropList reorders via keyboard (Enter, Arrow, Enter) and cancels with Escape", async ({
  page,
}) => {
  await page.goto("/");
  const items = page.getByRole("listitem");
  await expect(items).toHaveCount(3);
  const alpha = items.filter({ hasText: "Alpha" });

  await alpha.click();
  await expect(alpha).toHaveAttribute("aria-grabbed", "false");

  // Lift Alpha, move it down once, confirm the drop.
  await page.keyboard.press("Enter");
  await expect(alpha).toHaveAttribute("aria-grabbed", "true");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await expect(alpha).toHaveAttribute("aria-grabbed", "false");

  const reordered = await items.allTextContents();
  expect(reordered[0]).not.toBe("Alpha");
  expect(reordered).toContain("Alpha");

  // Lift the first item and cancel -- order must be unchanged.
  const beforeCancel = await items.allTextContents();
  await items.first().click();
  await page.keyboard.press("Enter");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Escape");
  const afterCancel = await items.allTextContents();
  expect(afterCancel).toEqual(beforeCancel);
});

test("installed DragAndDropList removes an item with Delete", async ({ page }) => {
  await page.goto("/");
  const items = page.getByRole("listitem");
  await items.first().click();
  await page.keyboard.press("Delete");
  await expect(items).toHaveCount(2);
});

test("wave5-drag-and-drop-list fixture has zero critical accessibility violations", async ({
  page,
}) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();
  const critical = results.violations.filter((violation) => violation.impact === "critical");
  expect(critical).toEqual([]);
});
