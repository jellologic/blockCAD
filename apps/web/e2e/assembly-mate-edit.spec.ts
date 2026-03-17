import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

/**
 * Helper: add a mate via the store so we can then test edit/delete on it.
 */
async function addMateViaStore(page: import("@playwright/test").Page) {
  await page.evaluate(() => {
    const store = (window as any).__assemblyStore;
    if (!store) throw new Error("Assembly store not on window");
    const state = store.getState();
    const comps = state.components;
    if (comps.length < 2) throw new Error("Need at least 2 components");
    state.addMate("coincident", comps[0].id, comps[1].id, 0, 1);
  });
}

/** Helper: get the current mates array from the store */
async function getMates(page: import("@playwright/test").Page) {
  return page.evaluate(() => {
    const store = (window as any).__assemblyStore;
    return store.getState().mates;
  });
}

test.describe("Assembly mate editing and deletion", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
  });

  test("edit mate button opens panel in edit mode with pre-populated values", async ({ page }) => {
    await addMateViaStore(page);
    const mates = await getMates(page);
    expect(mates.length).toBe(1);
    const mateId = mates[0].id;

    // Click the edit button on the mate entry
    await page.locator(`[data-testid="mate-edit-${mateId}"]`).click({ force: true });

    // Mate panel should open in edit mode
    await expect(page.locator('[data-testid="mate-panel-title"]')).toHaveText("Edit Mate", { timeout: 5000 });
    await expect(page.locator('[data-testid="mate-type-select"]')).toHaveValue("coincident");
  });

  test("editing a mate value updates the store", async ({ page }) => {
    // Add a distance mate so we can edit the value
    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const state = store.getState();
      const comps = state.components;
      state.addMate("distance", comps[0].id, comps[1].id, 0, 1, 10);
    });
    const mates = await getMates(page);
    expect(mates.length).toBe(1);
    expect(mates[0].kind).toBe("distance");
    const mateId = mates[0].id;

    // Open edit panel
    await page.locator(`[data-testid="mate-edit-${mateId}"]`).click({ force: true });
    await expect(page.locator('[data-testid="mate-panel-title"]')).toHaveText("Edit Mate", { timeout: 5000 });

    // Change kind to angle
    await page.locator('[data-testid="mate-type-select"]').selectOption("angle");

    // Confirm
    await page.locator('[data-testid="mate-confirm"]').click();

    // Verify the mate was updated in the store
    const updatedMates = await getMates(page);
    expect(updatedMates.length).toBe(1);
    expect(updatedMates[0].kind).toBe("angle");
    expect(updatedMates[0].id).toBe(mateId);
  });

  test("deleting a mate removes it from the store", async ({ page }) => {
    await addMateViaStore(page);
    const mates = await getMates(page);
    expect(mates.length).toBe(1);
    const mateId = mates[0].id;

    // Set up dialog handler to accept the confirmation
    page.on("dialog", (dialog) => dialog.accept());

    // Click delete button
    await page.locator(`[data-testid="mate-delete-${mateId}"]`).click({ force: true });

    // Verify mate is removed
    const remaining = await getMates(page);
    expect(remaining.length).toBe(0);
  });

  test("cancel edit preserves original mate", async ({ page }) => {
    await addMateViaStore(page);
    const mates = await getMates(page);
    const mateId = mates[0].id;

    // Open edit panel
    await page.locator(`[data-testid="mate-edit-${mateId}"]`).click({ force: true });
    await expect(page.locator('[data-testid="mate-panel-title"]')).toHaveText("Edit Mate", { timeout: 5000 });

    // Change the kind
    await page.locator('[data-testid="mate-type-select"]').selectOption("parallel");

    // Cancel instead of confirming
    await page.locator('[data-testid="mate-cancel"]').click();

    // Verify the original mate is unchanged
    const unchanged = await getMates(page);
    expect(unchanged.length).toBe(1);
    expect(unchanged[0].kind).toBe("coincident");
    expect(unchanged[0].id).toBe(mateId);
  });
});
