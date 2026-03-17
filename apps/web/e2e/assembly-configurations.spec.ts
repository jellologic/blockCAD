import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

test.describe("Assembly Configurations (C2)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
  });

  test("can create a configuration", async ({ page }) => {
    // Click add configuration button
    await page.locator('[data-testid="config-add-btn"]').click();
    await page.locator('[data-testid="config-name-input"]').fill("Open Position");
    await page.locator('[data-testid="config-confirm"]').click();

    // Configuration should appear in dropdown
    const select = page.locator('[data-testid="config-select"]');
    await expect(select).toBeVisible();
    await expect(select).toContainText("Open Position");
  });
});
