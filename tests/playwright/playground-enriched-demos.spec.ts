import { expect, test } from "@playwright/test";

// Runs against the playground app (`dx serve` from `apps/playground`,
// then `ADICO_PLAYWRIGHT_BASE_URL=<url> npm run test:playground-enriched`).
// Covers the two behaviors this change added to registry components that
// only a real browser can exercise: carousel pointer-drag paging and
// input-otp masked slots.

const track = (page: import("@playwright/test").Page) =>
  page.locator('[aria-roledescription="carousel"] [tabindex="0"]');

test("installed Carousel pages one slide on a drag past the threshold", async ({ page }) => {
  await page.goto("/carousel");
  const content = track(page);
  await expect(content).toBeVisible();
  await expect(content).toHaveCSS("cursor", "grab");

  const box = await content.boundingBox();
  if (!box) throw new Error("carousel track has no bounding box");
  const startX = box.x + box.width * 0.7;
  const y = box.y + box.height / 2;

  await page.mouse.move(startX, y);
  await page.mouse.down();
  // Drag left by ~40% of the viewport width — past the 20% paging threshold.
  await page.mouse.move(startX - box.width * 0.4, y, { steps: 8 });
  await page.mouse.up();

  await expect
    .poll(async () => content.evaluate((el) => el.scrollLeft), { timeout: 5_000 })
    .toBeGreaterThan(box.width * 0.5);
});

test("installed Carousel snaps back after a drag below the threshold", async ({ page }) => {
  await page.goto("/carousel");
  const content = track(page);
  await expect(content).toBeVisible();

  const box = await content.boundingBox();
  if (!box) throw new Error("carousel track has no bounding box");
  const startX = box.x + box.width * 0.7;
  const y = box.y + box.height / 2;

  await page.mouse.move(startX, y);
  await page.mouse.down();
  // ~8% of the viewport — well below the 20% threshold.
  await page.mouse.move(startX - box.width * 0.08, y, { steps: 4 });
  await page.mouse.up();

  await expect
    .poll(async () => content.evaluate((el) => el.scrollLeft), { timeout: 5_000 })
    .toBeLessThan(2);
});

test("installed Carousel still pages from its buttons after drag support", async ({ page }) => {
  await page.goto("/carousel");
  const content = track(page);
  await expect(content).toBeVisible();

  await page.getByRole("button", { name: "Next slide" }).click();
  await expect
    .poll(async () => content.evaluate((el) => el.scrollLeft), { timeout: 5_000 })
    .toBeGreaterThan(0);

  await page.getByRole("button", { name: "Previous slide" }).click();
  await expect
    .poll(async () => content.evaluate((el) => el.scrollLeft), { timeout: 5_000 })
    .toBeLessThan(2);
});

test("installed InputOTP masks and unmasks entered digits without losing them", async ({ page }) => {
  await page.goto("/input-otp");
  const slots = page.locator('input[maxlength="1"]');
  await expect(slots.first()).toBeVisible();
  const count = await slots.count();
  expect(count).toBeGreaterThanOrEqual(4);

  // Masked by default (fix-component-defects-and-tone-variants): entry
  // starts obscured, with an explicit reveal toggle rather than plain text.
  for (let i = 0; i < count; i++) {
    await expect(slots.nth(i)).toHaveAttribute("type", "password");
  }

  await slots.first().click();
  await page.keyboard.type("12");
  await expect(slots.nth(0)).toHaveValue("1");
  await expect(slots.nth(1)).toHaveValue("2");
  for (let i = 0; i < count; i++) {
    await expect(slots.nth(i)).toHaveAttribute("type", "password");
  }

  await page.getByRole("button", { name: "Show code" }).click();
  for (let i = 0; i < count; i++) {
    await expect(slots.nth(i)).toHaveAttribute("type", "text");
  }
  // Masking is presentational only: the entered characters survive.
  await expect(slots.nth(0)).toHaveValue("1");
  await expect(slots.nth(1)).toHaveValue("2");

  await page.getByRole("button", { name: "Hide code" }).click();
  for (let i = 0; i < count; i++) {
    await expect(slots.nth(i)).toHaveAttribute("type", "password");
  }
  await expect(slots.nth(0)).toHaveValue("1");
  await expect(slots.nth(1)).toHaveValue("2");
});
