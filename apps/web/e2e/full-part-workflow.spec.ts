import { test, expect } from "@playwright/test";

/**
 * Full multi-operation workflow test.
 * Creates a complete part: Sketch -> Extrude -> Fillet -> Hole (Cut) -> Shell -> Linear Pattern.
 * Validates the feature tree and mesh at each step.
 */

async function waitForKernel(page: import("@playwright/test").Page) {
  await page.goto("/editor");
  await page.waitForFunction(() => {
    const store = (window as any).__editorStore;
    return store && store.getState().kernel && !store.getState().isLoading;
  }, { timeout: 20000 });
}

test.describe("Full part workflow — multi-operation pipeline", () => {
  test("complete workflow: sketch + extrude + fillet + cut + shell + linear pattern", async ({ page }) => {
    await waitForKernel(page);

    // Step 1: Create sketch with rectangle
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("sketch", "Base Sketch", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "se-0", position: { x: 0, y: 0 } },
            { type: "point", id: "se-1", position: { x: 20, y: 0 } },
            { type: "point", id: "se-2", position: { x: 20, y: 15 } },
            { type: "point", id: "se-3", position: { x: 0, y: 15 } },
            { type: "line", id: "se-4", startId: "se-0", endId: "se-1" },
            { type: "line", id: "se-5", startId: "se-1", endId: "se-2" },
            { type: "line", id: "se-6", startId: "se-2", endId: "se-3" },
            { type: "line", id: "se-7", startId: "se-3", endId: "se-0" },
          ],
          constraints: [
            { id: "sc-0", kind: "fixed", entityIds: ["se-0"] },
            { id: "sc-1", kind: "horizontal", entityIds: ["se-4"] },
            { id: "sc-2", kind: "horizontal", entityIds: ["se-6"] },
            { id: "sc-3", kind: "vertical", entityIds: ["se-5"] },
            { id: "sc-4", kind: "vertical", entityIds: ["se-7"] },
            { id: "sc-5", kind: "distance", entityIds: ["se-0", "se-1"], value: 20 },
            { id: "sc-6", kind: "distance", entityIds: ["se-1", "se-2"], value: 15 },
          ],
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("1 feature", { timeout: 10000 });
    await expect(page.getByText("Base Sketch")).toBeVisible();

    // Step 2: Extrude the sketch
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("extrude", "Base Extrude", {
        type: "extrude",
        params: {
          direction: [0, 0, 1], depth: 10, symmetric: false, draft_angle: 0,
          end_condition: "blind", direction2_enabled: false, depth2: 0,
          draft_angle2: 0, end_condition2: "blind", from_offset: 0,
          thin_feature: false, thin_wall_thickness: 0,
          flip_side_to_cut: false, cap_ends: false, from_condition: "sketch_plane",
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("2 feature", { timeout: 10000 });
    await expect(page.locator('[data-testid="vertex-count"]')).toContainText("Verts:");
    await expect(page.getByText("Base Extrude")).toBeVisible();

    // Step 3: Apply fillet to edges
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("fillet", "Edge Fillet", {
        type: "fillet",
        params: {
          edge_indices: [0, 1, 2, 3],
          radius: 1,
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("3 feature", { timeout: 10000 });
    await expect(page.getByText("Edge Fillet")).toBeVisible();

    // Step 4: Add hole via cut extrude (circle sketch + cut)
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("sketch", "Hole Sketch", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 10], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "hc-0", position: { x: 10, y: 7.5 } },
            { type: "circle", id: "hc-1", centerId: "hc-0", radius: 2 },
          ],
          constraints: [
            { id: "hsc-0", kind: "fixed", entityIds: ["hc-0"] },
          ],
        },
      });

      kernel.addFeature("cut_extrude", "Through Hole", {
        type: "cut_extrude",
        params: {
          direction: [0, 0, -1], depth: 10, symmetric: false, draft_angle: 0,
          end_condition: "blind", direction2_enabled: false, depth2: 0,
          draft_angle2: 0, end_condition2: "blind", from_offset: 0,
          thin_feature: false, thin_wall_thickness: 0,
          flip_side_to_cut: false, cap_ends: false, from_condition: "sketch_plane",
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("5 feature", { timeout: 10000 });
    await expect(page.getByText("Hole Sketch")).toBeVisible();
    await expect(page.getByText("Through Hole")).toBeVisible();

    // Step 5: Apply shell
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("shell", "Shell", {
        type: "shell",
        params: {
          faces_to_remove: [5],
          thickness: 0.5,
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("6 feature", { timeout: 10000 });
    await expect(page.getByText("Shell")).toBeVisible();

    // Step 6: Apply linear pattern
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("linear_pattern", "Linear Pattern", {
        type: "linear_pattern",
        params: {
          direction: [1, 0, 0],
          spacing: 25,
          count: 2,
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("7 feature", { timeout: 10000 });
    await expect(page.getByText("Linear Pattern")).toBeVisible();

    // Final verification: mesh exists with significant geometry
    const vertText = await page.locator('[data-testid="vertex-count"]').textContent();
    const vertCount = parseInt(vertText!.replace("Verts: ", ""), 10);
    expect(vertCount).toBeGreaterThan(0);
  });

  test("feature tree shows all features in correct order", async ({ page }) => {
    await waitForKernel(page);

    // Build a multi-feature model
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("sketch", "Sketch 1", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "se-0", position: { x: 0, y: 0 } },
            { type: "point", id: "se-1", position: { x: 10, y: 0 } },
            { type: "point", id: "se-2", position: { x: 10, y: 10 } },
            { type: "point", id: "se-3", position: { x: 0, y: 10 } },
            { type: "line", id: "se-4", startId: "se-0", endId: "se-1" },
            { type: "line", id: "se-5", startId: "se-1", endId: "se-2" },
            { type: "line", id: "se-6", startId: "se-2", endId: "se-3" },
            { type: "line", id: "se-7", startId: "se-3", endId: "se-0" },
          ],
          constraints: [],
        },
      });

      kernel.addFeature("extrude", "Extrude 1", {
        type: "extrude",
        params: {
          direction: [0, 0, 1], depth: 10, symmetric: false, draft_angle: 0,
          end_condition: "blind", direction2_enabled: false, depth2: 0,
          draft_angle2: 0, end_condition2: "blind", from_offset: 0,
          thin_feature: false, thin_wall_thickness: 0,
          flip_side_to_cut: false, cap_ends: false, from_condition: "sketch_plane",
        },
      });

      kernel.addFeature("chamfer", "Chamfer 1", {
        type: "chamfer",
        params: {
          edge_indices: [0],
          distance: 1,
        },
      });

      store.getState().rebuild();
    });

    // All 3 features should be in the tree
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("3 feature", { timeout: 10000 });
    await expect(page.getByText("Sketch 1")).toBeVisible();
    await expect(page.getByText("Extrude 1")).toBeVisible();
    await expect(page.getByText("Chamfer 1")).toBeVisible();
  });

  test("status bar shows Ready after full pipeline", async ({ page }) => {
    await waitForKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("sketch", "S1", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "se-0", position: { x: 0, y: 0 } },
            { type: "point", id: "se-1", position: { x: 10, y: 0 } },
            { type: "point", id: "se-2", position: { x: 10, y: 10 } },
            { type: "point", id: "se-3", position: { x: 0, y: 10 } },
            { type: "line", id: "se-4", startId: "se-0", endId: "se-1" },
            { type: "line", id: "se-5", startId: "se-1", endId: "se-2" },
            { type: "line", id: "se-6", startId: "se-2", endId: "se-3" },
            { type: "line", id: "se-7", startId: "se-3", endId: "se-0" },
          ],
          constraints: [],
        },
      });

      kernel.addFeature("extrude", "E1", {
        type: "extrude",
        params: {
          direction: [0, 0, 1], depth: 10, symmetric: false, draft_angle: 0,
          end_condition: "blind", direction2_enabled: false, depth2: 0,
          draft_angle2: 0, end_condition2: "blind", from_offset: 0,
          thin_feature: false, thin_wall_thickness: 0,
          flip_side_to_cut: false, cap_ends: false, from_condition: "sketch_plane",
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="status-text"]')).toContainText("Ready", { timeout: 10000 });
  });

  test("full pipeline produces exportable mesh", async ({ page }) => {
    await waitForKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("sketch", "Sketch 1", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "se-0", position: { x: 0, y: 0 } },
            { type: "point", id: "se-1", position: { x: 10, y: 0 } },
            { type: "point", id: "se-2", position: { x: 10, y: 10 } },
            { type: "point", id: "se-3", position: { x: 0, y: 10 } },
            { type: "line", id: "se-4", startId: "se-0", endId: "se-1" },
            { type: "line", id: "se-5", startId: "se-1", endId: "se-2" },
            { type: "line", id: "se-6", startId: "se-2", endId: "se-3" },
            { type: "line", id: "se-7", startId: "se-3", endId: "se-0" },
          ],
          constraints: [],
        },
      });

      kernel.addFeature("extrude", "Extrude 1", {
        type: "extrude",
        params: {
          direction: [0, 0, 1], depth: 10, symmetric: false, draft_angle: 0,
          end_condition: "blind", direction2_enabled: false, depth2: 0,
          draft_angle2: 0, end_condition2: "blind", from_offset: 0,
          thin_feature: false, thin_wall_thickness: 0,
          flip_side_to_cut: false, cap_ends: false, from_condition: "sketch_plane",
        },
      });

      kernel.addFeature("fillet", "Fillet 1", {
        type: "fillet",
        params: { edge_indices: [0, 1, 2, 3], radius: 1 },
      });

      store.getState().rebuild();
    });

    // Verify mesh exists for export
    await expect(page.locator('[data-testid="vertex-count"]')).toContainText("Verts:", { timeout: 10000 });

    // Switch to View tab and verify export buttons are enabled
    await page.click('[data-testid="tab-view"]');
    const stlBtn = page.locator('[data-testid="export-stl"]');
    await expect(stlBtn).toBeVisible();

    // Export button should NOT be disabled (mesh exists)
    const isDisabled = await stlBtn.getAttribute("disabled");
    expect(isDisabled).toBeNull();
  });

  test("kernel feature list matches store features", async ({ page }) => {
    await waitForKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("sketch", "My Sketch", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "se-0", position: { x: 0, y: 0 } },
            { type: "point", id: "se-1", position: { x: 5, y: 0 } },
            { type: "point", id: "se-2", position: { x: 5, y: 5 } },
            { type: "point", id: "se-3", position: { x: 0, y: 5 } },
            { type: "line", id: "se-4", startId: "se-0", endId: "se-1" },
            { type: "line", id: "se-5", startId: "se-1", endId: "se-2" },
            { type: "line", id: "se-6", startId: "se-2", endId: "se-3" },
            { type: "line", id: "se-7", startId: "se-3", endId: "se-0" },
          ],
          constraints: [],
        },
      });

      kernel.addFeature("extrude", "My Extrude", {
        type: "extrude",
        params: {
          direction: [0, 0, 1], depth: 5, symmetric: false, draft_angle: 0,
          end_condition: "blind", direction2_enabled: false, depth2: 0,
          draft_angle2: 0, end_condition2: "blind", from_offset: 0,
          thin_feature: false, thin_wall_thickness: 0,
          flip_side_to_cut: false, cap_ends: false, from_condition: "sketch_plane",
        },
      });

      store.getState().rebuild();
    });

    // Kernel should report same number of features
    const featureCount = await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;
      return kernel.featureList().length;
    });

    expect(featureCount).toBe(2);

    await expect(page.getByText("My Sketch")).toBeVisible({ timeout: 10000 });
    await expect(page.getByText("My Extrude")).toBeVisible();
  });
});
