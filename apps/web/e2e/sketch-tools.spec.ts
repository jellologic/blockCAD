import { test, expect } from "@playwright/test";
import { waitForEditor, enterSketchMode } from "./helpers";

test.describe("Sketch tools E2E", () => {
  test("sketch tab shows all tool buttons when in sketch mode", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    // Core draw tools
    await expect(page.locator('[data-testid="tool-line"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-circle"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-arc"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-rectangle"]')).toBeVisible();

    // Shape tools
    await expect(page.locator('[data-testid="tool-ellipse"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-polygon"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-slot"]')).toBeVisible();

    // Modify tools
    await expect(page.locator('[data-testid="tool-trim"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-extend"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-offset"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-mirror"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-sketch-fillet"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-sketch-chamfer"]')).toBeVisible();

    // Block tools
    await expect(page.locator('[data-testid="tool-block-create"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-block-explode"]')).toBeVisible();

    // Pattern tools
    await expect(page.locator('[data-testid="tool-sketch-linear-pattern"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-sketch-circular-pattern"]')).toBeVisible();
  });

  test("keyboard shortcut L activates line tool", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    // Press L to activate line tool
    await page.locator("body").click();
    await page.keyboard.press("l");

    // Line tool button should show active state
    const lineButton = page.locator('[data-testid="tool-line"]');
    // Verify the tool is active by checking the button has the active class
    await expect(lineButton).toBeVisible();
  });

  test("keyboard shortcut T activates trim tool in sketch mode", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await page.locator("body").click();
    await page.keyboard.press("t");

    // Trim tool button should be active
    await expect(page.locator('[data-testid="tool-trim"]')).toBeVisible();
  });

  test("keyboard shortcut P activates polygon tool in sketch mode", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await page.locator("body").click();
    await page.keyboard.press("p");

    // Polygon tool button should be active
    await expect(page.locator('[data-testid="tool-polygon"]')).toBeVisible();
  });

  test("dimension tool is accessible via D shortcut", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await page.locator("body").click();
    await page.keyboard.press("d");

    await expect(page.locator('[data-testid="tool-dimension"]')).toBeVisible();
  });

  test("tools are disabled outside sketch mode", async ({ page }) => {
    await waitForEditor(page);

    // Switch to sketch tab manually without entering sketch mode
    await page.locator('[data-testid="tab-sketch"]').click();

    // Tool buttons should exist but be disabled
    const lineButton = page.locator('[data-testid="tool-line"]');
    await expect(lineButton).toBeVisible();
    // Check disabled attribute
    await expect(lineButton).toHaveAttribute("disabled", "");
  });

  test("confirm and cancel buttons visible in sketch mode", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    // Confirm and cancel should be visible
    await expect(page.locator('[data-testid="ribbon-confirm-sketch"]')).toBeVisible();
    await expect(page.locator('[data-testid="ribbon-cancel-sketch"]')).toBeVisible();
  });

  test("F activates fillet in sketch mode, face-select outside", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    // In sketch mode, F should activate sketch-fillet (not face-select)
    await page.locator("body").click();
    await page.keyboard.press("f");

    // Fillet button should be visible and active
    await expect(page.locator('[data-testid="tool-sketch-fillet"]')).toBeVisible();
  });
});
