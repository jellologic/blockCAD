import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";
import { getLineEndpoints, offsetLine } from "./geometry-utils";

/**
 * Offset tool: create a parallel copy of a line at a specified distance.
 * Click a line, then click to set the offset side/distance.
 */
export function handleOffsetClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const pending = session.pendingPoints;
  const entities = session.entities;

  if (pending.length === 0) {
    // First click: select the line to offset
    store.addPendingPoint(clickPos);
    return;
  }

  // Second click: create offset line
  const firstClick = pending[0]!;

  // Find nearest line to first click
  let nearestLineId: string | null = null;
  let nearestDist = Infinity;

  for (const entity of entities) {
    if (entity.type !== "line") continue;
    const endpoints = getLineEndpoints(entities, entity);
    if (!endpoints) continue;

    const dx = endpoints.end.x - endpoints.start.x;
    const dy = endpoints.end.y - endpoints.start.y;
    const len2 = dx * dx + dy * dy;
    if (len2 < 1e-12) continue;

    const t = Math.max(0, Math.min(1,
      ((firstClick.x - endpoints.start.x) * dx + (firstClick.y - endpoints.start.y) * dy) / len2
    ));
    const projX = endpoints.start.x + t * dx;
    const projY = endpoints.start.y + t * dy;
    const dist = Math.sqrt((firstClick.x - projX) ** 2 + (firstClick.y - projY) ** 2);

    if (dist < nearestDist) {
      nearestDist = dist;
      nearestLineId = entity.id;
    }
  }

  if (!nearestLineId) {
    store.clearPendingPoints();
    return;
  }

  const lineEntity = entities.find(e => e.id === nearestLineId && e.type === "line");
  if (!lineEntity || lineEntity.type !== "line") {
    store.clearPendingPoints();
    return;
  }

  const endpoints = getLineEndpoints(entities, lineEntity);
  if (!endpoints) {
    store.clearPendingPoints();
    return;
  }

  // Compute offset distance from second click to line
  const dx = endpoints.end.x - endpoints.start.x;
  const dy = endpoints.end.y - endpoints.start.y;
  const len = Math.sqrt(dx * dx + dy * dy);
  if (len < 1e-12) { store.clearPendingPoints(); return; }

  // Perpendicular distance with sign
  const nx = -dy / len;
  const ny = dx / len;
  const distance = (clickPos.x - endpoints.start.x) * nx + (clickPos.y - endpoints.start.y) * ny;

  // Create offset line
  const offset = offsetLine(endpoints.start, endpoints.end, distance);

  store.beginUndoBatch();

  const p1Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: p1Id, position: offset.start });

  const p2Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: p2Id, position: offset.end });

  const newLineId = store.genSketchEntityId();
  store.addSketchEntity({ type: "line", id: newLineId, startId: p1Id, endId: p2Id });

  // Add parallel constraint
  const cId = store.genSketchConstraintId();
  store.addSketchConstraint({
    id: cId,
    kind: "parallel",
    entityIds: [nearestLineId, newLineId],
  });

  store.endUndoBatch();
  store.clearPendingPoints();
}
