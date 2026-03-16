import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

test.describe("Assembly mates", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
  });

  test("mate button disabled with < 2 components", async ({ page }) => {
    await enterAssemblyMode(page);
    // No components yet — mate button should be disabled
    const mateBtn = page.locator('[data-testid="assembly-mate"]');
    await expect(mateBtn).toBeVisible();
    await expect(mateBtn).toBeDisabled();
  });

  test("mate button enabled with 2+ components", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    const mateBtn = page.locator('[data-testid="assembly-mate"]');
    await expect(mateBtn).toBeEnabled();
  });

  test("clicking Add Mate opens mate panel", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    await page.locator('[data-testid="assembly-mate"]').click();

    // Mate panel should appear
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('[data-testid="mate-comp-a-select"]')).toBeVisible();
    await expect(page.locator('[data-testid="mate-comp-b-select"]')).toBeVisible();
    await expect(page.locator('[data-testid="mate-confirm"]')).toBeVisible();
  });

  test("cancel mate returns to assembly tree", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
    await page.locator('[data-testid="assembly-mate"]').click();
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });

    await page.locator('[data-testid="mate-cancel"]').click();

    await expect(page.locator('[data-testid="assembly-component-count"]')).toBeVisible();
    await expect(page.locator('[data-testid="mate-type-select"]')).not.toBeVisible();
  });

  test("mate type dropdown has all types", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
    await page.locator('[data-testid="assembly-mate"]').click();
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });

    const options = page.locator('[data-testid="mate-type-select"] option');
    const count = await options.count();
    expect(count).toBeGreaterThanOrEqual(8); // All 8 standard mate types
  });
});
