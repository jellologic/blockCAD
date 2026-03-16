import { test, expect } from "@playwright/test";
import fs from "fs";

/**
 * Wait for the editor to load (kernel initialized), then inject
 * a sketch + extrude programmatically so we have geometry to export.
 */
async function setupEditorWithGeometry(page: import("@playwright/test").Page) {
  await page.goto("/editor");

  // Wait for kernel to initialize
  await page.waitForFunction(() => {
    const store = (window as any).__editorStore;
    return store && store.getState().kernel && !store.getState().isLoading;
  }, { timeout: 20000 });

  // Inject geometry via the store: add sketch + extrude programmatically
  await page.evaluate(() => {
    const store = (window as any).__editorStore;
    const state = store.getState();

    const kernel = state.kernel;

    // Add a sketch with a 10×5 rectangle
    kernel.addFeature("sketch", "Sketch 1", {
      type: "sketch",
      params: {
        plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
        entities: [
          { type: "point", id: "se-0", position: { x: 0, y: 0 } },
          { type: "point", id: "se-1", position: { x: 10, y: 0 } },
          { type: "point", id: "se-2", position: { x: 10, y: 5 } },
          { type: "point", id: "se-3", position: { x: 0, y: 5 } },
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
          { id: "sc-6", kind: "distance", entityIds: ["se-1", "se-2"], value: 5 },
        ],
      },
    });

    // Add extrude
    kernel.addFeature("extrude", "Extrude 1", {
      type: "extrude",
      params: {
        direction: [0, 0, 1], depth: 7, symmetric: false, draft_angle: 0,
        end_condition: "blind", direction2_enabled: false, depth2: 0,
        draft_angle2: 0, end_condition2: "blind", from_offset: 0,
        thin_feature: false, thin_wall_thickness: 0,
        flip_side_to_cut: false, cap_ends: false, from_condition: "sketch_plane",
      },
    });

    // Tessellate and update store
    state.rebuild();
  });

  // Wait for mesh to appear
  await expect(page.locator('[data-testid="vertex-count"]')).toContainText("Verts:", { timeout: 10000 });
}

test.describe("Export workflow", () => {
  test("export STL triggers download with valid binary", async ({ page }) => {
    await setupEditorWithGeometry(page);
    await page.click('[data-testid="tab-view"]');

    const exportBtn = page.locator('[data-testid="export-stl"]');
    await expect(exportBtn).toBeVisible();

    const [download] = await Promise.all([
      page.waitForEvent("download"),
      exportBtn.click(),
    ]);

    expect(download.suggestedFilename()).toBe("model.stl");
    const path = await download.path();
    expect(path).toBeTruthy();
    const bytes = fs.readFileSync(path!);
    expect(bytes.length).toBeGreaterThan(84);
    const triCount = bytes.readUInt32LE(80);
    expect(triCount).toBeGreaterThan(0);
    expect(bytes.length).toBe(84 + 50 * triCount);
  });

  test("export OBJ triggers download with valid OBJ", async ({ page }) => {
    await setupEditorWithGeometry(page);
    await page.click('[data-testid="tab-view"]');

    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.locator('[data-testid="export-obj"]').click(),
    ]);

    expect(download.suggestedFilename()).toBe("model.obj");
    const path = await download.path();
    const content = fs.readFileSync(path!, "utf-8");
    expect(content).toContain("# blockCAD OBJ export");
    expect(content).toContain("v ");
    expect(content).toContain("f ");
  });

  test("export 3MF triggers download with valid ZIP", async ({ page }) => {
    await setupEditorWithGeometry(page);
    await page.click('[data-testid="tab-view"]');

    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.locator('[data-testid="export-3mf"]').click(),
    ]);

    expect(download.suggestedFilename()).toBe("model.3mf");
    const path = await download.path();
    const bytes = fs.readFileSync(path!);
    expect(bytes[0]).toBe(0x50); // P
    expect(bytes[1]).toBe(0x4B); // K
  });

  test("export GLB triggers download with valid GLB", async ({ page }) => {
    await setupEditorWithGeometry(page);
    await page.click('[data-testid="tab-view"]');

    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page.locator('[data-testid="export-glb"]').click(),
    ]);

    expect(download.suggestedFilename()).toBe("model.glb");
    const path = await download.path();
    const bytes = fs.readFileSync(path!);
    expect(bytes.readUInt32LE(0)).toBe(0x46546C67); // glTF
    expect(bytes.readUInt32LE(4)).toBe(2); // version
  });

  test("toast shows success after export", async ({ page }) => {
    await setupEditorWithGeometry(page);
    await page.click('[data-testid="tab-view"]');

    await Promise.all([
      page.waitForEvent("download"),
      page.locator('[data-testid="export-stl"]').click(),
    ]);

    await expect(page.locator("text=STL exported")).toBeVisible({ timeout: 5000 });
  });
});
