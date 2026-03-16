import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";

/**
 * Ellipse tool: click to set center, then click to set radii.
 *
 * Step 1: Click → center point
 * Step 2: Click → sets radius_x (horizontal distance from center)
 *         and radius_y (vertical distance from center)
 */
export function handleEllipseClick(pos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const pending = session.pendingPoints;

  if (pending.length === 0) {
    // First click: set center
    store.addPendingPoint(pos);
    return;
  }

  // Second click: compute radii from center to click position
  const center = pending[0]!;
  const rx = Math.abs(pos.x - center.x);
  const ry = Math.abs(pos.y - center.y);

  if (rx < 0.01 && ry < 0.01) {
    store.clearPendingPoints();
    return;
  }

  store.beginUndoBatch();

  // Create center point
  const centerId = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: centerId, position: center });

  // Create ellipse as a circle (for now — the kernel Ellipse entity isn't
  // yet supported in the frontend SketchEntity2D type, so we approximate
  // with a circle using the average radius)
  const avgRadius = (rx + ry) / 2;
  const circleId = store.genSketchEntityId();
  store.addSketchEntity({ type: "circle", id: circleId, centerId, radius: avgRadius });

  // If radii are very different, add dimension constraints to communicate intent
  // (Future: when frontend supports Ellipse entity natively, create true ellipse)

  store.endUndoBatch();
  store.clearPendingPoints();
}
