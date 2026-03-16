import { test, expect } from "@playwright/test";
import { enterAssemblyMode } from "./helpers";

test.describe("Assembly mode", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000); // Wait for kernel init
  });

  test("assembly tab is visible in ribbon", async ({ page }) => {
    await expect(page.locator('[data-testid="tab-assembly"]')).toBeVisible();
  });

  test("entering assembly mode shows tree and ribbon buttons", async ({ page }) => {
    await enterAssemblyMode(page);

    // Assembly tree panel should appear
    await expect(page.locator('[data-testid="assembly-component-count"]')).toBeVisible();

    // Assembly ribbon should show Exit, Insert, BOM, Explode, Export
    await expect(page.locator('[data-testid="assembly-exit"]')).toBeVisible();
    await expect(page.locator('[data-testid="assembly-insert"]')).toBeVisible();
    await expect(page.locator('[data-testid="assembly-bom"]')).toBeVisible();
    await expect(page.locator('[data-testid="assembly-explode"]')).toBeVisible();
    await expect(page.locator('[data-testid="assembly-export"]')).toBeVisible();
  });

  test("exiting assembly mode returns to features", async ({ page }) => {
    await enterAssemblyMode(page);
    await expect(page.locator('[data-testid="assembly-exit"]')).toBeVisible();

    await page.locator('[data-testid="assembly-exit"]').click();

    // Assembly tree should be gone
    await expect(page.locator('[data-testid="assembly-component-count"]')).not.toBeVisible();
  });
});
