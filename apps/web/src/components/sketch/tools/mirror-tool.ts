import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";
import { getLineEndpoints, reflectPointAcrossLine } from "./geometry-utils";

/**
 * Mirror tool: two-step workflow.
 * Step 1: Click on entities to select for mirroring (collect point/line IDs)
 * Step 2: Click on a mirror axis line
 *
 * Simplified version: click a line to mirror, then click the axis line.
 */
export function handleMirrorClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const pending = session.pendingPoints;
  const entities = session.entities;

  if (pending.length === 0) {
    // First click: select entity to mirror (find nearest line)
    store.addPendingPoint(clickPos);
    return;
  }

  // Second click: select mirror axis line and perform mirror
  const firstClick = pending[0]!;

  // Find nearest line to first click (entity to mirror)
  const sourceLine = findNearestLine(entities, firstClick);
  // Find nearest line to second click (mirror axis)
  const axisLine = findNearestLine(entities, clickPos);

  if (!sourceLine || !axisLine || sourceLine.id === axisLine.id) {
    store.clearPendingPoints();
    return;
  }

  const sourceEndpoints = getLineEndpoints(entities, sourceLine);
  const axisEndpoints = getLineEndpoints(entities, axisLine);
  if (!sourceEndpoints || !axisEndpoints) {
    store.clearPendingPoints();
    return;
  }

  store.beginUndoBatch();

  // Mirror start point
  const mirroredStart = reflectPointAcrossLine(
    sourceEndpoints.start, axisEndpoints.start, axisEndpoints.end
  );
  const p1Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: p1Id, position: mirroredStart });

  // Mirror end point
  const mirroredEnd = reflectPointAcrossLine(
    sourceEndpoints.end, axisEndpoints.start, axisEndpoints.end
  );
  const p2Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: p2Id, position: mirroredEnd });

  // Create mirrored line
  const lineId = store.genSketchEntityId();
  store.addSketchEntity({ type: "line", id: lineId, startId: p1Id, endId: p2Id });

  // Add symmetric constraints (original start ↔ mirrored start about axis)
  const sc1 = store.genSketchConstraintId();
  store.addSketchConstraint({
    id: sc1,
    kind: "symmetric",
    entityIds: [sourceLine.startId, p1Id, axisLine.id],
  });

  const sc2 = store.genSketchConstraintId();
  store.addSketchConstraint({
    id: sc2,
    kind: "symmetric",
    entityIds: [sourceLine.endId, p2Id, axisLine.id],
  });

  store.endUndoBatch();
  store.clearPendingPoints();
}

function findNearestLine(
  entities: import("@blockCAD/kernel").SketchEntity2D[],
  pos: SketchPoint2D,
): (import("@blockCAD/kernel").SketchEntity2D & { type: "line" }) | null {
  let nearest: any = null;
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
      ((pos.x - endpoints.start.x) * dx + (pos.y - endpoints.start.y) * dy) / len2
    ));
    const projX = endpoints.start.x + t * dx;
    const projY = endpoints.start.y + t * dy;
    const dist = Math.sqrt((pos.x - projX) ** 2 + (pos.y - projY) ** 2);

    if (dist < nearestDist) {
      nearestDist = dist;
      nearest = entity;
    }
  }

  return nearest;
}
