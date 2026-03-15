import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";

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
    // First click: create start point, add to pending
    const ptId = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: ptId, position: clickPos });
    store.addPendingPoint(clickPos);
  } else {
    // Second click: apply snap then create end point and line
    const fromPt = session.pendingPoints[session.pendingPoints.length - 1]!;
    const { snapped } = applySnap(fromPt, clickPos);

    const startPt = store.sketchSession?.entities.filter(
      (e) => e.type === "point"
    ).slice(-1)[0];
    if (!startPt) return;

    const endPtId = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: endPtId, position: snapped });

    const lineId = store.genSketchEntityId();
    store.addSketchEntity({
      type: "line",
      id: lineId,
      startId: startPt.id,
      endId: endPtId,
    });

    // Chain mode: start next line from this endpoint
    store.clearPendingPoints();
    store.addPendingPoint(snapped);
  }
}
