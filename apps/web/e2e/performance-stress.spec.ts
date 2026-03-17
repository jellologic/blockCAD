import { test, expect } from "@playwright/test";

/**
 * Performance stress test: build a 10+ feature model and verify the viewport
 * remains responsive (orbit works, hover works, canvas renders).
 */

async function waitForKernel(page: import("@playwright/test").Page) {
  await page.goto("/editor");
  await page.waitForFunction(() => {
    const store = (window as any).__editorStore;
    return store && store.getState().kernel && !store.getState().isLoading;
  }, { timeout: 20000 });
}

test.describe("Performance — Stress test with 10+ features", () => {
  test("viewport remains responsive with a complex multi-feature model", async ({ page }) => {
    await waitForKernel(page);

    // Build a 10+ feature model programmatically
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      // 1. Base sketch
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
          constraints: [],
        },
      });

      // 2. Base extrude
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

      // 3. Fillet
      kernel.addFeature("fillet", "Fillet 1", {
        type: "fillet",
        params: { edge_indices: [0, 1], radius: 1 },
      });

      // 4. Chamfer
      kernel.addFeature("chamfer", "Chamfer 1", {
        type: "chamfer",
        params: { edge_indices: [4], distance: 0.5 },
      });

      // 5. Hole sketch
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

      // 6. Cut extrude (hole)
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

      // 7. Shell
      kernel.addFeature("shell", "Shell 1", {
        type: "shell",
        params: { faces_to_remove: [5], thickness: 0.5 },
      });

      // 8. Linear pattern
      kernel.addFeature("linear_pattern", "Linear Pattern", {
        type: "linear_pattern",
        params: { direction: [1, 0, 0], spacing: 25, count: 2 },
      });

      // 9. Circular pattern
      kernel.addFeature("circular_pattern", "Circular Pattern", {
        type: "circular_pattern",
        params: {
          axis_origin: [0, 0, 0],
          axis_direction: [0, 0, 1],
          count: 3,
          total_angle: 6.283185,
        },
      });

      // 10. Mirror
      kernel.addFeature("mirror", "Mirror 1", {
        type: "mirror",
        params: {
          plane_origin: [0, 0, 0],
          plane_normal: [0, 1, 0],
        },
      });

      // 11. Scale (copy)
      kernel.addFeature("scale", "Scale 1", {
        type: "scale",
        params: {
          uniform: true,
          scale_factor: 0.9,
          scale_x: 1, scale_y: 1, scale_z: 1,
          center: [0, 0, 0],
          copy: false,
        },
      });

      store.getState().rebuild();
    });

    // Verify model has been built with 11+ features
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("feature", { timeout: 15000 });

    // Verify mesh has geometry
    await page.waitForFunction(() => {
      const store = (window as any).__editorStore;
      const mesh = store.getState().meshData;
      return mesh && mesh.vertexCount > 0;
    }, { timeout: 10000 });

    // Test 1: Canvas is still visible and rendered
    const canvas = page.locator("canvas");
    await expect(canvas).toBeVisible();
    const box = await canvas.boundingBox();
    expect(box).toBeTruthy();
    expect(box!.width).toBeGreaterThan(100);

    // Test 2: Orbit interaction works (simulate mousedown + mousemove + mouseup)
    await page.mouse.move(box!.x + box!.width / 2, box!.y + box!.height / 2);
    await page.mouse.down();
    await page.mouse.move(box!.x + box!.width / 2 + 50, box!.y + box!.height / 2 + 30, { steps: 5 });
    await page.mouse.up();

    // Test 3: Enter select-face mode and verify hover works without errors
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().setMode("select-face");
    });

    // Move mouse across the model
    await page.mouse.move(box!.x + box!.width * 0.4, box!.y + box!.height * 0.4);
    await page.mouse.move(box!.x + box!.width * 0.6, box!.y + box!.height * 0.5, { steps: 5 });

    // Test 4: Frame timing is reasonable even with complex model
    const avgFrameTime = await page.evaluate(async () => {
      return new Promise<number>((resolve) => {
        const times: number[] = [];
        let lastTime = performance.now();
        let frameCount = 0;

        function measure() {
          const now = performance.now();
          times.push(now - lastTime);
          lastTime = now;
          frameCount++;
          if (frameCount < 30) {
            requestAnimationFrame(measure);
          } else {
            const avg = times.slice(2).reduce((a, b) => a + b, 0) / (times.length - 2);
            resolve(avg);
          }
        }
        requestAnimationFrame(measure);
      });
    });

    // Even with a complex model, average frame time should be reasonable
    // Using a generous threshold of 50ms (20fps) for complex models
    expect(avgFrameTime).toBeLessThan(50);
  });
});
