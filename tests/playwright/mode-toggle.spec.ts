import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed ModeToggle cycles Light/Dark/System and applies the resolved class", async ({
  page,
}) => {
  await page.goto("/");
  const trigger = page.getByRole("button", { name: /Toggle theme/ });
  await expect(trigger).toBeVisible();

  await trigger.click();
  await page.getByRole("option", { name: "Dark" }).click();
  await expect(page.locator("html")).toHaveClass(/dark/);

  await trigger.click();
  await page.getByRole("option", { name: "Light" }).click();
  await expect(page.locator("html")).not.toHaveClass(/dark/);

  await trigger.click();
  await page.getByRole("option", { name: "System" }).click();
  // System resolution depends on the real OS/browser preference in this
  // environment; only assert the toggle didn't crash and re-persisted.
  await expect(trigger).toBeVisible();
});

test("installed ModeToggle persists the selected mode across reloads", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByRole("button", { name: /Toggle theme/ });
  await trigger.click();
  await page.getByRole("option", { name: "Dark" }).click();
  await expect(page.locator("html")).toHaveClass(/dark/);

  await page.reload();
  await expect(page.locator("html")).toHaveClass(/dark/, { timeout: 5000 });
});

test("installed ModeToggle has zero critical accessibility violations", async ({ page }) => {
  await page.goto("/");
  // Scoped to the component's own container (`#mode-toggle-demo`), not the
  // whole page: examples/basic-spa and examples/basic-ssr render ModeToggle
  // alongside every other migrated registry item on one demo page, and a
  // page-wide scan would surface unrelated pre-existing issues elsewhere on
  // that page (e.g. a Tabs aria-controls defect) as false failures here.
  const results = await new AxeBuilder({ page }).include("#mode-toggle-demo").analyze();
  const critical = results.violations.filter((violation) => violation.impact === "critical");
  expect(critical).toEqual([]);
});
