import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed Toolbar roves focus with ArrowRight (horizontal default) and clicks activate", async ({
  page,
}) => {
  await page.goto("/");
  const bold = page.getByRole("button", { name: "Bold" });
  const italic = page.getByRole("button", { name: "Italic" });

  await bold.click();
  await expect(bold).toBeFocused();

  await page.keyboard.press("ArrowRight");
  await expect(italic).toBeFocused();

  await page.keyboard.press("ArrowLeft");
  await expect(bold).toBeFocused();
});

test("installed VirtualList renders a virtualized window of list items with ARIA metadata", async ({
  page,
}) => {
  const consoleErrors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      consoleErrors.push(message.text());
    }
  });

  await page.goto("/");
  const list = page.getByRole("list");
  await expect(list).toBeVisible();

  const items = page.getByRole("listitem");
  const count = await items.count();
  // 200 rows requested; virtualization must render fewer than the full count.
  expect(count).toBeGreaterThan(0);
  expect(count).toBeLessThan(200);

  const first = items.first();
  await expect(first).toHaveAttribute("aria-setsize", "200");
  await expect(first).toHaveAttribute("aria-posinset", "1");

  expect(consoleErrors).toEqual([]);
});

test("wave5-extras fixture has zero critical accessibility violations", async ({ page }) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();
  const critical = results.violations.filter((violation) => violation.impact === "critical");
  expect(critical).toEqual([]);
});
