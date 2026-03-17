import { test, expect } from "@playwright/test";
import { waitForEditor, enterSketchMode, confirmSketch } from "./helpers";

test.describe("Sketch fillet and chamfer", () => {
  test("sketch fillet tool activates via button", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await page.locator('[data-testid="tool-sketch-fillet"]').click();

    const btn = page.locator('[data-testid="tool-sketch-fillet"]');
    await expect(btn).toHaveAttribute("data-active", "true");
  });

  test("sketch chamfer tool activates via button", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await page.locator('[data-testid="tool-sketch-chamfer"]').click();

    const btn = page.locator('[data-testid="tool-sketch-chamfer"]');
    await expect(btn).toHaveAttribute("data-active", "true");
  });

  test("sketch fillet activates via keyboard shortcut F", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await page.locator("body").click();
    await page.keyboard.press("f");

    await expect(page.locator('[data-testid="tool-sketch-fillet"]')).toBeVisible();
  });

  test("sketch chamfer activates via keyboard shortcut H", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    await page.locator("body").click();
    await page.keyboard.press("h");

    await expect(page.locator('[data-testid="tool-sketch-chamfer"]')).toBeVisible();
  });

  test("draw intersecting lines, confirm sketch with entities", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    // Inject a right-angle corner (two lines meeting at origin)
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const session = store.getState().sketchSession;
      if (!session) throw new Error("No sketch session");

      session.addEntity({ type: "point", id: "p0", position: { x: -5, y: 0 } });
      session.addEntity({ type: "point", id: "p1", position: { x: 0, y: 0 } });
      session.addEntity({ type: "point", id: "p2", position: { x: 0, y: 5 } });
      session.addEntity({ type: "line", id: "l0", startId: "p0", endId: "p1" });
      session.addEntity({ type: "line", id: "l1", startId: "p1", endId: "p2" });
    });

    await confirmSketch(page);

    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "1 feature"
    );
  });

  test("sketch fillet and chamfer buttons are disabled outside sketch mode", async ({ page }) => {
    await waitForEditor(page);

    // Switch to sketch tab without entering sketch mode
    await page.locator('[data-testid="tab-sketch"]').click();

    const filletBtn = page.locator('[data-testid="tool-sketch-fillet"]');
    const chamferBtn = page.locator('[data-testid="tool-sketch-chamfer"]');
    await expect(filletBtn).toBeVisible();
    await expect(chamferBtn).toBeVisible();
    await expect(filletBtn).toHaveAttribute("disabled", "");
    await expect(chamferBtn).toHaveAttribute("disabled", "");
  });

  test("switching from fillet to chamfer deactivates fillet", async ({ page }) => {
    await waitForEditor(page);
    await enterSketchMode(page, "front");

    // Activate fillet
    await page.locator('[data-testid="tool-sketch-fillet"]').click();

    // Switch to chamfer
    await page.locator('[data-testid="tool-sketch-chamfer"]').click();

    // Chamfer should now be active
    await expect(page.locator('[data-testid="tool-sketch-chamfer"]')).toBeVisible();
  });
});
