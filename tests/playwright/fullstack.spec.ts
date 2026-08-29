import { expect, test } from "@playwright/test";

// Exercises `examples/basic-ssr`, which installs Button, Dialog, and Select
// through the real `adico` CLI. Run against a `dx serve --platform web`
// instance of that example (see README) so both server-rendered HTML and
// client hydration are covered by one real request/response cycle.

test("Button, Dialog, and Select render over SSR and hydrate without console errors", async ({
  page
}) => {
  const consoleIssues: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error" || message.type() === "warning") {
      consoleIssues.push(`[${message.type()}] ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => consoleIssues.push(`[pageerror] ${error.message}`));

  const response = await page.goto("/");
  const serverRenderedHtml = await response?.text();
  expect(serverRenderedHtml).toContain("SSR/hydration smoke check");
  expect(serverRenderedHtml).toContain("Open dialog");
  expect(serverRenderedHtml).toContain("Choose a fruit");

  // Interactions only succeed once the client wasm bundle has hydrated and
  // attached its event listeners to the server-rendered markup.
  await expect(page.getByRole("button", { name: "SSR/hydration smoke check" })).toBeVisible();

  await page.getByRole("button", { name: "Open dialog" }).click();
  const dialog = page.getByRole("dialog", { name: "Hydration check" });
  await expect(dialog).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();

  await page.getByLabel("Choose a fruit").click();
  await expect(page.getByText("Banana", { exact: true })).toBeVisible();
  await page.keyboard.press("Escape");

  expect(consoleIssues).toEqual([]);
});
