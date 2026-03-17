import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

test.describe("Assembly face picking", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
  });

  test("Select Face A button activates picking mode", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    // Open mate panel
    await page.locator('[data-testid="assembly-mate"]').click();
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });

    // Click "Select Face A" button
    const selectFaceA = page.locator('[data-testid="mate-select-face-a"]');
    await expect(selectFaceA).toBeVisible();
    await selectFaceA.click();

    // Picking hint should appear
    await expect(page.locator('[data-testid="mate-picking-hint"]')).toBeVisible();
    await expect(page.locator('[data-testid="mate-picking-hint"]')).toContainText("Face A");

    // Verify store has pickingMode set
    const pickingMode = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      return store.getState().pickingMode;
    });
    expect(pickingMode).toBe("face_a");
  });

  test("Select Face B button activates picking mode", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    await page.locator('[data-testid="assembly-mate"]').click();
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });

    const selectFaceB = page.locator('[data-testid="mate-select-face-b"]');
    await expect(selectFaceB).toBeVisible();
    await selectFaceB.click();

    await expect(page.locator('[data-testid="mate-picking-hint"]')).toBeVisible();
    await expect(page.locator('[data-testid="mate-picking-hint"]')).toContainText("Face B");

    const pickingMode = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      return store.getState().pickingMode;
    });
    expect(pickingMode).toBe("face_b");
  });

  test("clicking Select Face A again deactivates picking mode", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    await page.locator('[data-testid="assembly-mate"]').click();
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });

    const selectFaceA = page.locator('[data-testid="mate-select-face-a"]');
    await selectFaceA.click();
    await expect(page.locator('[data-testid="mate-picking-hint"]')).toBeVisible();

    // Click again to toggle off
    await selectFaceA.click();
    await expect(page.locator('[data-testid="mate-picking-hint"]')).not.toBeVisible();
  });

  test("onFacePicked populates face fields via store", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    await page.locator('[data-testid="assembly-mate"]').click();
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });

    // Activate picking mode for Face A
    await page.locator('[data-testid="mate-select-face-a"]').click();
    await expect(page.locator('[data-testid="mate-picking-hint"]')).toBeVisible();

    // Simulate a face pick via the store
    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      store.getState().onFacePicked("comp-0", 3);
    });

    // Picking hint should disappear (mode cleared)
    await expect(page.locator('[data-testid="mate-picking-hint"]')).not.toBeVisible();

    // The picked face info should appear
    await expect(page.locator('[data-testid="mate-face-a-picked"]')).toBeVisible();
    await expect(page.locator('[data-testid="mate-face-a-picked"]')).toContainText("comp-0 face 3");
  });

  test("cancel clears picking mode", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    await page.locator('[data-testid="assembly-mate"]').click();
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });

    await page.locator('[data-testid="mate-select-face-a"]').click();
    await expect(page.locator('[data-testid="mate-picking-hint"]')).toBeVisible();

    // Cancel the mate operation
    await page.locator('[data-testid="mate-cancel"]').click();

    // Store picking mode should be cleared
    const pickingMode = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      return store.getState().pickingMode;
    });
    expect(pickingMode).toBeNull();
  });
});
