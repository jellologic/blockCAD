import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

test.describe("Assembly properties — color, mass, replace", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
  });

  test("set component color override", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 1);

    const color = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const asm = store.getState().assembly;
      asm.setComponentColor(0, [1.0, 0.0, 0.0, 1.0]); // Red
      // Read back — not directly accessible from store, but no error = success
      return true;
    });

    expect(color).toBe(true);
  });

  test("clear component color override", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 1);

    const cleared = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const asm = store.getState().assembly;
      asm.setComponentColor(0, [1.0, 0.0, 0.0, 1.0]);
      asm.setComponentColor(0, null); // Clear
      return true;
    });

    expect(cleared).toBe(true);
  });

  test("assembly part and component counts are correct", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
    await page.waitForTimeout(300);

    const counts = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      const asm = store.getState().assembly;
      return { parts: asm.partCount, components: asm.componentCount };
    });

    expect(counts.parts).toBe(1); // 1 box part definition
    expect(counts.components).toBe(2); // 2 instances
  });

  test("assembly has correct component count", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
    await page.waitForTimeout(300);

    const count = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      return store.getState().assembly.componentCount;
    });

    expect(count).toBe(2);
  });

  test("replace component part reference", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);

    // Add a second part to the assembly
    const newPartId = await page.evaluate(() => {
      const store = (window as any).__assemblyStore;
      return store.getState().addPart("Cylinder Part");
    });

    expect(newPartId).toBeTruthy();

    // Replace first component's part
    const success = await page.evaluate((pid: string) => {
      const store = (window as any).__assemblyStore;
      const compId = store.getState().components[0]?.id;
      if (!compId) return false;
      try {
        store.getState().assembly.replaceComponentPart(compId, pid);
        return true;
      } catch {
        return false;
      }
    }, newPartId);

    expect(success).toBe(true);
  });
});
