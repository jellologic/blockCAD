import { type Page, expect } from "@playwright/test";

/** Navigate to the editor and wait for the kernel to fully load */
export async function waitForEditor(page: Page) {
  await page.goto("/editor");
  // Wait for hydration + kernel init — feature count element becomes visible
  await expect(page.locator('[data-testid="feature-count"]')).toBeVisible({
    timeout: 20000,
  });
  // Wait for kernel to be initialized
  await page.waitForFunction(() => {
    const store = (window as any).__editorStore;
    return store && store.getState().kernel !== null;
  }, { timeout: 20000 });
}

/** Enter sketch mode by clicking the Sketch ribbon button, then clicking the 3D plane.
 *  Uses fixed viewport (1280x720) so the plane positions are predictable.
 *  The front plane is at the center of the viewport in the default isometric camera. */
export async function enterSketchMode(
  page: Page,
  plane: "front" | "top" | "right" = "front"
) {
  // Click "Sketch" ribbon button to enter plane-selection mode
  await page.locator('[data-testid="ribbon-sketch"]').click();

  // Wait for the mode to switch to select-plane (planes become more opaque)
  await page.waitForTimeout(300);

  // Click on the canvas to select a reference plane.
  // In the default isometric camera [20,15,20], the planes are visible near the center.
  // The canvas sits inside the viewport div, taking up most of the right side.
  const canvas = page.locator("canvas");
  const box = await canvas.boundingBox();
  if (!box) throw new Error("Canvas not found");

  // Approximate click positions for each plane in the default isometric view:
  // Front plane (z=0, blue) — center-right of canvas
  // Top plane (y=0, green) — center-bottom of canvas
  // Right plane (x=0, red) — center-left of canvas
  const clickPositions: Record<string, { x: number; y: number }> = {
    front: { x: box.x + box.width * 0.55, y: box.y + box.height * 0.45 },
    top:   { x: box.x + box.width * 0.45, y: box.y + box.height * 0.55 },
    right: { x: box.x + box.width * 0.42, y: box.y + box.height * 0.40 },
  };

  const pos = clickPositions[plane];
  await page.mouse.click(pos.x, pos.y);

  // Wait for sketch mode to activate (confirm button appears)
  await expect(
    page.locator('[data-testid="sketch-confirm"]')
  ).toBeVisible({ timeout: 10000 });
}

/** Confirm the current sketch */
export async function confirmSketch(page: Page) {
  await page.locator('[data-testid="sketch-confirm"]').click();
  await expect(
    page.locator('[data-testid="sketch-confirm"]')
  ).not.toBeVisible();
}

/** Cancel the current sketch */
export async function cancelSketch(page: Page) {
  await page.locator('[data-testid="sketch-cancel"]').click();
  await expect(
    page.locator('[data-testid="sketch-cancel"]')
  ).not.toBeVisible();
}

/** Start an extrude operation */
export async function startExtrude(page: Page) {
  await page.locator('[data-testid="ribbon-extrude"]').click();
  await expect(
    page.locator('[data-testid="extrude-depth"]')
  ).toBeVisible({ timeout: 10000 });
}

/** Confirm the current operation (extrude, revolve, etc.) */
export async function confirmOperation(page: Page) {
  await page.locator('[data-testid="operation-confirm"]').click();
}

/** Switch to the View tab where export buttons live */
export async function navigateToViewTab(page: Page) {
  await page.click('[data-testid="tab-view"]');
  await expect(page.locator('[data-testid="export-stl"]')).toBeVisible();
}

/** Enter assembly mode programmatically via store */
export async function enterAssemblyMode(page: Page) {
  await page.evaluate(async () => {
    const store = (window as any).__assemblyStore;
    if (!store) throw new Error("Assembly store not on window");
    await store.getState().initAssembly();
  });
  // Wait for assembly tree panel to appear
  await expect(page.locator('[data-testid="assembly-component-count"]')).toBeVisible({ timeout: 10000 });
}

/** Programmatically create a box part + N components in assembly mode */
export async function setupAssemblyWithBoxes(page: Page, count: number = 2) {
  await page.evaluate((n) => {
    const store = (window as any).__assemblyStore;
    if (!store) throw new Error("Assembly store not on window");
    const state = store.getState();
    if (!state.assembly) throw new Error("Assembly not initialized");

    // Add a box part with sketch + extrude
    const partId = state.addPart("Box Part");
    state.addFeatureToPart(partId, "sketch", {
      type: "sketch",
      params: {
        plane: { origin: [0,0,0], normal: [0,0,1], uAxis: [1,0,0], vAxis: [0,1,0] },
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
    state.addFeatureToPart(partId, "extrude", {
      type: "extrude",
      params: {
        direction: [0,0,1], depth: 7, symmetric: false, draft_angle: 0,
        end_condition: "blind", direction2_enabled: false, depth2: 0,
        draft_angle2: 0, end_condition2: "blind", from_offset: 0,
        thin_feature: false, thin_wall_thickness: 0,
        flip_side_to_cut: false, cap_ends: false, from_condition: "sketch_plane",
      },
    });

    // Insert N components
    for (let i = 0; i < n; i++) {
      state.insertComponent(partId, `Box ${i + 1}`, [i * 15, 0, 0]);
    }
  }, count);
}
