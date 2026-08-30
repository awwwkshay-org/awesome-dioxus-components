import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("installed ColorPicker moves the thumb with arrow keys and via pointer drag", async ({
  page,
}) => {
  await page.goto("/");
  const thumb = page.getByLabel("Color area");
  await expect(thumb).toBeVisible();

  const saturationInput = page.getByLabel("Saturation");

  const initialSaturation = await saturationInput.inputValue();
  // The thumb's onmousedown/ontouchstart intentionally preventDefault to
  // avoid stealing focus during a pointer drag (upstream's own documented
  // behavior) -- reach it the same way a keyboard-only user does: Tab, not click.
  await thumb.focus();
  await page.keyboard.press("ArrowRight");
  const afterArrow = await saturationInput.inputValue();
  expect(Number(afterArrow)).not.toBe(Number(initialSaturation));

  // ArrowRight is a horizontal (delta_x) move: the primitive hands focus to
  // the saturation input afterward (see AreaThumb's onkeydown). Confirm that
  // handoff landed rather than asserting the untested vertical/value-input
  // branch, which is not independently verified in this session.
  await expect(saturationInput).toBeFocused();

  // Pointer drag on the color area surface.
  const area = page.getByRole("group").last();
  const box = await area.boundingBox();
  if (!box) throw new Error("color area has no bounding box");
  await page.mouse.move(box.x + box.width * 0.2, box.y + box.height * 0.2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * 0.8, box.y + box.height * 0.8);
  await page.mouse.up();

  const afterDrag = await saturationInput.inputValue();
  expect(Number(afterDrag)).not.toBe(Number(afterArrow));
});

test("wave5-color-picker fixture has zero critical accessibility violations", async ({ page }) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();
  const critical = results.violations.filter((violation) => violation.impact === "critical");
  expect(critical).toEqual([]);
});
