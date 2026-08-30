import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed ThemeSwitcher recolors --primary when a palette is selected", async ({
  page,
}) => {
  await page.goto("/");
  const rose = page.getByRole("radio", { name: "Rose palette" });
  await expect(rose).toBeVisible();

  await rose.click();
  await expect(rose).toHaveAttribute("aria-checked", "true");
  const primary = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--primary").trim(),
  );
  expect(primary).toContain("346.8");

  const violet = page.getByRole("radio", { name: "Violet palette" });
  await violet.click();
  await expect(violet).toHaveAttribute("aria-checked", "true");
  await expect(rose).toHaveAttribute("aria-checked", "false");
});

test("installed ThemeSwitcher has zero critical accessibility violations", async ({ page }) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();
  const critical = results.violations.filter((violation) => violation.impact === "critical");
  expect(critical).toEqual([]);
});
