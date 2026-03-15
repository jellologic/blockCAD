import { test, expect } from "@playwright/test";
import { waitForEditor } from "./helpers";

test.describe("Extrude workflow", () => {
  test("opens extrude PropertyManager and confirms", async ({ page }) => {
    await waitForEditor(page);

    // Click extrude button
    await page.locator('[data-testid="ribbon-extrude"]').click();

    // Wait for PropertyManager to appear with depth input
    await expect(
      page.locator('[data-testid="extrude-depth"]')
    ).toBeVisible({ timeout: 10000 });

    // Operation confirm/cancel buttons should be visible
    await expect(
      page.locator('[data-testid="operation-confirm"]')
    ).toBeVisible();

    // Set depth to 15
    await page.locator('[data-testid="extrude-depth"]').fill("15");

    // Confirm
    await page.locator('[data-testid="operation-confirm"]').click();

    // Status bar should show Ready after confirming
    await expect(page.locator('[data-testid="status-text"]')).toContainText(
      "Ready"
    );
  });

  test("cancel extrude does not add feature", async ({ page }) => {
    await waitForEditor(page);
    const initialCount = await page
      .locator('[data-testid="feature-count"]')
      .textContent();

    await page.locator('[data-testid="ribbon-extrude"]').click();
    await expect(
      page.locator('[data-testid="operation-cancel"]')
    ).toBeVisible({ timeout: 10000 });

    await page.locator('[data-testid="operation-cancel"]').click();

    // Feature count should be unchanged
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      initialCount!
    );
  });
});
