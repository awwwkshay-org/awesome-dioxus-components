import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed Select opens with its keyboard contract and selects the focused option", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByRole("button", { name: "Choose a fruit" });
  await expect(trigger).toHaveAttribute("aria-expanded", "false");

  await trigger.focus();
  await page.keyboard.press("ArrowDown");
  const listbox = page.getByRole("listbox", { name: "Fruit options" });
  await expect(listbox).toBeVisible();
  await expect(trigger).toHaveAttribute("aria-expanded", "true");

  await page.keyboard.press("Enter");
  await expect(listbox).toBeHidden();
  await expect(trigger).toContainText("Apple");
  await expect(trigger).toHaveAttribute("aria-expanded", "false");
});

test("installed Select supports typeahead, Escape, and accessible listbox markup", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByRole("button", { name: "Choose a fruit" });
  await trigger.click();
  const listbox = page.getByRole("listbox", { name: "Fruit options" });
  await expect(listbox).toBeVisible();

  await page.keyboard.press("b");
  await page.keyboard.press("Enter");
  await expect(trigger).toContainText("Banana");
  await expect(listbox).toBeHidden();

  await trigger.click();
  await page.keyboard.press("Escape");
  await expect(listbox).toBeHidden();

  await trigger.click();
  const accessibility = await new AxeBuilder({ page }).include('[role="listbox"]').analyze();
  expect(accessibility.violations).toEqual([]);
});
