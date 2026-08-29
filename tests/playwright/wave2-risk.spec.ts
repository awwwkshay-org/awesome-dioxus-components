import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed ScrollArea scrolls its content", async ({ page }) => {
  await page.goto("/");
  const firstItem = page.getByText("Scrollable item 1", { exact: true });
  const scrollArea = firstItem.locator("../..");

  await expect(firstItem).toBeVisible();
  expect(await scrollArea.evaluate((el) => el.scrollTop)).toBe(0);

  await scrollArea.evaluate((el) => {
    el.scrollTop = el.scrollHeight;
  });

  expect(await scrollArea.evaluate((el) => el.scrollTop)).toBeGreaterThan(0);
});

test("installed AlertDialog opens via its trigger, closes via Cancel, and closes with Escape", async ({
  page,
}) => {
  await page.goto("/");
  const trigger = page.getByRole("button", { name: "Delete item" });
  const dialog = page.getByRole("alertdialog", { name: "Delete item" });

  await trigger.click();
  await expect(dialog).toBeVisible();
  await expect(dialog).toHaveAttribute("aria-modal", "true");

  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(dialog).toBeHidden();

  await trigger.click();
  await expect(dialog).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
});

test("installed AlertDialog's Delete action closes it", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Delete item" }).click();
  const dialog = page.getByRole("alertdialog", { name: "Delete item" });
  await expect(dialog).toBeVisible();
  await page.getByRole("button", { name: "Delete", exact: true }).click();
  await expect(dialog).toBeHidden();
});

test("installed Toast appears after use_toast().info(...)", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Show toast" }).click();
  await expect(page.getByRole("alertdialog", { name: "Saved" })).toBeVisible();
});

test("installed Slider moves value with arrow keys", async ({ page }) => {
  await page.goto("/");
  const thumb = page.getByRole("slider", { name: "Volume" });
  await expect(thumb).toHaveAttribute("aria-valuenow", "50");
  await thumb.focus();
  await page.keyboard.press("ArrowRight");
  await expect(thumb).toHaveAttribute("aria-valuenow", "51");
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("ArrowLeft");
  await expect(thumb).toHaveAttribute("aria-valuenow", "49");
});

test("wave2-risk fixture has zero critical accessibility violations", async ({ page }) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();
  const critical = results.violations.filter((violation) => violation.impact === "critical");
  expect(critical).toEqual([]);
});
