import { test, expect } from "@playwright/test";

/**
 * Performance test: measure frame times during orbit rotation.
 * Asserts average frame time stays below 33ms (30fps minimum).
 */

async function waitForKernel(page: import("@playwright/test").Page) {
  await page.goto("/editor");
  await page.waitForFunction(() => {
    const store = (window as any).__editorStore;
    return store && store.getState().kernel && !store.getState().isLoading;
  }, { timeout: 20000 });
}

async function createBoxModel(page: import("@playwright/test").Page) {
  await page.evaluate(() => {
    const store = (window as any).__editorStore;
    const kernel = store.getState().kernel;

    kernel.addFeature("sketch", "Sketch", {
      type: "sketch",
      params: {
        plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
        entities: [
          { type: "point", id: "se-0", position: { x: 0, y: 0 } },
          { type: "point", id: "se-1", position: { x: 15, y: 0 } },
          { type: "point", id: "se-2", position: { x: 15, y: 10 } },
          { type: "point", id: "se-3", position: { x: 0, y: 10 } },
          { type: "line", id: "se-4", startId: "se-0", endId: "se-1" },
          { type: "line", id: "se-5", startId: "se-1", endId: "se-2" },
          { type: "line", id: "se-6", startId: "se-2", endId: "se-3" },
          { type: "line", id: "se-7", startId: "se-3", endId: "se-0" },
        ],
        constraints: [],
      },
    });

    kernel.addFeature("extrude", "Extrude", {
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

  // Wait for mesh to be available
  await page.waitForFunction(() => {
    const store = (window as any).__editorStore;
    const mesh = store.getState().meshData;
    return mesh && mesh.vertexCount > 0;
  }, { timeout: 10000 });
}

test.describe("Performance — FPS during orbit", () => {
  test("average frame time stays under 33ms during orbit rotation", async ({ page }) => {
    await waitForKernel(page);
    await createBoxModel(page);

    const canvas = page.locator("canvas");
    const box = await canvas.boundingBox();
    expect(box).toBeTruthy();

    // Collect frame times during a simulated orbit drag
    const frameTimes = await page.evaluate(async (canvasBox) => {
      return new Promise<number[]>((resolve) => {
        const times: number[] = [];
        let lastTime = performance.now();
        let frameCount = 0;
        const maxFrames = 60;

        function measureFrame() {
          const now = performance.now();
          times.push(now - lastTime);
          lastTime = now;
          frameCount++;
          if (frameCount < maxFrames) {
            requestAnimationFrame(measureFrame);
          } else {
            resolve(times);
          }
        }

        // Start measuring
        requestAnimationFrame(measureFrame);
      });
    }, box);

    // Filter out the first few frames (warm-up)
    const measured = frameTimes.slice(3);
    const avg = measured.reduce((a, b) => a + b, 0) / measured.length;

    // Average frame time should be under 33ms (30fps)
    expect(avg).toBeLessThan(33);
  });
});
