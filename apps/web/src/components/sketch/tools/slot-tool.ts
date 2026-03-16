import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";

/**
 * Slot tool: creates a linear slot (two parallel lines + two semicircular ends).
 *
 * Step 1: Click → first center point
 * Step 2: Click → second center point
 * Step 3: Click → sets slot width (perpendicular distance)
 *
 * Simplified: uses 2 clicks (centers), fixed default width.
 * Generates: 2 center points, 4 corner points, 2 lines, plus constraints.
 */
const DEFAULT_SLOT_WIDTH = 2.0;

export function handleSlotClick(pos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const pending = session.pendingPoints;

  if (pending.length === 0) {
    // First click: slot start center
    store.addPendingPoint(pos);
    return;
  }

  if (pending.length === 1) {
    // Second click: slot end center → generate slot with default width
    const start = pending[0]!;
    const end = pos;

    const dx = end.x - start.x;
    const dy = end.y - start.y;
    const len = Math.sqrt(dx * dx + dy * dy);
    if (len < 0.01) {
      store.clearPendingPoints();
      return;
    }

    const halfWidth = DEFAULT_SLOT_WIDTH / 2;

    // Perpendicular direction
    const nx = -dy / len;
    const ny = dx / len;

    store.beginUndoBatch();

    // 4 corner points
    const p1Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p1Id, position: { x: start.x + nx * halfWidth, y: start.y + ny * halfWidth } });

    const p2Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p2Id, position: { x: end.x + nx * halfWidth, y: end.y + ny * halfWidth } });

    const p3Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p3Id, position: { x: end.x - nx * halfWidth, y: end.y - ny * halfWidth } });

    const p4Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p4Id, position: { x: start.x - nx * halfWidth, y: start.y - ny * halfWidth } });

    // 2 center points for arc ends
    const c1Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: c1Id, position: start });

    const c2Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: c2Id, position: end });

    // 2 parallel side lines
    const line1Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: line1Id, startId: p1Id, endId: p2Id });

    const line2Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: line2Id, startId: p3Id, endId: p4Id });

    // 2 semicircular arcs at ends
    const arc1Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "arc", id: arc1Id, centerId: c2Id, startId: p2Id, endId: p3Id, radius: halfWidth });

    const arc2Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "arc", id: arc2Id, centerId: c1Id, startId: p4Id, endId: p1Id, radius: halfWidth });

    // Add parallel constraint between the two side lines
    const parId = store.genSketchConstraintId();
    store.addSketchConstraint({ id: parId, kind: "parallel", entityIds: [line1Id, line2Id] });

    // Fix start center to anchor
    const fixId = store.genSketchConstraintId();
    store.addSketchConstraint({ id: fixId, kind: "fixed", entityIds: [c1Id] });

    store.endUndoBatch();
    store.clearPendingPoints();
  }
}
