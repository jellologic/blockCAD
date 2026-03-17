import { test, expect } from "@playwright/test";
import { waitForEditor } from "./helpers";

/**
 * Helper: inject a box via kernel for baseline geometry.
 */
async function createBoxViaKernel(page: import("@playwright/test").Page) {
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

test.describe("Sweep advanced workflow", () => {
  test("profile sketch can be created on front plane", async ({ page }) => {
    await waitForEditor(page);

    // Create a profile sketch (circle) that could be used as a sweep profile
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("sketch", "Profile Sketch", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "sp-0", position: { x: 0, y: 0 } },
            { type: "circle", id: "sp-1", centerId: "sp-0", radius: 2 },
          ],
          constraints: [
            { id: "spc-0", kind: "fixed", entityIds: ["sp-0"] },
          ],
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("1 feature", { timeout: 10000 });
  });

  test("path sketch can be created on perpendicular plane", async ({ page }) => {
    await waitForEditor(page);

    // Create profile + path sketches on perpendicular planes
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      // Profile circle on front plane
      kernel.addFeature("sketch", "Profile", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "sp-0", position: { x: 0, y: 0 } },
            { type: "circle", id: "sp-1", centerId: "sp-0", radius: 1 },
          ],
          constraints: [],
        },
      });

      // Path line on right plane
      kernel.addFeature("sketch", "Path", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 0], normal: [1, 0, 0], uAxis: [0, 1, 0], vAxis: [0, 0, 1] },
          entities: [
            { type: "point", id: "pp-0", position: { x: 0, y: 0 } },
            { type: "point", id: "pp-1", position: { x: 0, y: 20 } },
            { type: "line", id: "pp-2", startId: "pp-0", endId: "pp-1" },
          ],
          constraints: [],
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("2 feature", { timeout: 10000 });
  });

  test("extrude creates solid body from profile sketch", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    // Verify mesh was generated (solid body exists)
    await expect(page.locator('[data-testid="vertex-count"]')).toContainText("Verts:", { timeout: 10000 });

    // Vertex count should be positive (box has 8+ vertices when tessellated)
    const vertText = await page.locator('[data-testid="vertex-count"]').textContent();
    const vertCount = parseInt(vertText!.replace("Verts: ", ""), 10);
    expect(vertCount).toBeGreaterThan(0);
  });

  test("revolve operation creates solid from profile", async ({ page }) => {
    await waitForEditor(page);

    // Create a profile and revolve it to test sweep-like behavior
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      // Half-circle profile sketch for revolve
      kernel.addFeature("sketch", "Revolve Profile", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "rp-0", position: { x: 5, y: 0 } },
            { type: "point", id: "rp-1", position: { x: 10, y: 0 } },
            { type: "point", id: "rp-2", position: { x: 10, y: 5 } },
            { type: "point", id: "rp-3", position: { x: 5, y: 5 } },
            { type: "line", id: "rp-4", startId: "rp-0", endId: "rp-1" },
            { type: "line", id: "rp-5", startId: "rp-1", endId: "rp-2" },
            { type: "line", id: "rp-6", startId: "rp-2", endId: "rp-3" },
            { type: "line", id: "rp-7", startId: "rp-3", endId: "rp-0" },
          ],
          constraints: [],
        },
      });

      kernel.addFeature("revolve", "Revolve 1", {
        type: "revolve",
        params: {
          axis_origin: [0, 0, 0],
          axis_direction: [0, 1, 0],
          angle: 6.283185,
          direction2_enabled: false,
          angle2: 0,
          symmetric: false,
          thin_feature: false,
          thin_wall_thickness: 0,
          flip_side_to_cut: false,
        },
      });

      store.getState().rebuild();
    });

    // Should have 2 features and a mesh
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("2 feature", { timeout: 10000 });
    await expect(page.locator('[data-testid="vertex-count"]')).toContainText("Verts:", { timeout: 10000 });
  });

  test("multiple sketches on different planes can coexist", async ({ page }) => {
    await waitForEditor(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      // Sketch on front plane
      kernel.addFeature("sketch", "Front Sketch", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "f-0", position: { x: 0, y: 0 } },
            { type: "circle", id: "f-1", centerId: "f-0", radius: 5 },
          ],
          constraints: [],
        },
      });

      // Sketch on top plane
      kernel.addFeature("sketch", "Top Sketch", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 0], normal: [0, 1, 0], uAxis: [1, 0, 0], vAxis: [0, 0, 1] },
          entities: [
            { type: "point", id: "t-0", position: { x: 0, y: 0 } },
            { type: "point", id: "t-1", position: { x: 0, y: 20 } },
            { type: "line", id: "t-2", startId: "t-0", endId: "t-1" },
          ],
          constraints: [],
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("2 feature", { timeout: 10000 });
    await expect(page.getByText("Front Sketch")).toBeVisible();
    await expect(page.getByText("Top Sketch")).toBeVisible();
  });
});
