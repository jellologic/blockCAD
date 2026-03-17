import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

test.describe("Assembly patterns", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
  });

  test("pattern button opens pattern panel", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 1);
    await page.waitForTimeout(500);

    await page.locator('[data-testid="assembly-pattern"]').click();

    await expect(page.locator('[data-testid="pattern-type-select"]')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('[data-testid="pattern-confirm"]')).toBeVisible();
    await expect(page.locator('[data-testid="pattern-cancel"]')).toBeVisible();
  });

  test("cancel pattern returns to assembly tree", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 1);
    await page.waitForTimeout(500);

    await page.locator('[data-testid="assembly-pattern"]').click();
    await expect(page.locator('[data-testid="pattern-type-select"]')).toBeVisible({ timeout: 5000 });

    await page.locator('[data-testid="pattern-cancel"]').click();

    await expect(page.locator('[data-testid="assembly-component-count"]')).toBeVisible();
    await expect(page.locator('[data-testid="pattern-type-select"]')).not.toBeVisible();
  });

  test("create linear pattern increases component count", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 1);
    await page.waitForTimeout(500);

    // Verify initial count
    await expect(page.locator('[data-testid="assembly-component-count"]')).toContainText("1 active");

    // Create linear pattern via store
    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const state = store.getState();
      const comp = state.components[0];
      state.createLinearPattern([comp.id], [1, 0, 0], 20, 3);
    });
    await page.waitForTimeout(500);

    // Should now have 3 components (1 original + 2 from pattern)
    await expect(page.locator('[data-testid="assembly-component-count"]')).toContainText("3 active");
  });

  test("create circular pattern increases component count", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 1);
    await page.waitForTimeout(500);

    await expect(page.locator('[data-testid="assembly-component-count"]')).toContainText("1 active");

    // Create circular pattern via store
    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const state = store.getState();
      const comp = state.components[0];
      state.createCircularPattern([comp.id], [0, 0, 0], [0, 0, 1], 90, 4);
    });
    await page.waitForTimeout(500);

    // Should now have 4 components (1 original + 3 from pattern)
    await expect(page.locator('[data-testid="assembly-component-count"]')).toContainText("4 active");
  });

  test("pattern type selector switches between linear and circular", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 1);
    await page.waitForTimeout(500);

    await page.locator('[data-testid="assembly-pattern"]').click();
    await expect(page.locator('[data-testid="pattern-type-select"]')).toBeVisible({ timeout: 5000 });

    // Default is linear - spacing field should be visible
    await expect(page.locator('[data-testid="pattern-spacing"]')).toBeVisible();
    await expect(page.locator('[data-testid="pattern-angle-spacing"]')).not.toBeVisible();

    // Switch to circular
    await page.locator('[data-testid="pattern-type-select"]').selectOption("circular");

    // Now angle spacing should be visible, not linear spacing
    await expect(page.locator('[data-testid="pattern-angle-spacing"]')).toBeVisible();
    await expect(page.locator('[data-testid="pattern-spacing"]')).not.toBeVisible();
  });
});
