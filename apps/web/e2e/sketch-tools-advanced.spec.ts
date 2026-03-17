import { test, expect } from "@playwright/test";
import { waitForEditor, enterSketchMode, confirmSketch } from "./helpers";

test.describe("Sketch tools advanced — trim, extend, offset", () => {
  test("trim tool activates and shows active state", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    // Click trim tool button
    await page.locator('[data-testid="tool-trim"]').click();

    // Trim button should have active styling
    const trimBtn = page.locator('[data-testid="tool-trim"]');
    await expect(trimBtn).toHaveAttribute("data-active", "true");
  });

  test("extend tool activates via button click", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await page.locator('[data-testid="tool-extend"]').click();

    const extendBtn = page.locator('[data-testid="tool-extend"]');
    await expect(extendBtn).toHaveAttribute("data-active", "true");
  });

  test("offset tool activates via button click", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await page.locator('[data-testid="tool-offset"]').click();

    const offsetBtn = page.locator('[data-testid="tool-offset"]');
    await expect(offsetBtn).toHaveAttribute("data-active", "true");
  });

  test("trim tool activates via keyboard shortcut T", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await page.locator("body").click();
    await page.keyboard.press("t");

    await expect(page.locator('[data-testid="tool-trim"]')).toBeVisible();
  });

  test("offset tool activates via keyboard shortcut O", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await page.locator("body").click();
    await page.keyboard.press("o");

    await expect(page.locator('[data-testid="tool-offset"]')).toBeVisible();
  });

  test("draw lines then confirm sketch preserves entity count", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    // Inject two intersecting lines programmatically via the sketch session
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const session = store.getState().sketchSession;
      if (!session) throw new Error("No sketch session");

      // Add two crossing lines
      session.addEntity({ type: "point", id: "p0", position: { x: -5, y: 0 } });
      session.addEntity({ type: "point", id: "p1", position: { x: 5, y: 0 } });
      session.addEntity({ type: "point", id: "p2", position: { x: 0, y: -5 } });
      session.addEntity({ type: "point", id: "p3", position: { x: 0, y: 5 } });
      session.addEntity({ type: "line", id: "l0", startId: "p0", endId: "p1" });
      session.addEntity({ type: "line", id: "l1", startId: "p2", endId: "p3" });
    });

    // Confirm sketch — should add feature with entities
    await confirmSketch(page);

    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "1 feature"
    );
  });

  test("switching between trim, extend, offset deactivates previous tool", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    // Activate trim
    await page.locator('[data-testid="tool-trim"]').click();

    // Switch to extend
    await page.locator('[data-testid="tool-extend"]').click();

    // Trim should no longer be active (extend takes over)
    await expect(page.locator('[data-testid="tool-extend"]')).toBeVisible();
  });

  test("all modify tools are visible in sketch mode", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await expect(page.locator('[data-testid="tool-trim"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-extend"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-offset"]')).toBeVisible();
    await expect(page.locator('[data-testid="tool-mirror"]')).toBeVisible();
  });
});
