import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";
import { findNearestPoint } from "./snap-utils";

/**
 * Handle a click in measure mode.
 * First click: set pending point.
 * Second click: compute distance and display on screen.
 */
export function handleMeasureClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  // Snap to existing point if nearby
  const snap = findNearestPoint(clickPos, session.entities);
  const pos = snap ? snap.position : clickPos;

  if (session.pendingPoints.length === 0) {
    // First click: store the start point, clear any previous measurement
    store.setMeasureResult(null);
    store.addPendingPoint(pos);
  } else {
    // Second click: compute distance and show on screen
    const startPt = session.pendingPoints[0]!;
    const dx = pos.x - startPt.x;
    const dy = pos.y - startPt.y;
    const distance = Math.sqrt(dx * dx + dy * dy);

    store.setMeasureResult({
      from: startPt,
      to: pos,
      distance,
    });

    // Clear pending to allow new measurement
    store.clearPendingPoints();
  }
}
