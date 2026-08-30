import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed TagGroupMulti roves focus with ArrowRight, toggles selection, and removes a tag", async ({
  page,
}) => {
  await page.goto("/");
  const bug = page.getByRole("row", { name: /bug/ });
  const feature = page.getByRole("row", { name: /feature/ });

  await expect(bug).toHaveAttribute("aria-selected", "true");
  await expect(feature).toHaveAttribute("aria-selected", "false");

  await bug.click();
  await page.keyboard.press("ArrowRight");
  await expect(feature).toBeFocused();

  await page.keyboard.press("Enter");
  await expect(feature).toHaveAttribute("aria-selected", "true");

  await page.getByRole("button", { name: "Remove item feature" }).click();
  await expect(feature).toHaveCount(0);
});

test("wave5-tag-group fixture has zero critical accessibility violations", async ({ page }) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();
  const critical = results.violations.filter((violation) => violation.impact === "critical");
  expect(critical).toEqual([]);
});
