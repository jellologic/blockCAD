import { test, expect } from "@playwright/test";

/**
 * Performance test: memory stability during create/delete cycles.
 * Creates and deletes features 20 times, checks JS heap doesn't grow unboundedly.
 */

async function waitForKernel(page: import("@playwright/test").Page) {
  await page.goto("/editor");
  await page.waitForFunction(() => {
    const store = (window as any).__editorStore;
    return store && store.getState().kernel && !store.getState().isLoading;
  }, { timeout: 20000 });
}

test.describe("Performance — Memory stability", () => {
  test("JS heap does not grow unboundedly during 20 create/delete cycles", async ({ page }) => {
    await waitForKernel(page);

    // Get baseline metrics
    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Performance.enable");

    // Force GC before baseline
    await cdp.send("HeapProfiler.collectGarbage");
    const baselineMetrics = await cdp.send("Performance.getMetrics");
    const baselineHeap = baselineMetrics.metrics.find(m => m.name === "JSHeapUsedSize")?.value ?? 0;

    // Run 20 create/delete cycles
    for (let cycle = 0; cycle < 20; cycle++) {
      await page.evaluate((i) => {
        const store = (window as any).__editorStore;
        const kernel = store.getState().kernel;

        // Create sketch + extrude
        kernel.addFeature("sketch", `Sketch-${i}`, {
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

        kernel.addFeature("extrude", `Extrude-${i}`, {
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
      }, cycle);

      // Suppress features to simulate "delete" (keeps kernel consistent)
      await page.evaluate(() => {
        const store = (window as any).__editorStore;
        const features = store.getState().features;
        for (let i = features.length - 1; i >= 0; i--) {
          store.getState().suppressFeature(i);
        }
      });
    }

    // Force GC and measure final heap
    await cdp.send("HeapProfiler.collectGarbage");
    const finalMetrics = await cdp.send("Performance.getMetrics");
    const finalHeap = finalMetrics.metrics.find(m => m.name === "JSHeapUsedSize")?.value ?? 0;

    // Heap growth should be bounded — allow up to 50MB growth (generous for 20 cycles)
    const heapGrowthMB = (finalHeap - baselineHeap) / (1024 * 1024);
    expect(heapGrowthMB).toBeLessThan(50);
  });
});
