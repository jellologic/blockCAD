import { type Page, expect } from "@playwright/test";
import { waitForEditor, enterSketchMode, confirmSketch } from "./helpers";

// ─── Coordinate Bridge ────────────────────────────────────────────

/**
 * Known camera positions when in sketch mode (from SketchCameraController).
 * Camera looks at origin [0,0,0] with fov=50.
 */
const SKETCH_CAMERA_POSITIONS: Record<string, [number, number, number]> = {
  front: [0, 0, 30],
  top: [0, 30, 0],
  right: [30, 0, 0],
};

/**
 * Convert a 2D sketch-plane coordinate to screen pixel position.
 *
 * Instead of accessing R3F internals (which aren't exposed), we compute
 * the projection analytically using the known camera position, FOV, and
 * sketch plane orientation.
 */
export async function sketchCoordToScreenPixel(
  page: Page,
  sketchX: number,
  sketchY: number
): Promise<{ px: number; py: number }> {
  const result = await page.evaluate(
    ({ sx, sy }) => {
      const editorStore = (window as any).__editorStore;
      if (!editorStore) throw new Error("Editor store not available");
      const state = editorStore.getState();
      const session = state.sketchSession;
      if (!session) throw new Error("Not in sketch mode");
      const plane = session.plane;
      const planeId = session.planeId;

      // Known camera targets for each plane
      const camPositions: Record<string, [number, number, number]> = {
        front: [0, 0, 30],
        top: [0, 30, 0],
        right: [30, 0, 0],
      };
      const camPos = camPositions[planeId] || camPositions.front;

      // sketchToWorld
      const worldX = plane.origin[0] + sx * plane.uAxis[0] + sy * plane.vAxis[0];
      const worldY = plane.origin[1] + sx * plane.uAxis[1] + sy * plane.vAxis[1];
      const worldZ = plane.origin[2] + sx * plane.uAxis[2] + sy * plane.vAxis[2];

      // Camera parameters
      const fov = 50; // degrees (from cad-viewport.tsx Canvas config)
      const near = 0.1;
      const canvas = document.querySelector("canvas");
      if (!canvas) throw new Error("Canvas not found");
      const rect = canvas.getBoundingClientRect();
      const aspect = rect.width / rect.height;

      // View matrix: camera at camPos, looking at origin, up = [0,1,0]
      // For front plane: camera at [0,0,30], looking at [0,0,0]
      // View space: x=right, y=up, z=toward camera
      const dx = worldX - camPos[0];
      const dy = worldY - camPos[1];
      const dz = worldZ - camPos[2];

      // Camera forward direction (normalized)
      const dist = Math.sqrt(camPos[0] ** 2 + camPos[1] ** 2 + camPos[2] ** 2);
      const fwd = [-camPos[0] / dist, -camPos[1] / dist, -camPos[2] / dist];

      // Camera right and up vectors
      // Up hint is [0,1,0] unless camera is looking straight down/up
      let upHint = [0, 1, 0];
      if (Math.abs(fwd[1]) > 0.99) {
        upHint = [0, 0, -1]; // Use z-axis as up hint for top/bottom views
      }

      // right = normalize(fwd × upHint)
      const rx = fwd[1] * upHint[2] - fwd[2] * upHint[1];
      const ry = fwd[2] * upHint[0] - fwd[0] * upHint[2];
      const rz = fwd[0] * upHint[1] - fwd[1] * upHint[0];
      const rLen = Math.sqrt(rx * rx + ry * ry + rz * rz);
      const right = [rx / rLen, ry / rLen, rz / rLen];

      // up = normalize(right × fwd)
      const ux = right[1] * fwd[2] - right[2] * fwd[1];
      const uy = right[2] * fwd[0] - right[0] * fwd[2];
      const uz = right[0] * fwd[1] - right[1] * fwd[0];
      const uLen = Math.sqrt(ux * ux + uy * uy + uz * uz);
      const up = [ux / uLen, uy / uLen, uz / uLen];

      // Project world point into view space
      const viewX = dx * right[0] + dy * right[1] + dz * right[2];
      const viewY = dx * up[0] + dy * up[1] + dz * up[2];
      const viewZ = -(dx * fwd[0] + dy * fwd[1] + dz * fwd[2]); // negative because camera looks along -z in view space

      // Perspective projection
      const tanHalfFov = Math.tan((fov * Math.PI) / 360);
      const ndcX = viewX / (viewZ * tanHalfFov * aspect);
      const ndcY = viewY / (viewZ * tanHalfFov);

      // NDC to pixel
      const px = rect.left + ((ndcX + 1) / 2) * rect.width;
      const py = rect.top + ((-ndcY + 1) / 2) * rect.height;

      return { px, py };
    },
    { sx: sketchX, sy: sketchY }
  );

  return result;
}

// ─── Camera Stabilization ─────────────────────────────────────────

/**
 * Wait for the Three.js camera to stop moving (lerp convergence).
 * Since we can't access the camera directly, we wait a fixed time
 * based on the known lerp rate (0.08/frame, ~60fps = ~1.5s to converge).
 */
export async function waitForCameraStable(
  page: Page,
  timeout = 2000
): Promise<void> {
  await page.waitForTimeout(timeout);
}

// ─── Viewport Interaction Helpers ─────────────────────────────────

/** Click at a specific 2D sketch-plane coordinate in the viewport */
export async function clickAtSketchCoord(
  page: Page,
  x: number,
  y: number
): Promise<void> {
  const { px, py } = await sketchCoordToScreenPixel(page, x, y);
  await page.mouse.click(px, py);
}

/**
 * Draw a rectangle by activating the tool and clicking two corners.
 * Must be in sketch mode with camera stable.
 */
