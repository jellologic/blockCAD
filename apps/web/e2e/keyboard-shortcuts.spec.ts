import { test, expect } from "@playwright/test";
import { waitForEditor } from "./helpers";

test.describe("Keyboard shortcuts", () => {
  test("E opens extrude PropertyManager", async ({ page }) => {
    await waitForEditor(page);
    // Focus the body to ensure keyboard events work
    await page.locator("body").click();
    await page.keyboard.press("e");
    await expect(
      page.locator('[data-testid="extrude-depth"]')
    ).toBeVisible({ timeout: 10000 });
  });

  test("Escape cancels extrude", async ({ page }) => {
    await waitForEditor(page);
    await page.locator("body").click();
    await page.keyboard.press("e");
    await expect(
      page.locator('[data-testid="extrude-depth"]')
    ).toBeVisible({ timeout: 10000 });
    await page.keyboard.press("Escape");
    await expect(
      page.locator('[data-testid="extrude-depth"]')
    ).not.toBeVisible();
  });

  test("S enters sketch mode on Front plane", async ({ page }) => {
    await waitForEditor(page);
    await page.locator("body").click();
    await page.keyboard.press("s");
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });
  });

  test("Escape exits sketch mode", async ({ page }) => {
    await waitForEditor(page);
    await page.locator("body").click();
    await page.keyboard.press("s");
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });

    // Escape exits sketch
    await page.keyboard.press("Escape");
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).not.toBeVisible();
  });

  test("Enter confirms sketch", async ({ page }) => {
    await waitForEditor(page);
    await page.locator("body").click();
    await page.keyboard.press("s");
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });

    await page.keyboard.press("Enter");
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).not.toBeVisible();

    // Feature should have been added
    await expect(page.locator('[data-testid="feature-count"]')).toContainText(
      "3 features"
    );
  });
});

test.describe("Sketch mode keyboard shortcuts", () => {
  test("T activates trim tool in sketch mode", async ({ page }) => {
    await waitForEditor(page);
    await page.locator("body").click();
    await page.keyboard.press("s");
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });

    await page.keyboard.press("t");
    await expect(page.locator('[data-testid="tool-trim"]')).toBeVisible();
  });

  test("P activates polygon tool in sketch mode", async ({ page }) => {
    await waitForEditor(page);
    await page.locator("body").click();
    await page.keyboard.press("s");
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });

    await page.keyboard.press("p");
    await expect(page.locator('[data-testid="tool-polygon"]')).toBeVisible();
  });

  test("O activates offset tool in sketch mode", async ({ page }) => {
    await waitForEditor(page);
    await page.locator("body").click();
    await page.keyboard.press("s");
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });

    await page.keyboard.press("o");
    await expect(page.locator('[data-testid="tool-offset"]')).toBeVisible();
  });

  test("F activates fillet in sketch mode", async ({ page }) => {
    await waitForEditor(page);
    await page.locator("body").click();
    await page.keyboard.press("s");
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });

    await page.keyboard.press("f");
    await expect(page.locator('[data-testid="tool-sketch-fillet"]')).toBeVisible();
  });

  test("H activates chamfer in sketch mode", async ({ page }) => {
    await waitForEditor(page);
    await page.locator("body").click();
    await page.keyboard.press("s");
    await expect(
      page.locator('[data-testid="sketch-confirm"]')
    ).toBeVisible({ timeout: 10000 });

    await page.keyboard.press("h");
    await expect(page.locator('[data-testid="tool-sketch-chamfer"]')).toBeVisible();
  });
});
