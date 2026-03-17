import { test, expect } from "@playwright/test";
import { waitForEditor } from "./helpers";

/**
 * Helper: inject a box (sketch + extrude) programmatically via kernel,
 * then rebuild so the mesh and feature tree are up-to-date.
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

  // Wait for mesh to render
  await expect(page.locator('[data-testid="vertex-count"]')).toContainText("Verts:", { timeout: 10000 });
}

test.describe("Variable fillet workflow", () => {
  test("fillet operation opens PropertyManager with radius input", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    // Start fillet operation via keyboard shortcut G
    await page.locator("body").click();
    await page.keyboard.press("g");

    // Fillet panel should appear
    await expect(page.locator('[data-testid="fillet-panel"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-testid="fillet-radius"]')).toBeVisible();
  });

  test("fillet radius input accepts numeric value", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    await page.locator("body").click();
    await page.keyboard.press("g");

    await expect(page.locator('[data-testid="fillet-radius"]')).toBeVisible({ timeout: 10000 });

    // Set radius to 2
    await page.locator('[data-testid="fillet-radius"]').fill("2");

    const val = await page.locator('[data-testid="fillet-radius"]').inputValue();
    expect(val).toBe("2");
  });

  test("fillet has select-edges button", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    await page.locator("body").click();
    await page.keyboard.press("g");

    await expect(page.locator('[data-testid="fillet-select-edges"]')).toBeVisible({ timeout: 10000 });
  });

  test("fillet confirm and cancel buttons are visible", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    await page.locator("body").click();
    await page.keyboard.press("g");

    await expect(page.locator('[data-testid="fillet-panel"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-testid="operation-confirm"]')).toBeVisible();
    await expect(page.locator('[data-testid="operation-cancel"]')).toBeVisible();
  });

  test("cancelling fillet does not add feature", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    const initialCount = await page.locator('[data-testid="feature-count"]').textContent();

    await page.locator("body").click();
    await page.keyboard.press("g");
    await expect(page.locator('[data-testid="fillet-panel"]')).toBeVisible({ timeout: 10000 });

    await page.locator('[data-testid="operation-cancel"]').click();

    // Feature count unchanged
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(initialCount!);
  });

  test("fillet with edge indices adds feature to tree", async ({ page }) => {
    await waitForEditor(page);
    await createBoxViaKernel(page);

    // Add fillet programmatically via kernel (simulating variable fillet with specific edge)
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

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("3 feature", { timeout: 10000 });
  });
});
