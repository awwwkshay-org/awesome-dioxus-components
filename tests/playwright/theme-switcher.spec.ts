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
  // Scoped to the component's own container (`#theme-switcher-demo`), not
  // the whole page -- see the matching comment in mode-toggle.spec.ts for
  // why: examples/basic-spa and examples/basic-ssr render this alongside
  // every other migrated registry item, and a page-wide scan would surface
  // unrelated pre-existing issues elsewhere as false failures here.
  const results = await new AxeBuilder({ page }).include("#theme-switcher-demo").analyze();
  const critical = results.violations.filter((violation) => violation.impact === "critical");
  expect(critical).toEqual([]);
});
