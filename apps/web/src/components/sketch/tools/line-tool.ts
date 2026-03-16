import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";
import { findNearestPoint } from "./snap-utils";

const SNAP_THRESHOLD = 8 * (Math.PI / 180); // 8 degrees

export function applySnap(
  from: SketchPoint2D,
  to: SketchPoint2D
): { snapped: SketchPoint2D; snapType: "h" | "v" | null } {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const angle = Math.atan2(Math.abs(dy), Math.abs(dx));

  if (angle < SNAP_THRESHOLD) {
    // Near horizontal — snap y
    return { snapped: { x: to.x, y: from.y }, snapType: "h" };
  }
  if (angle > Math.PI / 2 - SNAP_THRESHOLD) {
    // Near vertical — snap x
    return { snapped: { x: from.x, y: to.y }, snapType: "v" };
  }
  return { snapped: to, snapType: null };
}

/** Get snap preview info for cursor display */
export function getSnapPreview(
  from: SketchPoint2D,
  to: SketchPoint2D
): { snapped: SketchPoint2D; snapType: "h" | "v" | null } {
  return applySnap(from, to);
}

export function handleLineClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  if (session.pendingPoints.length === 0) {
    // First click: check for coincident snap to existing point
    const snap = findNearestPoint(clickPos, session.entities);
    if (snap) {
      // Snap to existing point — reuse it
      store.addPendingPoint(snap.position);
    } else {
      // Create new start point
      const ptId = store.genSketchEntityId();
      store.addSketchEntity({ type: "point", id: ptId, position: clickPos });
      store.addPendingPoint(clickPos);
    }
  } else {
    // Second click: apply snap then create end point and line
    const fromPt = session.pendingPoints[session.pendingPoints.length - 1]!;
    const { snapped } = applySnap(fromPt, clickPos);

    // Find the start point (either snapped-to or last created)
    const startPt = findStartPoint(session.entities, fromPt);
    if (!startPt) return;

    store.beginUndoBatch();

    // Check for coincident snap on end point
    const endSnap = findNearestPoint(snapped, session.entities);
    let endPtId: string;
    if (endSnap) {
      endPtId = endSnap.id;
    } else {
      endPtId = store.genSketchEntityId();
      store.addSketchEntity({ type: "point", id: endPtId, position: snapped });
    }

    const lineId = store.genSketchEntityId();
    store.addSketchEntity({
      type: "line",
      id: lineId,
      startId: startPt.id,
      endId: endPtId,
    });

    store.endUndoBatch();

    // Chain mode: start next line from this endpoint
    store.clearPendingPoints();
    store.addPendingPoint(endSnap ? endSnap.position : snapped);
  }
}

/** Find the point entity closest to a position (for matching pending point to entity) */
function findStartPoint(
  entities: any[],
  pos: SketchPoint2D
): { id: string } | null {
  const points = entities.filter((e: any) => e.type === "point");
  let best: any = null;
  let bestDist = 0.01;
  for (const pt of points) {
    const dx = pt.position.x - pos.x;
    const dy = pt.position.y - pos.y;
    const dist = Math.sqrt(dx * dx + dy * dy);
    if (dist < bestDist) {
      bestDist = dist;
      best = pt;
    }
  }
  return best || (points.length > 0 ? points[points.length - 1] : null);
}
