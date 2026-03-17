import { test, expect } from "@playwright/test";

async function waitForKernel(page: import("@playwright/test").Page) {
  await page.goto("/editor");
  await page.waitForFunction(() => {
    const store = (window as any).__editorStore;
    return store && store.getState().kernel && !store.getState().isLoading;
  }, { timeout: 20000 });
}

test.describe("Sample models dropdown", () => {
  test("dropdown shows 8 sample models", async ({ page }) => {
    await waitForKernel(page);

    // Click the Samples dropdown button
    const dropdown = page.locator('[data-testid="sample-models-dropdown"]');
    await expect(dropdown).toBeVisible();
    await dropdown.locator("button").first().click();

    // Verify all 8 sample items are shown
    const items = page.locator('[data-testid^="sample-"]');
    await expect(items).toHaveCount(8);
  });

  test("load Simple Box — 2 features + mesh with vertices", async ({ page }) => {
    await waitForKernel(page);

    // Open dropdown and click Simple Box
    const dropdown = page.locator('[data-testid="sample-models-dropdown"]');
    await dropdown.locator("button").first().click();
    await page.locator('[data-testid="sample-simple-box"]').click();

    // Verify 2 features in tree (sketch + extrude)
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("2 feature", { timeout: 10000 });

    // Verify mesh has vertices
    await expect(page.locator('[data-testid="vertex-count"]')).toContainText("Verts:", { timeout: 10000 });
  });

  test("load Filleted Box — 3 features in tree", async ({ page }) => {
    await waitForKernel(page);

    // Open dropdown and click Filleted Box
    const dropdown = page.locator('[data-testid="sample-models-dropdown"]');
    await dropdown.locator("button").first().click();
    await page.locator('[data-testid="sample-filleted-box"]').click();

    // Verify 3 features (sketch + extrude + fillet)
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("3 feature", { timeout: 10000 });

    // Verify mesh exists
    await expect(page.locator('[data-testid="vertex-count"]')).toContainText("Verts:", { timeout: 10000 });
  });

  test("load Hole Plate — 4 features in tree", async ({ page }) => {
    await waitForKernel(page);

    const dropdown = page.locator('[data-testid="sample-models-dropdown"]');
    await dropdown.locator("button").first().click();
    await page.locator('[data-testid="sample-hole-plate"]').click();

    // Verify 4 features (plate sketch + extrude + hole sketch + cut extrude)
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("4 feature", { timeout: 10000 });
    await expect(page.locator('[data-testid="vertex-count"]')).toContainText("Verts:", { timeout: 10000 });
  });

  test("loading a sample replaces previous model", async ({ page }) => {
    await waitForKernel(page);

    // Load Filleted Box (3 features)
    const dropdown = page.locator('[data-testid="sample-models-dropdown"]');
    await dropdown.locator("button").first().click();
    await page.locator('[data-testid="sample-filleted-box"]').click();
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("3 feature", { timeout: 10000 });

    // Load Simple Box (2 features) — should replace, not append
    await dropdown.locator("button").first().click();
    await page.locator('[data-testid="sample-simple-box"]').click();
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("2 feature", { timeout: 10000 });
  });
});
