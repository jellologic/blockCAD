import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";

/** Compute circumscribed circle center from 3 points */
export function circumcenter(
  p1: SketchPoint2D,
  p2: SketchPoint2D,
  p3: SketchPoint2D
): { center: SketchPoint2D; radius: number } | null {
  const ax = p1.x, ay = p1.y;
  const bx = p2.x, by = p2.y;
  const cx = p3.x, cy = p3.y;
  const D = 2 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
  if (Math.abs(D) < 1e-10) return null; // collinear
  const ux =
    ((ax * ax + ay * ay) * (by - cy) +
      (bx * bx + by * by) * (cy - ay) +
      (cx * cx + cy * cy) * (ay - by)) /
    D;
  const uy =
    ((ax * ax + ay * ay) * (cx - bx) +
      (bx * bx + by * by) * (ax - cx) +
      (cx * cx + cy * cy) * (bx - ax)) /
    D;
  const radius = Math.sqrt((ax - ux) ** 2 + (ay - uy) ** 2);
  return { center: { x: ux, y: uy }, radius };
}

export function handleArcClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  if (session.pendingPoints.length === 0) {
    // First click: start point
    store.addPendingPoint(clickPos);
  } else if (session.pendingPoints.length === 1) {
    // Second click: end point
    store.addPendingPoint(clickPos);
  } else {
    // Third click: point on arc -> compute center and create entities
    const start = session.pendingPoints[0]!;
    const end = session.pendingPoints[1]!;
    const mid = clickPos;

    const result = circumcenter(start, end, mid);
    if (!result) {
      store.clearPendingPoints();
      return; // collinear points, can't make arc
    }

    const centerId = store.genSketchEntityId();
    const startId = store.genSketchEntityId();
    const endId = store.genSketchEntityId();
    const arcId = store.genSketchEntityId();

    store.addSketchEntity({ type: "point", id: centerId, position: result.center });
    store.addSketchEntity({ type: "point", id: startId, position: start });
    store.addSketchEntity({ type: "point", id: endId, position: end });
    store.addSketchEntity({
      type: "arc",
      id: arcId,
      centerId,
      startId,
      endId,
      radius: result.radius,
    });

    store.clearPendingPoints();
    store.setSketchTool(null);
  }
}
