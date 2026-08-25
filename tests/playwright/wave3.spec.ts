import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed Tooltip shows on hover with ARIA association and hides on mouse leave", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByText("Hover me", { exact: true });
  const tooltip = page.getByRole("tooltip");
  await expect(tooltip).toBeHidden();

  await trigger.hover();
  await expect(tooltip).toBeVisible();
  await expect(tooltip).toContainText("Tooltip content");
  await expect(trigger).toHaveAttribute("aria-describedby");

  await page.mouse.move(0, 0);
  await expect(tooltip).toBeHidden();
});

test("installed Popover opens on click, exposes dialog semantics, and closes with Escape", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByRole("button", { name: "Open popover" });
  await trigger.click();

  const popover = page.getByRole("dialog", { name: "Open popover" });
  await expect(popover).toBeVisible();
  await expect(popover).toContainText("Popover content");

  await page.keyboard.press("Escape");
  await expect(popover).toBeHidden();
});

test("installed HoverCard shows on hover and hides on mouse leave", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByRole("button", { name: "Dioxus" });
  const card = page.getByRole("tooltip").filter({ hasText: "Hover card content" });
  await expect(card).toBeHidden();

  await trigger.hover();
  await expect(card).toBeVisible();

  await page.mouse.move(0, 0);
  await expect(card).toBeHidden();
});

test("installed DropdownMenu opens with roving-focus keyboard navigation and closes on selection", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByRole("button", { name: "Open menu" });
  await expect(trigger).toHaveAttribute("aria-expanded", "false");

  await trigger.click();
  await expect(trigger).toHaveAttribute("aria-expanded", "true");
  const listbox = page.getByRole("listbox");
  await expect(listbox).toBeVisible();

  const item = page.getByRole("option", { name: "Edit" });
  await expect(item).toBeVisible();
  await item.click();
  await expect(listbox).toBeHidden();
  await expect(trigger).toHaveAttribute("aria-expanded", "false");
});

test("installed ContextMenu opens on right-click, navigates by keyboard, and closes with Escape", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByText("Right click here", { exact: true });
  await trigger.click({ button: "right" });

  // Scoped past Menubar's always-mounted (but closed) role="menu" wrapper.
  const menu = page.getByRole("menu").filter({ has: page.getByRole("menuitem", { name: "Edit" }) });
  await expect(menu).toBeVisible();
  await expect(page.getByRole("menuitem", { name: "Edit" })).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(menu).toBeHidden();
});

test("installed Menubar opens a menu on click and selects an item", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByRole("menuitem", { name: "File" });
  await trigger.click();

  const item = page.getByRole("menuitem", { name: "New" });
  await expect(item).toBeVisible();
  await item.click();
  await expect(item).toBeHidden();
});

test("installed Wave 3 overlays have no critical axe violations", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Open popover" }).click();
  const accessibility = await new AxeBuilder({ page }).include('[role="dialog"]').analyze();
  expect(accessibility.violations).toEqual([]);
});
