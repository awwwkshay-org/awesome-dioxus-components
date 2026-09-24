import { expect, test } from "@playwright/test";

// `make-playground-shell-responsive`: asserts the playground's own app
// shell (routes.rs's `PlaygroundLayout`, not a registry component) is usable at
// 375px. The `>= md` nav column (registry/ui/sidebar.rs's
// `SidebarContent`, composed inside the resizable panel) is not rendered;
// a hamburger opens the full nav list in a `Sheet`; selecting an entry
// navigates and closes the sheet; nothing on the shell causes horizontal
// document overflow. See
// openspec/changes/make-playground-shell-responsive/design.md.

test("the >= md nav column is not visible at 375px", async ({ page }) => {
  await page.goto("/playground");
  await expect(page.locator('[data-slot="sidebar-content"]')).not.toBeVisible();
});

test("the hamburger opens the full nav list, fully legible", async ({ page }) => {
  await page.goto("/playground");
  const trigger = page.getByRole("button", { name: "Open navigation" });
  await expect(trigger).toBeVisible();
  await trigger.click();

  const nav = page.getByRole("dialog");
  await expect(nav).toBeVisible();
  const button = nav.getByRole("button", { name: "Button", exact: true });
  await expect(button).toBeVisible();
  // Not truncated to a fragment of its label (the pre-fix defect this
  // change closes): the full accessible name must render, not "B...".
  await expect(button).toHaveText("Button");
  await expect(nav.getByRole("button", { name: "Accordion", exact: true })).toBeVisible();
});

test("selecting a nav entry navigates and closes the sheet", async ({ page }) => {
  await page.goto("/playground");
  await page.getByRole("button", { name: "Open navigation" }).click();
  await page.getByRole("dialog").getByRole("button", { name: "Button", exact: true }).click();

  await expect(page).toHaveURL(/\/button$/);
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Button", exact: true })).toBeVisible();
});

test("no element on the shell causes horizontal document overflow", async ({ page }) => {
  await page.goto("/playground");
  const hasOverflow = () =>
    page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1);

  expect(await hasOverflow(), "closed shell should not overflow").toBe(false);

  await page.getByRole("button", { name: "Open navigation" }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  expect(await hasOverflow(), "shell with the nav sheet open should not overflow").toBe(false);
});
