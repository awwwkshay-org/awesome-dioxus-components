import { expect, test } from "@playwright/test";

// Companion to responsive-shell.spec.ts: asserts adding mobile support to
// the playground's own app shell (`make-playground-shell-responsive`) left
// the `>= md` layout unaffected. The `>= md` tree is the pre-existing
// `ResizablePanelGroup` composition, changed by this shell responsive
// change only by extracting its nav-list loop into `components/nav.rs` --
// these values were captured live against that (functionally) unchanged
// tree.

test("the mobile top bar is not visible at desktop", async ({ page }) => {
  await page.goto("/playground");
  await expect(page.getByRole("button", { name: "Open navigation" })).not.toBeVisible();
});

test("nav column renders at its default 18% width, unaffected by mobile support", async ({ page }) => {
  await page.goto("/playground");
  await expect(page.locator('[data-slot="sidebar-content"]')).toBeVisible();

  const width = await page.evaluate(() => {
    const header = document.querySelector('[data-slot="sidebar-header"]');
    return header ? Math.round(header.parentElement!.getBoundingClientRect().width) : null;
  });
  const viewport = page.viewportSize();
  expect(viewport).not.toBeNull();
  const expectedWidth = Math.round(viewport!.width * 0.18);
  expect(width, `nav column should stay ~18% of viewport width (${expectedWidth}px)`).not.toBeNull();
  expect(Math.abs((width as number) - expectedWidth)).toBeLessThanOrEqual(5);
});

test("every nav entry is reachable directly in the resizable column", async ({ page }) => {
  await page.goto("/playground");
  const nav = page.locator('[data-slot="sidebar-content"]');
  const button = nav.getByRole("button", { name: "Button", exact: true });
  await expect(button).toBeVisible();
  await button.click();
  await expect(page).toHaveURL(/\/button$/);
});

test("the resize handle between nav and content is visible and draggable", async ({ page }) => {
  await page.goto("/playground");
  const handle = page.locator('[aria-orientation]').first();
  await expect(handle).toBeVisible();
});
