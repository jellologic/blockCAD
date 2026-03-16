import { test, expect } from "@playwright/test";
import { waitForEditor } from "./helpers";

test.describe("Editor basic functionality", () => {
  test("loads editor with default features", async ({ page }) => {
    await waitForEditor(page);

    // Ribbon tabs exist
    await expect(page.locator('[data-testid="tab-features"]')).toBeVisible();
    await expect(page.locator('[data-testid="tab-sketch"]')).toBeVisible();
    await expect(page.locator('[data-testid="tab-view"]')).toBeVisible();

    // Feature tree shows initial features
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "feature"
    );

    // Viewport exists (canvas renders)
    await expect(page.locator("canvas")).toBeVisible();
  });

  test("ribbon tabs switch content", async ({ page }) => {
    await waitForEditor(page);

    // Default: Features tab active, Sketch button visible
    await expect(page.locator('[data-testid="ribbon-sketch"]')).toBeVisible();

    // Switch to Sketch tab
    await page.click('[data-testid="tab-sketch"]');
    // Sketch tab should show tool buttons (even if disabled)
    await expect(page.locator('[data-testid="tool-line"]')).toBeVisible();

    // Switch to View tab
    await page.click('[data-testid="tab-view"]');
    // Tool buttons should not be visible on View tab
    await expect(
      page.locator('[data-testid="tool-line"]')
    ).not.toBeVisible();

    // Switch back to Features
    await page.click('[data-testid="tab-features"]');
    await expect(page.locator('[data-testid="ribbon-sketch"]')).toBeVisible();
  });

  test("no site header on editor page", async ({ page }) => {
    await waitForEditor(page);
    // The nav links from the site header should NOT be visible
    await expect(page.locator('a[href="/dashboard"]')).not.toBeVisible();
  });
});
