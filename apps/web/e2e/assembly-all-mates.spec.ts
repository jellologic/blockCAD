import { test, expect } from "@playwright/test";
import { enterAssemblyMode, setupAssemblyWithBoxes } from "./helpers";

const ALL_MATE_TYPES = [
  "Coincident",
  "Distance",
  "Angle",
  "Concentric",
  "Parallel",
  "Perpendicular",
  "Tangent",
  "Lock",
  "Hinge",
  "Gear",
  "Screw",
  "Limit",
  "Rack Pinion",
  "Cam",
  "Universal Joint",
  "Width",
  "Symmetric",
  "Slot",
];

/** Mate types that should show parameter inputs, mapped to their expected input test IDs */
const PARAMETERIZED_MATES: Record<string, string[]> = {
  distance: ["mate-param-value"],
  angle: ["mate-param-value"],
  gear: ["mate-param-ratio"],
  screw: ["mate-param-pitch"],
  limit: ["mate-param-min", "mate-param-max"],
  rack_pinion: ["mate-param-pitch_radius"],
  cam: ["mate-param-eccentricity", "mate-param-base_radius"],
  slot: ["mate-param-axis_x", "mate-param-axis_y", "mate-param-axis_z"],
};

/** Mate types that should NOT show parameter inputs */
const NO_PARAM_MATES = [
  "coincident",
  "concentric",
  "parallel",
  "perpendicular",
  "tangent",
  "lock",
  "hinge",
  "universal_joint",
  "width",
  "symmetric",
];

test.describe("Assembly all mate types", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/editor");
    await page.waitForTimeout(2000);
  });

  test("dropdown shows all 18 mate types", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
    await page.locator('[data-testid="assembly-mate"]').click();
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });

    const options = page.locator('[data-testid="mate-type-select"] option');
    const count = await options.count();
    expect(count).toBeGreaterThanOrEqual(18);

    // Verify each expected mate type label exists
    const optionTexts: string[] = [];
    for (let i = 0; i < count; i++) {
      const text = await options.nth(i).textContent();
      if (text) optionTexts.push(text);
    }

    for (const label of ALL_MATE_TYPES) {
      expect(optionTexts).toContain(label);
    }
  });

  test("dropdown has category groups", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
    await page.locator('[data-testid="assembly-mate"]').click();
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });

    const optgroups = page.locator('[data-testid="mate-type-select"] optgroup');
    const groupCount = await optgroups.count();
    expect(groupCount).toBe(3);

    const labels: string[] = [];
    for (let i = 0; i < groupCount; i++) {
      const label = await optgroups.nth(i).getAttribute("label");
      if (label) labels.push(label);
    }
    expect(labels).toContain("Standard");
    expect(labels).toContain("Mechanical");
    expect(labels).toContain("Advanced");
  });

  test("parameterized types show correct input fields", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
    await page.locator('[data-testid="assembly-mate"]').click();
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });

    const select = page.locator('[data-testid="mate-type-select"]');

    for (const [mateValue, expectedInputs] of Object.entries(PARAMETERIZED_MATES)) {
      await select.selectOption(mateValue);

      // Parameters section should be visible
      await expect(page.locator('[data-testid="mate-params-section"]')).toBeVisible();

      // Each expected input should be present
      for (const testId of expectedInputs) {
        await expect(page.locator(`[data-testid="${testId}"]`)).toBeVisible();
      }
    }
  });

  test("non-parameterized types show no parameter inputs", async ({ page }) => {
    await enterAssemblyMode(page);
    await setupAssemblyWithBoxes(page, 2);
    await page.locator('[data-testid="assembly-mate"]').click();
    await expect(page.locator('[data-testid="mate-type-select"]')).toBeVisible({ timeout: 5000 });

    const select = page.locator('[data-testid="mate-type-select"]');

    for (const mateValue of NO_PARAM_MATES) {
      await select.selectOption(mateValue);

      // Parameters section should NOT be visible
      await expect(page.locator('[data-testid="mate-params-section"]')).not.toBeVisible();
    }
  });
});
