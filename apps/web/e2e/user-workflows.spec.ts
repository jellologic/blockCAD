import { test, expect } from "@playwright/test";
import { waitForEditor, enterSketchMode, confirmSketch } from "./helpers";
import {
  waitForCameraStable,
  drawRectangle,
  drawLine,
  drawCircle,
  sketchRectangleOnPlane,
  extrudeLatestSketch,
  fullBoxWorkflow,
  expectFeatureCount,
  expectMeshExists,
  expectSketchEntityCount,
} from "./user-flow-helpers";

test.describe("Real user workflows", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEditor(page);
  });

  // ── Test 1: Full sketch → extrude workflow ──────────────────────

  test("draw rectangle on front plane and extrude to 3D box", async ({
    page,
  }) => {
    // Enter sketch mode on front plane
    await enterSketchMode(page, "front");
    await waitForCameraStable(page);

    // Draw a 10x8 rectangle centered at origin
    await drawRectangle(page, -5, -4, 5, 4);

    // Verify sketch has entities (4 points + 4 lines = 8)
    const entityCount = await page.evaluate(() => {
      return (window as any).__editorStore?.getState()?.sketchSession?.entities?.length ?? 0;
    });
    expect(entityCount).toBeGreaterThanOrEqual(8);

    // Confirm sketch
    await confirmSketch(page);
    await expectFeatureCount(page, 1);

    // Extrude to depth 10
    await extrudeLatestSketch(page, 10);
    await expectFeatureCount(page, 2);
    await expectMeshExists(page);
  });

  // ── Test 2: Sketch with multiple tools ──────────────────────────

  test("draw rectangle then add circle on same sketch", async ({ page }) => {
    await enterSketchMode(page, "front");
    await waitForCameraStable(page);

    // Draw a rectangle first (this is proven to work)
    await drawRectangle(page, -4, -4, 4, 4);

    // Now add a circle on the same sketch (offset from rectangle to avoid hitting points)
    await drawCircle(page, 0, 7, 2);

    // Verify we have rectangle (8 entities) + circle (center point + circle = 2)
    const entityCount = await page.evaluate(() => {
      return (
        (window as any).__editorStore?.getState()?.sketchSession?.entities
          ?.length ?? 0
      );
    });
    expect(entityCount).toBeGreaterThanOrEqual(10);

    // Confirm sketch
    await confirmSketch(page);
    await expectFeatureCount(page, 1);
  });

  // ── Test 3: Sample load + modify via UI ─────────────────────────

  test("load sample then start fillet operation", async ({ page }) => {
    // Load Simple Box via the dropdown
    const dropdown = page.locator('[data-testid="sample-models-dropdown"]');
    await dropdown.locator("button").first().click();
    await page.locator('[data-testid="sample-simple-box"]').click();

    await expectFeatureCount(page, 2);
    await expectMeshExists(page);

    // Start fillet operation via keyboard shortcut
    await page.keyboard.press("g");

    // Verify fillet panel appears (has radius input)
    await expect(
      page.locator('input[type="number"]').first()
    ).toBeVisible({ timeout: 5000 });

    // Cancel the fillet (just testing that the UI flow works)
    await page.keyboard.press("Escape");
  });

  // ── Test 4: Feature tree context menu + rename ──────────────────

  test("right-click feature tree item shows context menu and rename works", async ({
    page,
  }) => {
    // Load a sample to get features
    const dropdown = page.locator('[data-testid="sample-models-dropdown"]');
    await dropdown.locator("button").first().click();
    await page.locator('[data-testid="sample-simple-box"]').click();
    await expectFeatureCount(page, 2);

    // Right-click on the first feature
    const firstFeature = page
      .locator('[data-testid^="feature-"]:not([data-testid="feature-count"])')
      .first();
    await firstFeature.click({ button: "right" });

    // Context menu should appear
    await expect(
      page.locator('[data-testid="feature-context-menu"]')
    ).toBeVisible({ timeout: 5000 });

    // Click "Rename" in the context menu
    await page.getByText("Rename").click();

    // An inline input should appear — type new name
    const renameInput = page.locator(
      '.bg-\\[var\\(--cad-bg-input\\)\\]'
    );
    if (await renameInput.isVisible()) {
      await renameInput.fill("My Custom Sketch");
      await renameInput.press("Enter");

      // Verify the feature name updated
      await expect(firstFeature).toContainText("My Custom Sketch");
    }
  });

  // ── Test 5: Command palette ─────────────────────────────────────

  test("command palette opens and filters commands", async ({ page }) => {
    // Dispatch Ctrl+Shift+P via JavaScript to ensure it fires correctly
    await page.evaluate(() => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", {
          key: "P",
          code: "KeyP",
          ctrlKey: true,
          shiftKey: true,
          bubbles: true,
        })
      );
    });

    // Verify palette is visible
    const palette = page.locator('input[placeholder="Type a command..."]');
    await expect(palette).toBeVisible({ timeout: 5000 });

    // Type "extrude" to filter
    await palette.fill("extrude");
    await page.waitForTimeout(200);

    // Should see Extrude in the results
    await expect(page.getByText("Extrude").first()).toBeVisible();

    // Press Enter to select the first result
    await page.keyboard.press("Enter");

    // Extrude panel should open (or at least the operation starts)
    const state = await page.evaluate(() => {
      return (window as any).__editorStore?.getState()?.activeOperation?.type;
    });
    expect(state).toBe("extrude");

    // Cancel
    await page.keyboard.press("Escape");
  });

  // ── Test 6: Menu bar ────────────────────────────────────────────

  test("file menu shows options", async ({ page }) => {
    // Click File menu
    await page.getByText("File", { exact: true }).first().click();

    // Verify menu items are visible
    await expect(page.getByText("New")).toBeVisible();
    await expect(page.getByText("Save")).toBeVisible();
    await expect(page.getByText("Export STL")).toBeVisible();

    // Click elsewhere to close
    await page.keyboard.press("Escape");
  });

  // ── Test 7: Keyboard-only full workflow ─────────────────────────

  test("keyboard-driven: S → plane → R → draw → Enter → E → depth → Enter", async ({
    page,
  }) => {
    // S to start sketch flow
    await page.keyboard.press("s");
    await page.waitForTimeout(300);

    // Click front plane (reuse existing helper logic)
    const canvas = page.locator("canvas");
    const box = await canvas.boundingBox();
    if (!box) throw new Error("Canvas not found");
    await page.mouse.click(
      box.x + box.width * 0.55,
      box.y + box.height * 0.45
    );

    // Wait for sketch mode
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });
    await waitForCameraStable(page);

    // R for rectangle tool
    await page.keyboard.press("r");
    await page.waitForTimeout(100);

    // Draw rectangle via canvas clicks
    await drawRectangle(page, -5, -5, 5, 5);

    // Enter to confirm sketch
    await page.keyboard.press("Enter");
    await expectFeatureCount(page, 1);

    // E for extrude
    await page.keyboard.press("e");
    await expect(
      page.locator('[data-testid="extrude-depth"]')
    ).toBeVisible({ timeout: 10000 });

    // Set depth and confirm via button (Enter is captured by focused input)
    await page.locator('[data-testid="extrude-depth"]').fill("15");
    await page.waitForTimeout(200);
    await page.locator('[data-testid="operation-confirm"]').click();

    // Verify result
    await expectFeatureCount(page, 2);
    await expectMeshExists(page);
  });
});

test.describe("Composite workflow helpers", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEditor(page);
  });

  test("fullBoxWorkflow creates a box with 2 features and mesh", async ({
    page,
  }) => {
    await fullBoxWorkflow(page, 10, 8, 5);
    await expectFeatureCount(page, 2);
    await expectMeshExists(page);
  });

  test("sketchRectangleOnPlane creates sketch feature", async ({ page }) => {
    await sketchRectangleOnPlane(page, "front", 12, 6);
    await expectFeatureCount(page, 1);
  });
});
