import { test, expect } from "@playwright/test";
import { enterAssemblyMode } from "./helpers";

test.describe("Assembly sub-assemblies", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
  });

  test("create a sub-assembly and insert components into it", async ({ page }) => {
    await enterAssemblyMode(page);

    // Create a box part
    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const state = store.getState();
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

      // Create a sub-assembly and insert 2 components into it
      const subIdx = state.insertSubAssembly("Bracket Group");
      state.insertComponent(partId, "Bracket A", [0, 0, 0], subIdx);
      state.insertComponent(partId, "Bracket B", [15, 0, 0], subIdx);

      // Insert 1 top-level component
      state.insertComponent(partId, "Standalone Part", [30, 0, 0]);
    });

    await page.waitForTimeout(500);

    // Verify component count: 3 total (2 in sub-assembly + 1 top-level)
    await expect(page.locator('[data-testid="assembly-component-count"]')).toContainText("3 active", { timeout: 5000 });

    // Sub-assembly node should be visible
    await expect(page.locator('[data-testid="sub-assembly-0"]')).toBeVisible();

    // Sub-assembly should show its nested components
    await expect(page.locator('[data-testid="sub-assembly-0"]')).toContainText("Bracket A");
    await expect(page.locator('[data-testid="sub-assembly-0"]')).toContainText("Bracket B");

    // Top-level component should be visible outside the sub-assembly
    await expect(page.locator("text=Standalone Part")).toBeVisible();
  });

  test("collapse sub-assembly hides nested components", async ({ page }) => {
    await enterAssemblyMode(page);

    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const state = store.getState();
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

      const subIdx = state.insertSubAssembly("Motor Assembly");
      state.insertComponent(partId, "Motor Mount", [0, 0, 0], subIdx);
    });

    await page.waitForTimeout(500);

    // Initially expanded: component is visible
    await expect(page.locator('[data-testid="sub-assembly-0"]')).toContainText("Motor Mount");

    // Collapse the sub-assembly
    await page.locator('[data-testid="sub-assembly-toggle-0"]').click();

    // After collapse the component row should be hidden
    const componentRow = page.locator('[data-testid="sub-assembly-0"] [data-testid^="component-row-"]');
    await expect(componentRow).not.toBeVisible();
  });
});
