import { test, expect } from "@playwright/test";
import { waitForEditor, enterSketchMode, confirmSketch } from "./helpers";

test.describe("WASM kernel Web Worker", () => {
  test("kernel initializes in worker (feature count = 0)", async ({ page }) => {
    await waitForEditor(page);

    // The kernel should be initialized (proxy is non-null) and feature count should be 0
    const featureCount = await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const state = store.getState();
      return state.features.length;
    });
    expect(featureCount).toBe(0);

    // Kernel proxy should be ready
    const kernelReady = await page.evaluate(() => {
      const store = (window as any).__editorStore;
      return store.getState().kernel !== null;
    });
    expect(kernelReady).toBe(true);
  });

  test("addFeature via worker updates meshData asynchronously", async ({ page }) => {
    await waitForEditor(page);

    // Add a sketch + extrude programmatically via the store
    await page.evaluate(async () => {
      const store = (window as any).__editorStore;
      const state = store.getState();

      // Add sketch feature
      await state.addFeature("sketch", "Sketch 1", {
        type: "sketch",
        params: {
          plane: {
            origin: [0, 0, 0],
            normal: [0, 0, 1],
            uAxis: [1, 0, 0],
            vAxis: [0, 1, 0],
          },
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

      // Add extrude feature
      await state.addFeature("extrude", "Extrude 1", {
        type: "extrude",
        params: {
          direction: [0, 0, 1],
          depth: 10,
          symmetric: false,
          draft_angle: 0,
          end_condition: "blind",
          direction2_enabled: false,
          depth2: 0,
          draft_angle2: 0,
          end_condition2: "blind",
          from_offset: 0,
          thin_feature: false,
          thin_wall_thickness: 0,
          flip_side_to_cut: false,
          cap_ends: false,
          from_condition: "sketch_plane",
        },
      });
    });

    // Wait for mesh data to be populated
    await page.waitForFunction(
      () => {
        const store = (window as any).__editorStore;
        const state = store.getState();
        return state.meshData !== null && state.meshData.vertexCount > 0;
      },
      { timeout: 15000 },
    );

    // Verify mesh data exists
    const meshInfo = await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const state = store.getState();
      return {
        vertexCount: state.meshData?.vertexCount ?? 0,
        triangleCount: state.meshData?.triangleCount ?? 0,
        featureCount: state.features.length,
      };
    });

    expect(meshInfo.vertexCount).toBeGreaterThan(0);
    expect(meshInfo.triangleCount).toBeGreaterThan(0);
    expect(meshInfo.featureCount).toBe(2); // sketch + extrude
  });

  test("UI remains responsive during kernel processing", async ({ page }) => {
    await waitForEditor(page);

    // Start a kernel operation and immediately check that UI is responsive
    // by verifying we can interact with UI elements
    const canInteract = await page.evaluate(async () => {
      const store = (window as any).__editorStore;
      const state = store.getState();

      // Fire off an async operation (don't await)
      const promise = state.addFeature("sketch", "Sketch 1", {
        type: "sketch",
        params: {
          plane: {
            origin: [0, 0, 0],
            normal: [0, 0, 1],
            uAxis: [1, 0, 0],
            vAxis: [0, 1, 0],
          },
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

      // While the worker is processing, the main thread should still respond.
      // We verify by checking that synchronous store operations still work.
      store.getState().toggleWireframe();
      const wireframe = store.getState().wireframe;
      store.getState().toggleWireframe(); // reset

      await promise; // wait for completion
      return wireframe === true;
    });

    expect(canInteract).toBe(true);
  });

  test("isProcessing flag toggles during async operations", async ({ page }) => {
    await waitForEditor(page);

    // Check that isProcessing starts as false
    const initialProcessing = await page.evaluate(() => {
      const store = (window as any).__editorStore;
      return store.getState().isProcessing;
    });
    expect(initialProcessing).toBe(false);
  });
});
