import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

test.describe("Assembly mate visualization", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
  });

  test("hovering a mate row in the tree shows visualization data-testid", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    // Programmatically add a Coincident mate between the two components
    const mateId = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const state = store.getState();
      const comps = state.components;
      if (comps.length < 2) throw new Error("Need at least 2 components");
      state.addMate("Coincident", comps[0].id, comps[1].id, 0, 0);
      return store.getState().mates[0]?.id;
    });

    expect(mateId).toBeTruthy();

    // The mate row should now exist in the tree
    const mateRow = page.locator(`[data-testid="assembly-mate-row-${mateId}"]`);
    await expect(mateRow).toBeVisible({ timeout: 5000 });

    // Hover over the mate row
    await mateRow.hover();

    // Verify the hoveredMateId is set in the store
    const hoveredId = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      return store.getState().hoveredMateId;
    });
    expect(hoveredId).toBe(mateId);
  });

  test("un-hovering a mate row clears hoveredMateId", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    // Add a mate
    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const state = store.getState();
      const comps = state.components;
      state.addMate("Coincident", comps[0].id, comps[1].id, 0, 0);
    });

    const mateId = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      return store.getState().mates[0]?.id;
    });

    const mateRow = page.locator(`[data-testid="assembly-mate-row-${mateId}"]`);
    await expect(mateRow).toBeVisible({ timeout: 5000 });

    // Hover and then move away
    await mateRow.hover();
    // Move mouse to the header area to un-hover
    await page.locator('[data-testid="assembly-component-count"]').hover();

    const hoveredId = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      return store.getState().hoveredMateId;
    });
    expect(hoveredId).toBeNull();
  });

  test("hovering a mate highlights connected component rows", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    // Add a mate
    const compIds = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const state = store.getState();
      const comps = state.components;
      state.addMate("Coincident", comps[0].id, comps[1].id, 0, 0);
      return [comps[0].id, comps[1].id];
    });

    const mateId = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      return store.getState().mates[0]?.id;
    });

    const mateRow = page.locator(`[data-testid="assembly-mate-row-${mateId}"]`);
    await expect(mateRow).toBeVisible({ timeout: 5000 });

    // Hover over the mate
    await mateRow.hover();

    // Both component rows should have the highlight class
    for (const compId of compIds) {
      const compRow = page.locator(`[data-testid="assembly-component-row-${compId}"]`);
      await expect(compRow).toBeVisible();
      // The row should have the yellow highlight background
      await expect(compRow).toHaveClass(/bg-yellow/);
    }
  });
});
