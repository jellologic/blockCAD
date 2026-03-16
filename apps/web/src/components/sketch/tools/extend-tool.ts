import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";
import { getLineEndpoints, lineLineIntersection } from "./geometry-utils";

/**
 * Extend tool: click near an endpoint of a line to extend it
 * to the nearest intersection with another line.
 */
export function handleExtendClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const entities = session.entities;

  // Find the nearest line endpoint to the click
  let nearestLineId: string | null = null;
  let nearestDist = Infinity;
  let extendFromStart = false;

  for (const entity of entities) {
    if (entity.type !== "line") continue;
    const endpoints = getLineEndpoints(entities, entity);
    if (!endpoints) continue;

    const distToStart = Math.sqrt((clickPos.x - endpoints.start.x) ** 2 + (clickPos.y - endpoints.start.y) ** 2);
    const distToEnd = Math.sqrt((clickPos.x - endpoints.end.x) ** 2 + (clickPos.y - endpoints.end.y) ** 2);

    const minDist = Math.min(distToStart, distToEnd);
    if (minDist < nearestDist) {
      nearestDist = minDist;
      nearestLineId = entity.id;
      extendFromStart = distToStart < distToEnd;
    }
  }

  if (!nearestLineId || nearestDist > 2.0) return;

  const lineEntity = entities.find(e => e.id === nearestLineId && e.type === "line");
  if (!lineEntity || lineEntity.type !== "line") return;

  const endpoints = getLineEndpoints(entities, lineEntity);
  if (!endpoints) return;

  // Find intersections of this line's extension with other lines
  // Look for intersections with t outside [0,1] in the extend direction
  let bestIntersection: { point: SketchPoint2D; t: number } | null = null;
  let bestAbsT = Infinity;

  for (const other of entities) {
    if (other.type !== "line" || other.id === nearestLineId) continue;
    const otherEndpoints = getLineEndpoints(entities, other);
    if (!otherEndpoints) continue;

    const ix = lineLineIntersection(
      endpoints.start, endpoints.end,
      otherEndpoints.start, otherEndpoints.end,
    );
    if (!ix) continue;
    if (ix.u < -0.001 || ix.u > 1.001) continue; // Not on other segment

    // For extending from start: t < 0
    // For extending from end: t > 1
    if (extendFromStart && ix.t < 0 && Math.abs(ix.t) < bestAbsT) {
      bestAbsT = Math.abs(ix.t);
      bestIntersection = ix;
    }
    if (!extendFromStart && ix.t > 1 && Math.abs(ix.t - 1) < bestAbsT) {
      bestAbsT = Math.abs(ix.t - 1);
      bestIntersection = ix;
    }
  }

  if (!bestIntersection) return;

  store.beginUndoBatch();

  // Create intersection point
  const ptId = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: ptId, position: bestIntersection.point });

  // Delete original line
  store.deleteSelectedEntities([nearestLineId]);

  // Create new extended line
  const newLineId = store.genSketchEntityId();
  if (extendFromStart) {
    store.addSketchEntity({ type: "line", id: newLineId, startId: ptId, endId: lineEntity.endId });
  } else {
    store.addSketchEntity({ type: "line", id: newLineId, startId: lineEntity.startId, endId: ptId });
  }

  store.endUndoBatch();
}
