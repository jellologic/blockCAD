import { test, expect } from "@playwright/test";
import { waitForEditor } from "./helpers";

/**
 * Helper: create a box and wait for mesh.
 */
async function createBoxAndWaitForMesh(page: import("@playwright/test").Page) {
  await page.goto("/editor");

  await page.waitForFunction(() => {
    const store = (window as any).__editorStore;
    return store && store.getState().kernel && !store.getState().isLoading;
  }, { timeout: 20000 });

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
        constraints: [
          { id: "sc-0", kind: "fixed", entityIds: ["se-0"] },
          { id: "sc-1", kind: "horizontal", entityIds: ["se-4"] },
          { id: "sc-2", kind: "horizontal", entityIds: ["se-6"] },
          { id: "sc-3", kind: "vertical", entityIds: ["se-5"] },
          { id: "sc-4", kind: "vertical", entityIds: ["se-7"] },
          { id: "sc-5", kind: "distance", entityIds: ["se-0", "se-1"], value: 10 },
          { id: "sc-6", kind: "distance", entityIds: ["se-1", "se-2"], value: 10 },
        ],
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

    store.getState().rebuild();
  });

  await expect(page.locator('[data-testid="vertex-count"]')).toContainText("Verts:", { timeout: 10000 });
}

test.describe("Mass properties / mesh data display", () => {
  test("status bar shows vertex count after model creation", async ({ page }) => {
    await createBoxAndWaitForMesh(page);

    const vertText = await page.locator('[data-testid="vertex-count"]').textContent();
    expect(vertText).toContain("Verts:");
    const count = parseInt(vertText!.replace("Verts: ", ""), 10);
    expect(count).toBeGreaterThan(0);
  });

  test("vertex count increases with more complex geometry", async ({ page }) => {
    await createBoxAndWaitForMesh(page);

    const initialVertText = await page.locator('[data-testid="vertex-count"]').textContent();
    const initialCount = parseInt(initialVertText!.replace("Verts: ", ""), 10);

    // Add fillet to increase geometry complexity
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("fillet", "Fillet 1", {
        type: "fillet",
        params: {
          edge_indices: [0, 1, 2, 3],
          radius: 1,
        },
      });

      store.getState().rebuild();
    });

    await page.waitForTimeout(500);

    const newVertText = await page.locator('[data-testid="vertex-count"]').textContent();
    const newCount = parseInt(newVertText!.replace("Verts: ", ""), 10);

    // Fillet should produce more vertices than a plain box
    expect(newCount).toBeGreaterThan(initialCount);
  });

  test("status bar shows Ready state", async ({ page }) => {
    await createBoxAndWaitForMesh(page);

    await expect(page.locator('[data-testid="status-text"]')).toContainText("Ready");
  });

  test("feature count reflects added features", async ({ page }) => {
    await createBoxAndWaitForMesh(page);

    // Box = sketch + extrude = 2 features
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("2 feature");
  });

  test("mesh data available via store for volume computation", async ({ page }) => {
    await createBoxAndWaitForMesh(page);

    // Verify mesh data is accessible from the store
    const meshInfo = await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const meshData = store.getState().meshData;
      if (!meshData) return null;
      return {
        vertexCount: meshData.vertexCount,
        hasPositions: meshData.positions.length > 0,
        hasIndices: meshData.indices.length > 0,
      };
    });

    expect(meshInfo).not.toBeNull();
    expect(meshInfo!.vertexCount).toBeGreaterThan(0);
    expect(meshInfo!.hasPositions).toBe(true);
    expect(meshInfo!.hasIndices).toBe(true);
  });

  test("triangle count derivable from mesh indices", async ({ page }) => {
    await createBoxAndWaitForMesh(page);

    const triCount = await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const meshData = store.getState().meshData;
      if (!meshData) return 0;
      // Each triangle uses 3 indices
      return meshData.indices.length / 3;
    });

    // A box should have at least 12 triangles (2 per face, 6 faces)
    expect(triCount).toBeGreaterThanOrEqual(12);
  });

  test("mesh bounding box is reasonable for 10x10x10 box", async ({ page }) => {
    await createBoxAndWaitForMesh(page);

    const bounds = await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const meshData = store.getState().meshData;
      if (!meshData) return null;

      let minX = Infinity, maxX = -Infinity;
      let minY = Infinity, maxY = -Infinity;
      let minZ = Infinity, maxZ = -Infinity;

      for (let i = 0; i < meshData.positions.length; i += 3) {
        const x = meshData.positions[i];
        const y = meshData.positions[i + 1];
        const z = meshData.positions[i + 2];
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
        if (z < minZ) minZ = z;
        if (z > maxZ) maxZ = z;
      }

      return { minX, maxX, minY, maxY, minZ, maxZ };
    });

    expect(bounds).not.toBeNull();
    // Box spans ~0 to 10 in each dimension
    expect(bounds!.maxX - bounds!.minX).toBeCloseTo(10, 0);
    expect(bounds!.maxY - bounds!.minY).toBeCloseTo(10, 0);
    expect(bounds!.maxZ - bounds!.minZ).toBeCloseTo(10, 0);
  });
});
