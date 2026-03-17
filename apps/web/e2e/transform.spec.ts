import { test, expect } from "@playwright/test";
import { waitForEditor } from "./helpers";

/**
 * Helper: create a box via kernel with rebuild.
 */
async function createBoxViaKernel(page: import("@playwright/test").Page) {
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

test.describe("Transform operations — linear pattern, circular pattern, mirror", () => {
  test("linear pattern adds feature to tree", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("linear_pattern", "Linear Pattern 1", {
        type: "linear_pattern",
        params: {
          direction: [1, 0, 0],
          spacing: 15,
          count: 3,
        },
      });

      store.getState().rebuild();
    });

    // Should have 3 features: Sketch, Extrude, Linear Pattern
    await expect(page.locator('[data-testid="feature-count"]')).toContainText("3 feature", { timeout: 10000 });
    await expect(page.getByText("Linear Pattern 1")).toBeVisible();
  });

  test("circular pattern adds feature to tree", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("circular_pattern", "Circular Pattern 1", {
        type: "circular_pattern",
        params: {
          axis_origin: [0, 0, 0],
          axis_direction: [0, 0, 1],
          count: 4,
          total_angle: 6.283185,
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("3 feature", { timeout: 10000 });
    await expect(page.getByText("Circular Pattern 1")).toBeVisible();
  });

  test("mirror adds feature to tree", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("mirror", "Mirror 1", {
        type: "mirror",
        params: {
          plane_origin: [0, 0, 0],
          plane_normal: [1, 0, 0],
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("3 feature", { timeout: 10000 });
    await expect(page.getByText("Mirror 1")).toBeVisible();
  });

  test("linear pattern increases vertex count", async ({ page }) => {
    await createBoxViaKernel(page);

    const initialVertText = await page.locator('[data-testid="vertex-count"]').textContent();
    const initialCount = parseInt(initialVertText!.replace("Verts: ", ""), 10);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("linear_pattern", "Linear Pattern 1", {
        type: "linear_pattern",
        params: {
          direction: [1, 0, 0],
          spacing: 15,
          count: 3,
        },
      });

      store.getState().rebuild();
    });

    await page.waitForTimeout(500);

    const newVertText = await page.locator('[data-testid="vertex-count"]').textContent();
    const newCount = parseInt(newVertText!.replace("Verts: ", ""), 10);

    // Pattern of 3 should produce ~3x the vertices
    expect(newCount).toBeGreaterThan(initialCount);
  });

  test("linear pattern PropertyManager opens via UI", async ({ page }) => {
    await createBoxViaKernel(page);

    // Start linear pattern via store
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().startOperation("linear_pattern");
    });

    await expect(page.locator('[data-testid="linear-pattern-panel"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-testid="operation-confirm"]')).toBeVisible();
    await expect(page.locator('[data-testid="operation-cancel"]')).toBeVisible();
  });

  test("circular pattern PropertyManager opens via store", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().startOperation("circular_pattern");
    });

    await expect(page.locator('[data-testid="circular-pattern-panel"]')).toBeVisible({ timeout: 10000 });
  });

  test("mirror PropertyManager opens via store", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().startOperation("mirror");
    });

    await expect(page.locator('[data-testid="mirror-panel"]')).toBeVisible({ timeout: 10000 });
  });

  test("cancel linear pattern does not add feature", async ({ page }) => {
    await createBoxViaKernel(page);

    const initialCount = await page.locator('[data-testid="feature-count"]').textContent();

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().startOperation("linear_pattern");
    });

    await expect(page.locator('[data-testid="linear-pattern-panel"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-testid="operation-cancel"]').click();

    await expect(page.locator('[data-testid="feature-count"]')).toContainText(initialCount!);
  });
});
