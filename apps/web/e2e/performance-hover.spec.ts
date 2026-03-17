import { test, expect } from "@playwright/test";

/**
 * Performance test: rapid face hover should not cause frame drops.
 * Asserts no single frame exceeds 100ms during rapid pointer movement.
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

  await page.waitForFunction(() => {
    const store = (window as any).__editorStore;
    const mesh = store.getState().meshData;
    return mesh && mesh.vertexCount > 0;
  }, { timeout: 10000 });
}

test.describe("Performance — Hover responsiveness", () => {
  test("rapid hover over faces causes no frame drops (no frame > 100ms)", async ({ page }) => {
    await waitForKernel(page);
    await createBoxModel(page);

    // Enter select-face mode to enable hover highlighting
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().setMode("select-face");
    });

    const canvas = page.locator("canvas");
    const box = await canvas.boundingBox();
    expect(box).toBeTruthy();

    // Measure frame times while rapidly moving the mouse across the canvas
    const frameTimes = await page.evaluate(async (canvasBox) => {
      return new Promise<number[]>((resolve) => {
        const times: number[] = [];
        let lastTime = performance.now();
        let frameCount = 0;
        const maxFrames = 40;

        // Simulate rapid pointer movements via dispatching events
        const canvasEl = document.querySelector("canvas")!;
        let step = 0;
        const totalSteps = 20;
        const interval = setInterval(() => {
          if (step >= totalSteps) {
            clearInterval(interval);
            return;
          }
          const x = canvasBox!.x + canvasBox!.width * (0.3 + 0.4 * (step / totalSteps));
          const y = canvasBox!.y + canvasBox!.height * (0.3 + 0.2 * Math.sin(step * 0.5));
          canvasEl.dispatchEvent(new PointerEvent("pointermove", {
            clientX: x, clientY: y, bubbles: true,
          }));
          step++;
        }, 16);

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

        requestAnimationFrame(measureFrame);
      });
    }, box);

    // No single frame should exceed 100ms
    const maxFrame = Math.max(...frameTimes.slice(2));
    expect(maxFrame).toBeLessThan(100);
  });
});
