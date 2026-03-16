import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";
import { findIntersectionsWithLine, getLineEndpoints } from "./geometry-utils";

/**
 * Trim tool: click on a line segment to remove the clicked portion
 * between the two nearest intersection points.
 */
export function handleTrimClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const entities = session.entities;

  // Find the nearest line to the click
  let nearestLineId: string | null = null;
  let nearestDist = Infinity;
  let nearestT = 0;

  for (const entity of entities) {
    if (entity.type !== "line") continue;
    const endpoints = getLineEndpoints(entities, entity);
    if (!endpoints) continue;

    // Project click onto line and compute distance
    const dx = endpoints.end.x - endpoints.start.x;
    const dy = endpoints.end.y - endpoints.start.y;
    const len2 = dx * dx + dy * dy;
    if (len2 < 1e-12) continue;

    const t = Math.max(0, Math.min(1,
      ((clickPos.x - endpoints.start.x) * dx + (clickPos.y - endpoints.start.y) * dy) / len2
    ));
    const projX = endpoints.start.x + t * dx;
    const projY = endpoints.start.y + t * dy;
    const dist = Math.sqrt((clickPos.x - projX) ** 2 + (clickPos.y - projY) ** 2);

    if (dist < nearestDist) {
      nearestDist = dist;
      nearestLineId = entity.id;
      nearestT = t;
    }
  }

  if (!nearestLineId || nearestDist > 2.0) return; // No line close enough

  // Find intersections on this line
  const intersections = findIntersectionsWithLine(nearestLineId, entities);

  if (intersections.length === 0) return; // No intersections to trim at

  // Find the two intersection points bracketing the click parameter t
  let leftT = 0; // start of line
  let rightT = 1; // end of line

  for (const ix of intersections) {
    if (ix.t <= nearestT && ix.t > leftT) leftT = ix.t;
    if (ix.t >= nearestT && ix.t < rightT) rightT = ix.t;
  }

  // If click is between two intersections, delete the middle portion
  // by removing the line and creating two new segments
  const lineEntity = entities.find(e => e.id === nearestLineId && e.type === "line");
  if (!lineEntity || lineEntity.type !== "line") return;

  const endpoints = getLineEndpoints(entities, lineEntity);
  if (!endpoints) return;

  store.beginUndoBatch();

  // Delete the original line
  store.deleteSelectedEntities([nearestLineId]);

  // Create new segments if they have non-zero length
  if (leftT > 0.01) {
    // Segment from start to left intersection
    const midPt: SketchPoint2D = {
      x: endpoints.start.x + leftT * (endpoints.end.x - endpoints.start.x),
      y: endpoints.start.y + leftT * (endpoints.end.y - endpoints.start.y),
    };
    const ptId = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: ptId, position: midPt });
    const lineId = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: lineId, startId: lineEntity.startId, endId: ptId });
  }

  if (rightT < 0.99) {
    // Segment from right intersection to end
    const midPt: SketchPoint2D = {
      x: endpoints.start.x + rightT * (endpoints.end.x - endpoints.start.x),
      y: endpoints.start.y + rightT * (endpoints.end.y - endpoints.start.y),
    };
    const ptId = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: ptId, position: midPt });
    const lineId = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: lineId, startId: ptId, endId: lineEntity.endId });
  }

  store.endUndoBatch();
}
