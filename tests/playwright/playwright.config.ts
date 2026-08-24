import { defineConfig } from "@playwright/test";

const dialogAppUrl = process.env.ADICO_DIALOG_APP_URL ?? "http://127.0.0.1:5174";

export default defineConfig({
  testDir: ".",
  testMatch: "*.spec.ts",
  timeout: 30_000,
  use: {
    baseURL: dialogAppUrl,
    browserName: "chromium",
    trace: "retain-on-failure"
  }
});
