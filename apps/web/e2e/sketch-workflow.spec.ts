import { test, expect } from "@playwright/test";
import { waitForEditor } from "./helpers";

test.describe("Sketch workflow", () => {
  test("opens plane selector and enters sketch mode", async ({ page }) => {
    await waitForEditor(page);

    // Click Sketch button in ribbon
    await page.locator('[data-testid="ribbon-sketch"]').click();

    // Plane selector should appear
    await expect(page.locator('[data-testid="plane-front"]')).toBeVisible({
      timeout: 5000,
    });

    // Select Front Plane
    await page.locator('[data-testid="plane-front"]').click();

    // Should be in sketch mode — confirm/cancel visible
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });

    // Sketch tab should be active with tool buttons
    await expect(page.locator('[data-testid="tool-line"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-rectangle"]')).toBeVisible();
  });

  test("confirm sketch adds feature to tree", async ({ page }) => {
    await waitForEditor(page);

    // Enter sketch mode
    await page.locator('[data-testid="ribbon-sketch"]').click();
    await page.locator('[data-testid="plane-front"]').click();
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });

    // Confirm the sketch
    await page.locator('[data-testid="sketch-confirm"]').click();

    // Should return to view mode — feature count increased
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "3 features"
    );
  });

  test("cancel sketch does not add feature", async ({ page }) => {
    await waitForEditor(page);

    // Enter sketch mode
    await page.locator('[data-testid="ribbon-sketch"]').click();
    await page.locator('[data-testid="plane-front"]').click();
    await expect(
      page.locator('[data-testid="sketch-cancel"]')
    ).toBeVisible({ timeout: 10000 });

    // Cancel the sketch
    await page.locator('[data-testid="sketch-cancel"]').click();

    // Feature count should still be 2
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "2 features"
    );
  });

  test("sketch mode shows property panel info", async ({ page }) => {
    await waitForEditor(page);

    // Enter sketch mode
    await page.locator('[data-testid="ribbon-sketch"]').click();
    await page.locator('[data-testid="plane-front"]').click();
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });

    // Property panel should show sketch info
    await expect(page.getByText("Front Plane")).toBeVisible();
    await expect(page.getByText("0 points")).toBeVisible();
    await expect(page.getByText("Not Constrained")).toBeVisible();
  });
});
