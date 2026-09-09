import { defineConfig } from "@playwright/test";

const fixtureAppUrl =
  process.env.ADICO_PLAYWRIGHT_BASE_URL ??
  process.env.ADICO_DIALOG_APP_URL ??
  "http://127.0.0.1:5174";

const baseUse = {
  baseURL: fixtureAppUrl,
  browserName: "chromium" as const,
  trace: "retain-on-failure" as const
};

export default defineConfig({
  testDir: ".",
  timeout: 30_000,
  projects: [
    {
      // The pre-existing suite. Pinning viewport explicitly (rather than
      // relying on Playwright's own default) is what turns "desktop
      // unchanged" into an assertion for the mobile-first sweep's
      // desktop-invariance project below, not just an implicit default.
      name: "desktop",
      testIgnore: /responsive.*\.spec\.ts/,
      use: { ...baseUse, viewport: { width: 1280, height: 800 } }
    },
    {
      // `make-registry-components-mobile-first`: asserts no registry
      // component overflows a 375px viewport. See responsive.spec.ts.
      name: "mobile",
      testMatch: /(^|\/)responsive\.spec\.ts$/,
      use: { ...baseUse, viewport: { width: 375, height: 812 } }
    },
    {
      // Companion to `mobile`: asserts today's desktop geometry is
      // unchanged by the mobile-first sweep. See responsive-desktop.spec.ts.
      name: "desktop-invariance",
      testMatch: /(^|\/)responsive-desktop\.spec\.ts$/,
      use: { ...baseUse, viewport: { width: 1280, height: 800 } }
    }
  ]
});
