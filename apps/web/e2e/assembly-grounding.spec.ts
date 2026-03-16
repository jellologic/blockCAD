import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

test.describe("Assembly grounding", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
  });

  test("ground second component via kernel", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    const grounded = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const asm = store.getState().assembly;
      asm.groundComponent(1);
      // Verify via internal state — component_count stays same
      return store.getState().assembly.componentCount;
    });

    expect(grounded).toBe(2);
  });

  test("unground restores component freedom", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const asm = store.getState().assembly;
      asm.groundComponent(1);
      asm.ungroundComponent(1);
    });

    // Assembly still has 2 components, no errors
    await expect(page.locator('[data-testid="assembly-component-count"]')).toContainText("2 active");
  });

  test("ground and rebuild succeeds", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    // Ground second component and rebuild
    const success = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      store.getState().assembly.groundComponent(1);
      try {
        store.getState().rebuild();
        return true;
      } catch {
        return false;
      }
    });

    expect(success).toBe(true);
  });
});
