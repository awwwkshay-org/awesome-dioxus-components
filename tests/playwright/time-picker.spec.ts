import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import type { Locator, Page } from "@playwright/test";

// Runs against `tests/installation/time-picker-consumer`, which binds its
// digital (columns) and analog (dial) pickers to one shared signal, so the
// three ways of setting a time can be asserted to converge on the same value.
// `#time-value` is the fixture's readout of that shared signal.

const readout = "#time-value";

/** Opens the disclosure inside the given section and returns its popup. */
async function openPicker(page: Page, section: string): Promise<Locator> {
  // `PopoverTrigger` renders a plain `button` with no `aria-expanded`, so the
  // trigger is addressed by its accessible name instead.
  await page.locator(section).getByRole("button", { name: "Toggle time picker" }).click();
  const popup = page.locator('[role="dialog"]');
  await expect(popup).toBeVisible();
  return popup;
}

/**
 * Hour and minute columns both run through overlapping labels ("07" is an
 * hour *and* a minute), so a bare name lookup is ambiguous. `div.w-14` is the
 * column wrapper `TimePickerColumns` renders, in hour-then-minute order.
 */
function column(popup: Locator, index: number): Locator {
  return popup.locator("div.w-14").nth(index);
}

test("segmented input in the trigger accepts a typed time", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(readout)).toHaveText("none");

  // The segments are the always-available text path: one spinbutton each for
  // hour and minute, typed into directly rather than through any popup.
  const segments = page.locator('#digital [role="spinbutton"]');
  await expect(segments).toHaveCount(2);

  await segments.nth(0).focus();
  await page.keyboard.type("09");
  await segments.nth(1).focus();
  await page.keyboard.type("30");

  await expect(page.locator(readout)).toHaveText("09:30");
});

test("columns view selects a time with the keyboard alone", async ({ page }) => {
  await page.goto("/");
  const popup = await openPicker(page, "#digital");

  // The columns are plain native buttons, so they are reachable and
  // activatable with the keyboard -- this is the documented accessible path,
  // the dial being pointer-only enhancement.
  const hour07 = column(popup, 0).getByRole("button", { name: "07", exact: true });
  await hour07.focus();
  await expect(hour07).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.locator(readout)).toHaveText(/^07:/);

  const minute45 = column(popup, 1).getByRole("button", { name: "45", exact: true });
  await minute45.focus();
  await expect(minute45).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.locator(readout)).toHaveText("07:45");
});

// The clock dial's pointer drag is deliberately NOT tested here. The dial
// relies on `pointer-events-none` to keep its hour labels and hand from
// intercepting the pointer, and this fixture cannot compile that utility:
// `tests/installation/.gitignore` ignores `*/src/components/`, and Tailwind v4
// skips gitignored paths during source detection, so the fixture's stylesheet
// contains no utilities from the installed components at all. Without it the
// pointer lands on a label rather than the dial and the angle is meaningless.
// Dial coverage therefore lives in `playground-time-picker.spec.ts`, which
// runs against the playground, where the stylesheet is complete.

test("all three input paths converge on one value", async ({ page }) => {
  await page.goto("/");

  // 1. Type into the segments.
  const segments = page.locator('#digital [role="spinbutton"]');
  await segments.nth(0).focus();
  await page.keyboard.type("04");
  await segments.nth(1).focus();
  await page.keyboard.type("20");
  await expect(page.locator(readout)).toHaveText("04:20");

  // 2. Both pickers are bound to that one signal, so the analog picker's own
  // segments show what the digital one was typed into.
  const analogSegments = page.locator('#analog [role="spinbutton"]');
  await expect(analogSegments.nth(0)).toHaveAttribute("aria-valuenow", "4");
  await expect(analogSegments.nth(1)).toHaveAttribute("aria-valuenow", "20");

  // 3. Setting it from the columns updates that same shared value.
  const popup = await openPicker(page, "#digital");
  await column(popup, 0).getByRole("button", { name: "09", exact: true }).click();
  await expect(page.locator(readout)).toHaveText("09:20");
});

test("DateTimePicker combines a date and a time into one value", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("#datetime-value")).toHaveText("none");

  await page.locator("#datetime").getByRole("button", { name: "Pick date & time" }).click();
  const popup = page.locator('[role="dialog"]');
  await expect(popup).toBeVisible();

  // Time first: on its own it is not a complete value, because no date is set
  // yet. Picking the date is left until last on purpose -- it closes the
  // popover (`DatePickerCalendar` dismisses on select), so doing it first
  // would mean reopening.
  await column(popup, 0).getByRole("button", { name: "06", exact: true }).click();
  await column(popup, 1).getByRole("button", { name: "15", exact: true }).click();
  await expect(page.locator("#datetime-value")).toHaveText("none");

  // Completing the date commits both halves together: the buffered time above
  // must survive, which is the regression this composition previously had.
  await popup.getByRole("grid").locator('button[tabindex="0"]').click();
  await expect(page.locator("#datetime-value")).toHaveText(/T06:15$/);
});

test("segmented input and columns have no critical accessibility violations", async ({ page }) => {
  await page.goto("/");
  await openPicker(page, "#digital");

  // Scoped to the segmented input and the columns view deliberately: the
  // clock dial has no WAI-ARIA analog-clock pattern and is `aria-hidden`
  // pointer-only enhancement, never the only way to set a value.
  const results = await new AxeBuilder({ page })
    .include("#digital")
    .include('[role="dialog"]')
    .analyze();
  const critical = results.violations.filter((v) => v.impact === "critical");
  expect(critical).toEqual([]);
});
