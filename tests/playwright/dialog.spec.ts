import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed Dialog opens, exposes ARIA semantics, restores focus, and closes with Escape", async ({ page }) => {
  await page.goto("/");
  const trigger = page.getByRole("button", { name: "Open dialog" });
  await trigger.click();

  const dialog = page.getByRole("dialog", { name: "Edit profile" });
  await expect(dialog).toBeVisible();
  await expect(dialog).toHaveAttribute("aria-describedby");
  await expect(dialog).toContainText("Update your information.");
  await expect(page.locator("html")).toHaveCSS("overflow", "hidden");
  await expect(trigger).toBeFocused();

  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(page.locator("html")).not.toHaveCSS("overflow", "hidden");
  await expect(trigger).toBeFocused();
});

test("installed Dialog traps Tab focus within its content and wraps at both ends", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Open dialog" }).click();
  const dialog = page.getByRole("dialog", { name: "Edit profile" });
  await expect(dialog).toBeVisible();

  const save = page.getByRole("button", { name: "Save changes" });
  const nestedTrigger = page.getByRole("button", { name: "Open nested dialog" });
  await expect(save).toBeFocused();

  await page.keyboard.press("Tab");
  await expect(nestedTrigger).toBeFocused();

  // Forward from the last focusable item wraps back to the first, rather
  // than escaping the trap onto the page's own "Open dialog" trigger.
  await page.keyboard.press("Tab");
  await expect(save).toBeFocused();

  // Shift+Tab from the first item wraps backward to the last.
  await page.keyboard.press("Shift+Tab");
  await expect(nestedTrigger).toBeFocused();

  await page.keyboard.press("Shift+Tab");
  await expect(save).toBeFocused();
});

test("installed Dialog closes after outside interaction and has no critical axe violations", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Open dialog" }).click();
  const dialog = page.getByRole("dialog", { name: "Edit profile" });
  await expect(dialog).toBeVisible();

  await page.locator('[data-adico-dialog-overlay="true"]').click({ position: { x: 5, y: 5 } });
  await expect(dialog).toBeHidden();

  await page.getByRole("button", { name: "Open dialog" }).click();
  const accessibility = await new AxeBuilder({ page }).include('[role="dialog"]').analyze();
  expect(accessibility.violations).toEqual([]);
});

test("installed nested Dialog closes only the active layer with Escape", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Open dialog" }).click();
  const outerDialog = page.getByRole("dialog", { name: "Edit profile" });
  await expect(outerDialog).toBeVisible();

  await page.getByRole("button", { name: "Open nested dialog" }).click();
  const nestedDialog = page.getByRole("dialog", { name: "Nested dialog" });
  await expect(nestedDialog).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(nestedDialog).toBeHidden();
  await expect(outerDialog).toBeVisible();
});
