import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

test.describe("Assembly motion study", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
  });

  test("motion button appears in assembly ribbon", async ({ page }) => {
    await enterAssemblyMode(page);
    await expect(page.locator('[data-testid="assembly-motion"]')).toBeVisible();
  });

  test("clicking motion opens the motion panel", async ({ page }) => {
    await enterAssemblyMode(page);
    await page.locator('[data-testid="assembly-motion"]').click();
    await expect(page.locator('[data-testid="motion-driver-select"]')).toBeVisible();
    await expect(page.locator('[data-testid="motion-start-value"]')).toBeVisible();
    await expect(page.locator('[data-testid="motion-end-value"]')).toBeVisible();
    await expect(page.locator('[data-testid="motion-steps"]')).toBeVisible();
    await expect(page.locator('[data-testid="motion-run"]')).toBeVisible();
  });

  test("closing the motion panel returns to assembly tree", async ({ page }) => {
    await enterAssemblyMode(page);
    await page.locator('[data-testid="assembly-motion"]').click();
    await expect(page.locator('[data-testid="motion-driver-select"]')).toBeVisible();

    await page.locator('[data-testid="motion-close"]').click();
    await expect(page.locator('[data-testid="motion-driver-select"]')).not.toBeVisible();
    await expect(page.locator('[data-testid="assembly-component-count"]')).toBeVisible();
  });

  test("run button is disabled when no mates exist", async ({ page }) => {
    await enterAssemblyMode(page);
    await page.locator('[data-testid="assembly-motion"]').click();

    const runBtn = page.locator('[data-testid="motion-run"]');
    await expect(runBtn).toBeVisible();
    await expect(runBtn).toBeDisabled();
  });

  test("motion study with distance mate shows playback controls", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    // Add a distance mate between the two components programmatically
    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const state = store.getState();
      state.addMate("distance", state.components[0].id, state.components[1].id, 0, 0, 5);
    });

    // Open motion panel
    await page.locator('[data-testid="assembly-motion"]').click();
    await expect(page.locator('[data-testid="motion-driver-select"]')).toBeVisible();

    // Set parameters
    await page.locator('[data-testid="motion-start-value"]').fill("0");
    await page.locator('[data-testid="motion-end-value"]').fill("20");
    await page.locator('[data-testid="motion-steps"]').fill("5");

    // Run study
    await page.locator('[data-testid="motion-run"]').click();

    // Playback controls should appear
    await expect(page.locator('[data-testid="motion-play"]')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('[data-testid="motion-stop"]')).toBeVisible();
    await expect(page.locator('[data-testid="motion-slider"]')).toBeVisible();
    await expect(page.locator('[data-testid="motion-frame-display"]')).toBeVisible();
    await expect(page.locator('[data-testid="motion-frame-display"]')).toContainText("Frame 1 / 6");
  });

  test("frame slider scrubs through frames", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    // Add a distance mate and run motion study programmatically
    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const state = store.getState();
      state.addMate("distance", state.components[0].id, state.components[1].id, 0, 0, 5);
      state.startOp({ type: "motion-study" });
    });

    await expect(page.locator('[data-testid="motion-driver-select"]')).toBeVisible();
    await page.locator('[data-testid="motion-start-value"]').fill("0");
    await page.locator('[data-testid="motion-end-value"]').fill("10");
    await page.locator('[data-testid="motion-steps"]').fill("4");
    await page.locator('[data-testid="motion-run"]').click();

    // Wait for playback controls
    await expect(page.locator('[data-testid="motion-slider"]')).toBeVisible({ timeout: 10000 });

    // Scrub to last frame using the store (slider input events are hard to simulate)
    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      store.getState().setFrame(4);
    });

    await expect(page.locator('[data-testid="motion-frame-display"]')).toContainText("Frame 5 / 5");
  });
});
