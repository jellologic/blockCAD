import { test, expect } from "@playwright/test";
import { waitForEditor } from "./helpers";

/**
 * Helper: create a box via kernel.
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

test.describe("Advanced features — shell, chamfer, fillet", () => {
  test("shell operation opens PropertyManager", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().startOperation("shell");
    });

    await expect(page.locator('[data-testid="shell-panel"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-testid="shell-thickness"]')).toBeVisible();
    await expect(page.locator('[data-testid="operation-confirm"]')).toBeVisible();
  });

  test("shell thickness input accepts value", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().startOperation("shell");
    });

    await expect(page.locator('[data-testid="shell-thickness"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-testid="shell-thickness"]').fill("2");

    const val = await page.locator('[data-testid="shell-thickness"]').inputValue();
    expect(val).toBe("2");
  });

  test("cancel shell does not add feature", async ({ page }) => {
    await createBoxViaKernel(page);

    const initialCount = await page.locator('[data-testid="feature-count"]').textContent();

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().startOperation("shell");
    });

    await expect(page.locator('[data-testid="shell-panel"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-testid="operation-cancel"]').click();

    await expect(page.locator('[data-testid="feature-count"]')).toContainText(initialCount!);
  });

  test("shell feature added via kernel shows in tree", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("shell", "Shell 1", {
        type: "shell",
        params: {
          faces_to_remove: [5],
          thickness: 1,
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("3 feature", { timeout: 10000 });
    await expect(page.getByText("Shell 1")).toBeVisible();
  });

  test("chamfer operation opens PropertyManager", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().startOperation("chamfer");
    });

    await expect(page.locator('[data-testid="chamfer-panel"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-testid="chamfer-distance"]')).toBeVisible();
  });

  test("chamfer distance input accepts value", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().startOperation("chamfer");
    });

    await expect(page.locator('[data-testid="chamfer-distance"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-testid="chamfer-distance"]').fill("2");

    const val = await page.locator('[data-testid="chamfer-distance"]').inputValue();
    expect(val).toBe("2");
  });

  test("chamfer asymmetric checkbox toggles second distance", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().startOperation("chamfer");
    });

    await expect(page.locator('[data-testid="chamfer-panel"]')).toBeVisible({ timeout: 10000 });

    // Check asymmetric to reveal second distance input
    await page.locator('[data-testid="chamfer-asymmetric"]').check();

    await expect(page.locator('[data-testid="chamfer-distance2"]')).toBeVisible();
  });

  test("chamfer feature added via kernel shows in tree", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("chamfer", "Chamfer 1", {
        type: "chamfer",
        params: {
          edge_indices: [0, 1, 2, 3],
          distance: 1,
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("3 feature", { timeout: 10000 });
    await expect(page.getByText("Chamfer 1")).toBeVisible();
  });

  test("fillet feature added via kernel shows in tree", async ({ page }) => {
    await createBoxViaKernel(page);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("fillet", "Fillet 1", {
        type: "fillet",
        params: {
          edge_indices: [0, 1, 2, 3],
          radius: 2,
        },
      });

      store.getState().rebuild();
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText("3 feature", { timeout: 10000 });
    await expect(page.getByText("Fillet 1")).toBeVisible();
  });

  test("shell increases vertex count from hollowing", async ({ page }) => {
    await createBoxViaKernel(page);

    const initialVertText = await page.locator('[data-testid="vertex-count"]').textContent();
    const initialCount = parseInt(initialVertText!.replace("Verts: ", ""), 10);

    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      const kernel = store.getState().kernel;

      kernel.addFeature("shell", "Shell 1", {
        type: "shell",
        params: {
          faces_to_remove: [5],
          thickness: 1,
        },
      });

      store.getState().rebuild();
    });

    await page.waitForTimeout(500);

    const newVertText = await page.locator('[data-testid="vertex-count"]').textContent();
    const newCount = parseInt(newVertText!.replace("Verts: ", ""), 10);

    // Shell produces more vertices (inner + outer walls)
    expect(newCount).toBeGreaterThan(initialCount);
  });
});
