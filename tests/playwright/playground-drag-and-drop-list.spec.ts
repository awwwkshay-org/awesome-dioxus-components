import { expect, test } from "@playwright/test";

// Regresses a hang that only reproduced on the playground's own composition
// of Drag And Drop List: the page called `use_drag_and_drop_list_items()`
// from an ancestor scope of `DragAndDropList`'s context provider, which
// panicked during render. `tests/installation/wave5-drag-and-drop-list-consumer`
// uses the default-children path (a descendant scope), so it could not catch
// this — this spec targets the playground route directly instead.
test("playground Drag And Drop List page renders without hanging and reorders via keyboard", async ({
  page,
}) => {
  await page.goto("/drag-and-drop-list");

  const list = page.getByRole("list", { name: "Reorderable items" });
  const items = list.getByRole("listitem");
  await expect(items).toHaveCount(4);
  await expect(items.filter({ hasText: "Alpha" })).toBeVisible();

  const alpha = items.filter({ hasText: "Alpha" });
  await alpha.click();
  await expect(alpha).toHaveAttribute("aria-grabbed", "false");

  await page.keyboard.press("Enter");
  await expect(alpha).toHaveAttribute("aria-grabbed", "true");
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Enter");
  await expect(alpha).toHaveAttribute("aria-grabbed", "false");

  const reordered = await items.allTextContents();
  expect(reordered[0]).not.toBe("Alpha");
  expect(reordered).toContain("Alpha");
});
