import { type Page, expect } from "@playwright/test";

/** Navigate to the editor and wait for the kernel to fully load */
export async function waitForEditor(page: Page) {
  await page.goto("/editor");
  // Wait for hydration + kernel init — feature count will go from "0" to "2"
  await expect(page.locator('[data-testid="feature-count"]')).toContainText(
    "2 features",
    { timeout: 20000 }
  );
}

/** Enter sketch mode on the given plane */
export async function enterSketchMode(
  page: Page,
  plane: "front" | "top" | "right" = "front"
) {
  await page.locator('[data-testid="ribbon-sketch"]').click();
  await page.locator(`[data-testid="plane-${plane}"]`).click();
  await expect(
    page.locator('[data-testid="sketch-confirm"]')
  ).toBeVisible({ timeout: 10000 });
}

/** Confirm the current sketch */
export async function confirmSketch(page: Page) {
  await page.locator('[data-testid="sketch-confirm"]').click();
  await expect(
    page.locator('[data-testid="sketch-confirm"]')
  ).not.toBeVisible();
}

/** Cancel the current sketch */
export async function cancelSketch(page: Page) {
  await page.locator('[data-testid="sketch-cancel"]').click();
  await expect(
    page.locator('[data-testid="sketch-cancel"]')
  ).not.toBeVisible();
}

/** Start an extrude operation */
export async function startExtrude(page: Page) {
  await page.locator('[data-testid="ribbon-extrude"]').click();
  await expect(
    page.locator('[data-testid="extrude-depth"]')
  ).toBeVisible({ timeout: 10000 });
}

/** Confirm the current operation (extrude, revolve, etc.) */
export async function confirmOperation(page: Page) {
  await page.locator('[data-testid="operation-confirm"]').click();
}
