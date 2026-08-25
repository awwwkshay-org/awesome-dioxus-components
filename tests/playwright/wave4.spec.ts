import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed Combobox filters by typeahead and selects with the keyboard", async ({ page }) => {
  await page.goto("/");
  const input = page.getByRole("combobox");
  await expect(input).toHaveAttribute("aria-expanded", "false");

  await input.click();
  await expect(input).toHaveAttribute("aria-expanded", "true");
  const listbox = page.getByRole("listbox");
  await expect(listbox).toBeVisible();
  await expect(page.getByRole("option", { name: "Apple" })).toBeVisible();
  await expect(page.getByRole("option", { name: "Banana" })).toBeVisible();

  await input.fill("ban");
  await expect(page.getByRole("option", { name: "Apple" })).toBeHidden();
  await expect(page.getByRole("option", { name: "Banana" })).toBeVisible();

  await input.press("ArrowDown");
  await input.press("Enter");
  await expect(listbox).toBeHidden();
  await expect(input).toHaveValue("Banana");
  await expect(input).toHaveAttribute("aria-expanded", "false");
});

test("installed Calendar navigates and selects dates with arrow keys", async ({ page }) => {
  await page.goto("/");
  const grid = page.getByRole("grid");
  await expect(grid).toBeVisible();

  const focusable = grid.locator('button[tabindex="0"]');
  const initialLabel = await focusable.getAttribute("aria-label");
  await focusable.focus();
  await page.keyboard.press("ArrowRight");

  const nextFocusable = grid.locator('button[tabindex="0"]');
  await expect(nextFocusable).toBeFocused();
  const nextLabel = await nextFocusable.getAttribute("aria-label");
  expect(nextLabel).not.toBe(initialLabel);

  await page.keyboard.press("Enter");
  await expect(nextFocusable).toHaveAttribute("data-selected", "true");
});

test("installed DatePicker opens a Calendar inside its popover", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByRole("button", { name: "Select date" });
  await trigger.click();

  const popover = page.getByRole("dialog");
  await expect(popover).toBeVisible();
  await expect(popover.getByRole("grid")).toBeVisible();
});

test("installed Sidebar toggles open/closed state from its trigger", async ({ page }) => {
  await page.goto("/");
  const sidebar = page.locator('[data-slot="sidebar"]');
  await expect(sidebar).toHaveAttribute("data-state", "expanded");

  const trigger = page.locator('[data-slot="sidebar-trigger"]');
  await trigger.click();
  await expect(sidebar).toHaveAttribute("data-state", "collapsed");

  await trigger.click();
  await expect(sidebar).toHaveAttribute("data-state", "expanded");
});

test("installed Wave 4 collection components have no critical axe violations", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("combobox").click();
  const accessibility = await new AxeBuilder({ page })
    .include('[role="listbox"]')
    .include('[role="grid"]')
    .analyze();
  expect(accessibility.violations).toEqual([]);
});
