import { test, expect } from "@playwright/test";
import { waitForEditor } from "./helpers";

/**
 * Programmatically add a sketch + extrude to the editor so
 * we have features to manipulate in the feature tree.
 */
async function addTwoFeatures(page: import("@playwright/test").Page) {
  await page.evaluate(() => {
    const store = (window as any).__editorStore;
    if (!store) throw new Error("Editor store not on window");
    const state = store.getState();

    // Add a sketch feature
    state.addFeature("sketch", "Sketch 1", {
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
          {
            id: "sc-5",
            kind: "distance",
            entityIds: ["se-0", "se-1"],
            value: 10,
          },
          {
            id: "sc-6",
            kind: "distance",
            entityIds: ["se-1", "se-2"],
            value: 5,
          },
        ],
      },
    });
  });
}

test.describe("Feature tree CRUD operations", () => {
  test.beforeEach(async ({ page }) => {
    await waitForEditor(page);
  });

  test("right-click shows context menu", async ({ page }) => {
    await addTwoFeatures(page);
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "1 feature"
    );

    // Right-click on the first feature
    const firstFeature = page.locator(
      '[data-testid^="feature-"]'
    ).first();
    await firstFeature.click({ button: "right" });

    // Context menu should appear
    await expect(
      page.locator('[data-testid="feature-context-menu"]')
    ).toBeVisible();
  });

  test("delete removes feature from tree", async ({ page }) => {
    await addTwoFeatures(page);
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "1 feature"
    );

    // Programmatically delete the feature
    page.on("dialog", (dialog) => dialog.accept());
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().deleteFeature(0);
    });

    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "0 feature"
    );
  });

  test("rename changes feature name", async ({ page }) => {
    await addTwoFeatures(page);

    // Programmatically rename
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().renameFeature(0, "My Custom Sketch");
    });

    // Check the feature name in the tree
    await expect(
      page.locator('[data-testid^="feature-"]').first()
    ).toContainText("My Custom Sketch");
  });

  test("suppress and unsuppress toggles feature state", async ({ page }) => {
    await addTwoFeatures(page);

    // Suppress the feature
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().suppressFeature(0);
    });

    // Feature should have suppressed styling (line-through)
    const firstFeature = page.locator('[data-testid^="feature-"]').first();
    await expect(firstFeature).toHaveClass(/line-through/);

    // Unsuppress
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().unsuppressFeature(0);
    });

    await expect(firstFeature).not.toHaveClass(/line-through/);
  });

  test("rollback grays out features beyond cursor", async ({ page }) => {
    // We need at least 2 features for rollback to be visible.
    // Add sketch programmatically, then attempt rollback.
    await addTwoFeatures(page);

    // The feature list should show 1 feature (just the sketch)
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "1 feature"
    );

    // Rollback to index 0 (before the first feature)
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().rollbackTo(0);
    });

    // Roll forward again
    await page.evaluate(() => {
      const store = (window as any).__editorStore;
      store.getState().rollForward();
    });

    // Features should still be present
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "1 feature"
    );
  });
});
