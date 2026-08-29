import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed Checkbox toggles on click and Space with correct ARIA state", async ({ page }) => {
  await page.goto("/");
  const checkbox = page.getByRole("checkbox", { name: "Accept terms" });
  await expect(checkbox).toHaveAttribute("aria-checked", "false");

  await checkbox.click();
  await expect(checkbox).toHaveAttribute("aria-checked", "true");

  await checkbox.focus();
  await page.keyboard.press("Space");
  await expect(checkbox).toHaveAttribute("aria-checked", "false");
});

test("installed Switch toggles on click and Space with correct ARIA state", async ({ page }) => {
  await page.goto("/");
  const toggle = page.getByRole("switch", { name: "Enable notifications" });
  await expect(toggle).toHaveAttribute("aria-checked", "false");

  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-checked", "true");

  await toggle.focus();
  await page.keyboard.press("Space");
  await expect(toggle).toHaveAttribute("aria-checked", "false");
});

test("installed Toggle toggles pressed state on click", async ({ page }) => {
  await page.goto("/");
  const toggle = page.getByRole("button", { name: "Bold" });
  await expect(toggle).toHaveAttribute("aria-pressed", "false");

  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-pressed", "true");
});

test("installed Collapsible expands and collapses via its trigger", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByRole("button", { name: "Toggle section" });
  const content = page.getByText("Collapsible content");
  await expect(trigger).toHaveAttribute("aria-expanded", "true");
  await expect(content).toBeVisible();

  await trigger.click();
  await expect(trigger).toHaveAttribute("aria-expanded", "false");
});

test("wave2-state fixture has zero critical accessibility violations", async ({ page }) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();
  const critical = results.violations.filter((violation) => violation.impact === "critical");
  expect(critical).toEqual([]);
});
