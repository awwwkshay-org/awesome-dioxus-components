import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed Accordion opens with click, roves focus with ArrowDown, and activates with Enter", async ({
  page,
}) => {
  await page.goto("/");
  const sectionOne = page.getByRole("button", { name: "Section one" });
  const sectionTwo = page.getByRole("button", { name: "Section two" });

  await expect(sectionOne).toHaveAttribute("aria-expanded", "false");
  await sectionOne.click();
  await expect(sectionOne).toHaveAttribute("aria-expanded", "true");

  await page.keyboard.press("ArrowDown");
  await expect(sectionTwo).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(sectionTwo).toHaveAttribute("aria-expanded", "true");
  // allow_multiple_open is false, so opening the second item closes the first.
  await expect(sectionOne).toHaveAttribute("aria-expanded", "false");
});

test("installed RadioGroup roves focus and selects with ArrowDown", async ({ page }) => {
  await page.goto("/");
  const blue = page.getByRole("radio", { name: "Blue" });
  const red = page.getByRole("radio", { name: "Red" });

  await blue.click();
  await expect(blue).toHaveAttribute("aria-checked", "true");
  await expect(red).toHaveAttribute("aria-checked", "false");

  await page.keyboard.press("ArrowDown");
  await expect(red).toBeFocused();
  await expect(red).toHaveAttribute("aria-checked", "true");
  await expect(blue).toHaveAttribute("aria-checked", "false");
});

test("installed Tabs roves focus with ArrowDown without switching, then activates with Enter", async ({
  page,
}) => {
  await page.goto("/");
  const tab1 = page.getByRole("tab", { name: "Tab 1" });
  const tab2 = page.getByRole("tab", { name: "Tab 2" });

  await tab1.click();
  await expect(tab1).toHaveAttribute("aria-selected", "true");
  await expect(page.getByText("Tab 1 content")).toBeVisible();

  await page.keyboard.press("ArrowDown");
  await expect(tab2).toBeFocused();
  // Manual-activation pattern: roving focus alone must not switch the panel.
  await expect(tab1).toHaveAttribute("aria-selected", "true");
  await expect(page.getByText("Tab 1 content")).toBeVisible();

  await page.keyboard.press("Enter");
  await expect(tab2).toHaveAttribute("aria-selected", "true");
  await expect(page.getByText("Tab 2 content")).toBeVisible();
});

test("installed ToggleGroup roves focus with ArrowRight and toggles pressed state radio-style", async ({
  page,
}) => {
  await page.goto("/");
  const bold = page.getByRole("button", { name: "Bold" });
  const italic = page.getByRole("button", { name: "Italic" });

  await bold.click();
  await expect(bold).toHaveAttribute("aria-pressed", "true");

  await page.keyboard.press("ArrowRight");
  await expect(italic).toBeFocused();
  await italic.click();
  await expect(italic).toHaveAttribute("aria-pressed", "true");
  // allow_multiple_pressed defaults to false, so pressing Italic unpresses Bold.
  await expect(bold).toHaveAttribute("aria-pressed", "false");
});

test("wave2-roving-focus fixture has zero critical accessibility violations", async ({ page }) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();
  const critical = results.violations.filter((violation) => violation.impact === "critical");
  expect(critical).toEqual([]);
});