export async function drawRectangle(
  page: Page,
  x1: number,
  y1: number,
  x2: number,
  y2: number
): Promise<void> {
  // Activate rectangle tool via keyboard
  await page.keyboard.press("r");
  await page.waitForTimeout(150);

  // Click first corner
  await clickAtSketchCoord(page, x1, y1);

  // Wait for pending point to register
  await page.waitForFunction(
    () => {
      const s = (window as any).__editorStore?.getState();
      return s?.sketchSession?.pendingPoints?.length >= 1;
    },
    { timeout: 5000 }
  );

  // Click second corner
  await clickAtSketchCoord(page, x2, y2);

  // Wait for rectangle entities to be created (4 points + 4 lines = 8 entities)
  await page.waitForFunction(
    () => {
      const s = (window as any).__editorStore?.getState();
      return (s?.sketchSession?.entities?.length ?? 0) >= 8;
    },
    { timeout: 5000 }
  );
}

/**
 * Draw a line by activating the tool and clicking two points.
 * Must be in sketch mode with camera stable.
 */
export async function drawLine(
  page: Page,
  x1: number,
  y1: number,
  x2: number,
  y2: number
): Promise<void> {
  await page.keyboard.press("l");
  await page.waitForTimeout(150);

  const entsBefore = await page.evaluate(() => {
    return (
      (window as any).__editorStore?.getState()?.sketchSession?.entities
        ?.length ?? 0
    );
  });

  // Click start point
  await clickAtSketchCoord(page, x1, y1);
  await page.waitForTimeout(300);

  // Click end point
  await clickAtSketchCoord(page, x2, y2);

  // Wait for at least 1 line to be created
  await page.waitForFunction(
    (before) => {
      const s = (window as any).__editorStore?.getState();
      const ents = s?.sketchSession?.entities ?? [];
      const lineCount = ents.filter((e: any) => e.type === "line").length;
      return lineCount > 0 && ents.length >= before + 2;
    },
    { timeout: 5000 },
    entsBefore
  );

  // Deactivate line tool (it's a chain tool)
  await page.keyboard.press("Escape");
  await page.waitForTimeout(150);
}

/**
 * Draw a circle by activating the tool, clicking center, then radius point.
 */
export async function drawCircle(
  page: Page,
  cx: number,
  cy: number,
  radius: number
): Promise<void> {
  await page.keyboard.press("c");
  await page.waitForTimeout(150);

  const entsBefore = await page.evaluate(() => {
    return (
      (window as any).__editorStore?.getState()?.sketchSession?.entities
        ?.length ?? 0
    );
  });

  await clickAtSketchCoord(page, cx, cy);
  await page.waitForTimeout(200);
  await clickAtSketchCoord(page, cx + radius, cy);

  // Wait for circle entity to appear
  await page.waitForFunction(
    () => {
      const s = (window as any).__editorStore?.getState();
      const ents = s?.sketchSession?.entities ?? [];
      return ents.some((e: any) => e.type === "circle");
    },
    { timeout: 5000 }
  );
}

// ─── Workflow Composers ───────────────────────────────────────────

/**
 * Complete workflow: enter sketch mode, draw a rectangle, confirm.
 */
export async function sketchRectangleOnPlane(
  page: Page,
  plane: "front" | "top" | "right",
  width: number,
  height: number
): Promise<void> {
  await enterSketchMode(page, plane);
  await waitForCameraStable(page);
  await drawRectangle(page, -width / 2, -height / 2, width / 2, height / 2);
  await confirmSketch(page);
}

/**
 * Start an extrude operation, set depth via UI input, and confirm.
 */
export async function extrudeLatestSketch(
  page: Page,
  depth: number
): Promise<void> {
  await page.keyboard.press("e");

  await expect(
    page.locator('[data-testid="extrude-depth"]')
  ).toBeVisible({ timeout: 10000 });

  const depthInput = page.locator('[data-testid="extrude-depth"]');
  await depthInput.fill(String(depth));
  await page.waitForTimeout(200);

  // Click confirm button (Enter key gets captured by the focused input)
  await page.locator('[data-testid="operation-confirm"]').click();

  await page.waitForFunction(
    () => {
      const s = (window as any).__editorStore?.getState();
      return s?.activeOperation === null;
    },
    { timeout: 10000 }
  );
}

/**
 * Full box workflow: sketch rectangle on front plane + extrude.
 */
export async function fullBoxWorkflow(
  page: Page,
  width: number,
  height: number,
  depth: number
): Promise<void> {
  await sketchRectangleOnPlane(page, "front", width, height);
  await extrudeLatestSketch(page, depth);
}

// ─── Assertion Helpers ────────────────────────────────────────────

export async function expectFeatureCount(
  page: Page,
  n: number
): Promise<void> {
  const expected = n === 1 ? "1 feature" : `${n} feature`;
  await expect(
    page.locator('[data-testid="feature-count"]')
  ).toContainText(expected, { timeout: 10000 });
}

export async function expectMeshExists(page: Page): Promise<void> {
  await expect(
    page.locator('[data-testid="vertex-count"]')
  ).toContainText("Verts:", { timeout: 10000 });
}

export async function expectSketchEntityCount(
  page: Page,
  n: number
): Promise<void> {
  await page.waitForFunction(
    (expected) => {
      const s = (window as any).__editorStore?.getState();
      return (s?.sketchSession?.entities?.length ?? 0) === expected;
    },
    { timeout: 5000 },
    n
  );
}

export async function expectStatusText(
  page: Page,
  text: string
): Promise<void> {
  await expect(
    page.locator('[data-testid="status-text"]')
  ).toContainText(text, { timeout: 5000 });
}
