import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

test.describe("Assembly components", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
  });

  test("insert button opens component insert panel", async ({ page }) => {
    await enterAssemblyMode(page);
    await page.locator('[data-testid="assembly-insert"]').click();

    // Insert panel should appear with part dropdown and name input
    await expect(page.locator('[data-testid="insert-name"]')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('[data-testid="insert-confirm"]')).toBeVisible();
    await expect(page.locator('[data-testid="insert-cancel"]')).toBeVisible();
  });

  test("cancel insert returns to assembly tree", async ({ page }) => {
    await enterAssemblyMode(page);
    await page.locator('[data-testid="assembly-insert"]').click();
    await expect(page.locator('[data-testid="insert-name"]')).toBeVisible({ timeout: 5000 });

    await page.locator('[data-testid="insert-cancel"]').click();

    // Should return to assembly tree
    await expect(page.locator('[data-testid="assembly-component-count"]')).toBeVisible();
    await expect(page.locator('[data-testid="insert-name"]')).not.toBeVisible();
  });

  test("programmatic component insertion shows in tree", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
    // Wait for React to pick up store changes
    await page.waitForTimeout(500);

    // Tree should show 2 components
    await expect(page.locator('[data-testid="assembly-component-count"]')).toContainText("2 active", { timeout: 5000 });
  });

  test("three components shows correct count", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 3);

    await expect(page.locator('[data-testid="assembly-component-count"]')).toContainText("3 active");
  });

  test("parts section shows added part", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 1);

    // Parts section should show the box part
    await expect(page.locator("text=Box Part")).toBeVisible();
  });
});
