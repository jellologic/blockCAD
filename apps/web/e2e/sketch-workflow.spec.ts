import { test, expect } from "@playwright/test";
import { waitForEditor, enterSketchMode } from "./helpers";

test.describe("Sketch workflow", () => {
  test("enters sketch mode and shows tools", async ({ page }) => {
    await waitForEditor(page);

    // Enter sketch mode programmatically (plane selection requires 3D click)
    await enterSketchMode(page, "front");

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
    await enterSketchMode(page, "front");

    // Confirm the sketch
    await page.locator('[data-testid="sketch-confirm"]').click();

    // Should return to view mode — feature count increased by 1
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "1 feature"
    );
  });

  test("cancel sketch does not add feature", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    // Cancel the sketch
    await page.locator('[data-testid="sketch-cancel"]').click();

    // Feature count should still be 0 (no sketch added)
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "0 features"
    );
  });

  test("sketch mode shows property panel info", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    // Property panel should show sketch info
    await expect(page.getByText("Front Plane")).toBeVisible();
    await expect(page.getByText("0 points")).toBeVisible();
    await expect(page.getByText("Empty")).toBeVisible();
  });
});
