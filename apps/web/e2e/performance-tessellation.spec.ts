import { test, expect } from "@playwright/test";

/**
 * Performance test: measure tessellation time for a multi-feature model.
 * Asserts that adding a feature and getting meshData takes < 500ms.
 */

async function waitForKernel(page: import("@playwright/test").Page) {
  await page.goto("/editor");
  await page.waitForFunction(() => {
    const store = (window as any).__editorStore;
    return store && store.getState().kernel && !store.getState().isLoading;
  }, { timeout: 20000 });
}

test.describe("Performance — Tessellation timing", () => {
  test("5-feature model tessellates in under 500ms", async ({ page }) => {
    await waitForKernel(page);

    // Build a 5-feature model and measure the time for the final addFeature + tessellate
    const tessellationTime = await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      // Feature 1: Base sketch
      kernel.addFeature("sketch", "Sketch 1", {
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
          constraints: [],
        },
      });

      // Feature 2: Extrude
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

      // Feature 3: Fillet
      kernel.addFeature("fillet", "Fillet 1", {
        type: "fillet",
        params: { edge_indices: [0, 1], radius: 1 },
      });

      // Feature 4: Chamfer
      kernel.addFeature("chamfer", "Chamfer 1", {
        type: "chamfer",
        params: { edge_indices: [4], distance: 0.5 },
      });

      // Feature 5: Measure tessellation of the complete model
      const start = performance.now();
      kernel.addFeature("linear_pattern", "Pattern 1", {
        type: "linear_pattern",
        params: { direction: [1, 0, 0], spacing: 25, count: 2 },
      });
      const mesh = kernel.tessellate();
      const elapsed = performance.now() - start;

      store.getState().rebuild();
      return elapsed;
    });

    // Tessellation should complete in under 500ms
    expect(tessellationTime).toBeLessThan(500);
  });
});
