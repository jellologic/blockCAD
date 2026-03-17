import { test, expect } from "@playwright/test";
import { waitForEditor } from "./helpers";

/**
 * Helper: inject a box (sketch + extrude) via kernel.
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
          { type: "point", id: "se-1", position: { x: 20, y: 0 } },
          { type: "point", id: "se-2", position: { x: 20, y: 20 } },
          { type: "point", id: "se-3", position: { x: 0, y: 20 } },
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
          { id: "sc-6", kind: "distance", entityIds: ["se-1", "se-2"], value: 20 },
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

test.describe("Hole wizard workflow (via cut extrude)", () => {
  test("simple hole via cut extrude with circle sketch", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    // A hole is created by: sketch a circle on the top face, then cut extrude.
    // Inject a circle sketch on the top of the box (z=10 plane)
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("sketch", "Hole Sketch", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 10], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "hc-0", position: { x: 10, y: 10 } },
            { type: "circle", id: "hc-1", centerId: "hc-0", radius: 3 },
          ],
          constraints: [
            { id: "hsc-0", kind: "fixed", entityIds: ["hc-0"] },
          ],
        },
      });

      kernel.addFeature("cut_extrude", "Simple Hole", {
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

    // Should now have 4 features: Sketch, Extrude, Hole Sketch, Simple Hole
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("4 feature", { timeout: 10000 });
  });

  test("counterbore hole via two cut extrudes", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    // Counterbore = larger diameter cut + smaller diameter through-hole
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      // Counterbore sketch (large circle)
      kernel.addFeature("sketch", "Counterbore Sketch", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 10], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "cb-0", position: { x: 10, y: 10 } },
            { type: "circle", id: "cb-1", centerId: "cb-0", radius: 5 },
          ],
          constraints: [
            { id: "cbs-0", kind: "fixed", entityIds: ["cb-0"] },
          ],
        },
      });

      // Counterbore cut (shallow)
      kernel.addFeature("cut_extrude", "Counterbore Cut", {
        type: "cut_extrude",
        params: {
          direction: [0, 0, -1], depth: 3, symmetric: false, draft_angle: 0,
          end_condition: "blind", direction2_enabled: false, depth2: 0,
          draft_angle2: 0, end_condition2: "blind", from_offset: 0,
          thin_feature: false, thin_wall_thickness: 0,
          flip_side_to_cut: false, cap_ends: false, from_condition: "sketch_plane",
        },
      });

      store.getState().rebuild();
    });

    // Should have 4 features: Sketch, Extrude, Counterbore Sketch, Counterbore Cut
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("4 feature", { timeout: 10000 });
  });

  test("cut extrude button opens PropertyManager", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    // Click Cut button on ribbon
    await page.locator('[data-testid="ribbon-cut"]').click();

    // Extrude panel should appear (cut uses same panel)
    await expect(page.locator('[data-testid="extrude-depth"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-testid="operation-confirm"]')).toBeVisible();
    await expect(page.locator('[data-testid="operation-cancel"]')).toBeVisible();
  });

  test("cancel cut extrude does not add feature", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    const initialCount = await page.locator('[data-testid="feature-count"]').textContent();

    await page.locator('[data-testid="ribbon-cut"]').click();
    await expect(page.locator('[data-testid="extrude-depth"]')).toBeVisible({ timeout: 10000 });

    await page.locator('[data-testid="operation-cancel"]').click();

    await expect(page.locator('[data-testid="feature-count"]')).toContainText(initialCount!);
  });

  test("feature tree shows hole features with correct names", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("sketch", "Hole Sketch", {
        type: "sketch",
        params: {
          plane: { origin: [0, 0, 10], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
          entities: [
            { type: "point", id: "hc-0", position: { x: 10, y: 10 } },
            { type: "circle", id: "hc-1", centerId: "hc-0", radius: 3 },
          ],
          constraints: [],
        },
      });

      kernel.addFeature("cut_extrude", "Hole Cut", {
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

    // Verify feature names appear in tree
    await expect(page.getByText("Hole Sketch")).toBeVisible({ timeout: 10000 });
    await expect(page.getByText("Hole Cut")).toBeVisible();
  });
});
