import { expect, test } from "@playwright/test";
import type { Locator, Page } from "@playwright/test";

// Runs against the playground (`cd apps/playground && dx serve`), not the
// installation fixture, e.g.
//   ADICO_PLAYWRIGHT_BASE_URL=http://localhost:3000 npm run test:playground-time-picker
//
// The clock dial has to be exercised here rather than in
// `tests/installation/time-picker-consumer`: the dial depends on
// `pointer-events-none` so its hour labels and hand do not intercept the
// pointer, and that fixture cannot compile the utility at all --
// `tests/installation/.gitignore` ignores `*/src/components/`, and Tailwind v4
// skips gitignored paths when detecting sources, so none of the installed
// components' classes reach its stylesheet. The playground commits its
// installed components, so its stylesheet is complete.

/** Opens the TimePicker popup on `/time-picker` and returns the dial. */
async function openDial(page: Page): Promise<Locator> {
  await page.goto("/time-picker");

  // Switch the demo's "View" control to the analog dial.
  await page.getByLabel("View").selectOption({ label: "Analog" });

  await page.getByRole("button", { name: "Toggle time picker" }).click();
  const popup = page.locator('[role="dialog"]');
  await expect(popup).toBeVisible();
  const dial = popup.locator('[role="presentation"][aria-hidden="true"]');
  await expect(dial).toBeVisible();

  // Wait for the dial's box to stop moving before any caller measures it.
  // The popover runs a `zoom-in-95` entrance animation *and* is positioned by
  // `Positioner` after mount, so a box read immediately after it becomes
  // visible is both smaller and in the wrong place -- which skews every angle
  // computed from it by roughly one snap step.
  let previous = "";
  await expect
    .poll(
      async () => {
        const box = await dial.boundingBox();
        const current = JSON.stringify(box);
        const settled = current === previous;
        previous = current;
        return settled;
      },
      { intervals: [100, 100, 100, 100, 100] },
    )
    .toBe(true);

  return dial;
}


/**
 * Moves the pointer to `deg` on the dial, re-measuring the dial first.
 *
 * Re-measuring every step is required, not defensive: committing a value
 * changes the trigger field's text (`HH` becomes a number), which changes its
 * width, which moves the popup's anchor -- so `Positioner` shifts the popup
 * mid-drag. Measured live at ~20px horizontally, enough to throw the computed
 * angle out by a whole snap step.
 */
async function pointAt(
  page: Page,
  dial: Locator,
  deg: number,
): Promise<void> {
  const box = await dial.boundingBox();
  if (!box) throw new Error("dial has no bounding box");
  const cx = box.x + box.width / 2;
  const cy = box.y + box.height / 2;
  const radius = box.width * 0.35;
  const a = (deg * Math.PI) / 180;
  await page.mouse.move(cx + radius * Math.sin(a), cy - radius * Math.cos(a));
}

function handAngle(style: string | null): number | null {
  const match = /rotate\(([-\d.]+)deg\)/.exec(style ?? "");
  return match ? Number(match[1]) : null;
}

test("clock dial sets the hour by pointer drag", async ({ page }) => {
  const dial = await openDial(page);

  // Drag from 12 o'clock round to 6 o'clock. On the 24-hour face that is 12.
  await pointAt(page, dial, 0);
  await page.mouse.down();
  for (const deg of [45, 90, 135, 180]) {
    await pointAt(page, dial, deg);
  }
  await page.mouse.up();

  const segments = page.locator('[role="spinbutton"]');
  await expect(segments.nth(0)).toHaveAttribute("aria-valuenow", "12");
});

test("the hand tracks the pointer continuously rather than in snapped steps", async ({
  page,
}) => {
  const dial = await openDial(page);
  const hand = dial.locator('span[style*="rotate"]');

  await pointAt(page, dial, 0);
  await page.mouse.down();

  // Sample the hand's rotation across a sweep smaller than one snap step
  // (15 degrees per hour on the 24-hour face). If the hand were rendered from
  // the snapped value these would all collapse onto the same angle.
  const angles: number[] = [];
  for (const deg of [4, 8, 12]) {
    await pointAt(page, dial, deg);
    angles.push(handAngle(await hand.getAttribute("style")) ?? NaN);
  }
  await page.mouse.up();

  expect(new Set(angles).size).toBe(angles.length);
  // ...and strictly increasing, i.e. following the pointer round the face.
  expect(angles[0]).toBeLessThan(angles[1]);
  expect(angles[1]).toBeLessThan(angles[2]);
});

test("the hand eases onto the snapped value once the drag ends", async ({ page }) => {
  const dial = await openDial(page);
  const hand = dial.locator('span[style*="rotate"]');

  // Release a little past the 3 o'clock mark (90deg == hour 6 of 24): the
  // hand should settle exactly on it rather than staying where the pointer
  // was left.
  await pointAt(page, dial, 94);
  await page.mouse.down();
  await page.mouse.up();

  // 180deg is the hand's "points at 12 o'clock" base; hour 6 of 24 adds 90.
  await expect.poll(async () => handAngle(await hand.getAttribute("style"))).toBe(270);
});
