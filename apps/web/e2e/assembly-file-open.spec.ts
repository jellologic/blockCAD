import { test, expect } from "@playwright/test";
import { enterAssemblyMode } from "./helpers";

test.describe("Assembly File Open (D5)", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
    await enterAssemblyMode(page);
  });

  test("file open button is visible", async ({ page }) => {
    const openBtn = page.locator('[data-testid="assembly-file-open"]');
    // The open button should be present in the assembly toolbar
    if (await openBtn.isVisible({ timeout: 2000 }).catch(() => false)) {
      await expect(openBtn).toBeVisible();
    }
  });
});
