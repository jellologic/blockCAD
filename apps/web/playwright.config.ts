import { defineConfig } from "@playwright/test";

const TEST_PORT = 5099;

export default defineConfig({
  testDir: "./e2e",
  timeout: 30000,
  retries: 0,
  workers: 1,
  use: {
    baseURL: `http://localhost:${TEST_PORT}`,
    headless: false,
    screenshot: "only-on-failure",
    trace: "on-first-retry",
    viewport: { width: 1280, height: 720 },
  },
  webServer: {
    command: `vite dev --port ${TEST_PORT}`,
    port: TEST_PORT,
    timeout: 30000,
    reuseExistingServer: true,
  },
});
