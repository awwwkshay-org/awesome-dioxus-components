import { expect, test } from "@playwright/test";

// Runs against the playground app (`dx serve` from `apps/playground`, then
// `ADICO_PLAYWRIGHT_BASE_URL=<url> npm run test:playground-resizable-split`).
//
// Regression coverage for the `ResizablePanel` seeding race fixed by
// `openspec/changes/fix-resizable-panel-seeding/`: two sibling panels raced
// to seed a shared size table, and whichever seeded first clobbered the
// other's constraints too, collapsing the preview/controls split to an
// equal, permanently-stuck 30/30 instead of the coded 70/30. The load-bearing
// assertion is the drag test below, not the fresh-load ratio: an entirely
// unseeded table can still *paint* a correct-looking 70/30 via a
// `unwrap_or(default_size)` render fallback while dragging stays completely
// inert, so only actually dragging the handle to its bounds proves the fix.

// `Demo`'s preview/controls split is the only `ResizablePanelGroup` on any
// page whose direction is `Vertical` -- this component's `ResizableHandle`
// inverts `aria-orientation` relative to its own `ResizableDirection` (see
// `registry/ui/resizable.rs`), so a `Vertical` group's handle reports
// `aria-orientation="horizontal"`. `Layout`'s nav/content split is
// `Horizontal` (handle reports `aria-orientation="vertical"`), so this
// selector never matches it.
const splitHandle = (page: import("@playwright/test").Page) =>
  page.locator('[role="separator"][aria-orientation="horizontal"]');

async function panelHeights(page: import("@playwright/test").Page) {
  const handle = splitHandle(page);
  return handle.evaluate((el) => {
    const preview = el.previousElementSibling as HTMLElement | null;
    const controls = el.nextElementSibling as HTMLElement | null;
    if (!preview || !controls) {
      throw new Error("resizable handle is missing a sibling panel");
    }
    return {
      preview: preview.getBoundingClientRect().height,
      controls: controls.getBoundingClientRect().height,
    };
  });
}

for (const path of ["/alert-dialog", "/badge"]) {
  test(`${path} renders the preview/controls split at its coded 70/30 default, not equal`, async ({
    page,
  }) => {
    await page.goto(path);
    await expect(splitHandle(page)).toBeVisible();

    const { preview, controls } = await panelHeights(page);
    const total = preview + controls;
    // 70/30 of the group, not 50/50 (the clobbered-seeding regression) --
    // allow generous tolerance for the handle's own height and any
    // sub-pixel layout rounding.
    expect(preview / total).toBeGreaterThan(0.6);
    expect(controls / total).toBeLessThan(0.4);
  });
}

test("the preview/controls handle actually drags to its new bounds (load-bearing: proves seeding, not just the initial paint)", async ({
  page,
}) => {
  await page.goto("/alert-dialog");
  const handle = splitHandle(page);
  await expect(handle).toBeVisible();

  // Drag far past the range in each direction -- clamp_delta caps the
  // result at the panel's own bound regardless of how far the pointer
  // travels, so a large delta reliably reaches the extreme without needing
  // exact pixel math. The handle's own position shifts after each drag, so
  // its bounding box is re-read before every drag rather than assumed.
  async function dragBy(deltaY: number) {
    const currentBox = await handle.boundingBox();
    if (!currentBox) throw new Error("resizable handle has no bounding box");
    const x = currentBox.x + currentBox.width / 2;
    const y = currentBox.y + currentBox.height / 2;
    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x, y + deltaY, { steps: 10 });
    await page.mouse.up();
  }

  // Dragging the handle DOWN (positive Y) grows the preview panel (index 0,
  // "prev" of the handle) and shrinks the controls panel (index 1, "next")
  // -- see `resizable.rs`'s pointermove handler: `wanted` is measured from
  // pointer position and added to "prev"'s size, subtracted from "next"'s.
  await dragBy(2000);
  {
    const { preview, controls } = await panelHeights(page);
    const total = preview + controls;
    // Controls' coded min is 20%, preview's coded max is 80%.
    expect(controls / total).toBeGreaterThan(0.15);
    expect(controls / total).toBeLessThan(0.25);
    expect(preview / total).toBeGreaterThan(0.75);
    expect(preview / total).toBeLessThan(0.85);
  }

  await dragBy(-4000);
  {
    const { preview, controls } = await panelHeights(page);
    const total = preview + controls;
    // Controls' coded max is 40%, preview's coded min is 60%.
    expect(controls / total).toBeGreaterThan(0.35);
    expect(controls / total).toBeLessThan(0.45);
    expect(preview / total).toBeGreaterThan(0.55);
    expect(preview / total).toBeLessThan(0.65);
  }
});
