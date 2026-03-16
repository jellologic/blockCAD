import type { SketchPoint2D, SketchEntity2D } from "@blockCAD/kernel";

const POINT_SNAP_THRESHOLD = 0.5;

/**
 * Find the nearest existing point entity within snap threshold.
 * Returns the point's id and position if found, null otherwise.
 */
export function findNearestPoint(
  pos: SketchPoint2D,
  entities: SketchEntity2D[],
  threshold: number = POINT_SNAP_THRESHOLD
): { id: string; position: SketchPoint2D } | null {
  let best: { id: string; position: SketchPoint2D } | null = null;
  let bestDist = threshold;

  for (const entity of entities) {
    if (entity.type !== "point") continue;
    const dx = pos.x - entity.position.x;
    const dy = pos.y - entity.position.y;
    const dist = Math.sqrt(dx * dx + dy * dy);
    if (dist < bestDist) {
      bestDist = dist;
      best = { id: entity.id, position: entity.position };
    }
  }

  return best;
}

/**
 * Determine the snap type for the current cursor position relative to existing points.
 * Returns "coincident" if near an existing point, null otherwise.
 */
export function getCoincidentSnap(
  pos: SketchPoint2D,
  entities: SketchEntity2D[],
  threshold: number = POINT_SNAP_THRESHOLD
): { type: "coincident"; target: { id: string; position: SketchPoint2D } } | null {
  const nearest = findNearestPoint(pos, entities, threshold);
  if (nearest) return { type: "coincident", target: nearest };
  return null;
}
