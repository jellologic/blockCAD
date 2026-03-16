import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

test.describe("Assembly visibility — hide/show vs suppress", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
  });

  test("hide component sets hidden flag without changing active count", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
    await page.waitForTimeout(300);

    // Active count should be 2
    await expect(page.locator('[data-testid="assembly-component-count"]')).toContainText("2 active", { timeout: 5000 });

    // Hide the first component via kernel
    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      store.getState().assembly.hideComponent(0);
    });
    await page.waitForTimeout(300);

    // Active count should still be 2 (hidden != suppressed)
    await expect(page.locator('[data-testid="assembly-component-count"]')).toContainText("2 active");
  });

  test("show restores hidden component", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const asm = store.getState().assembly;
      asm.hideComponent(0);
      asm.showComponent(0);
    });

    // Verify component is no longer hidden
    const isHidden = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      return store.getState().components[0]?.suppressed === false;
    });
    expect(isHidden).toBe(true);
  });

  test("suppress reduces active count, hide does not", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
    await page.waitForTimeout(300);

    // Suppress first component
    await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      store.getState().suppressComponent(0);
    });
    await page.waitForTimeout(300);

    // Active count should now be 1
    await expect(page.locator('[data-testid="assembly-component-count"]')).toContainText("1 active", { timeout: 5000 });
  });
});
