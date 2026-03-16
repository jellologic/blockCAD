import type { SketchPoint2D, SketchEntity2D } from "@blockCAD/kernel";
import { lineLineIntersection, getPointPosition, getLineEndpoints } from "./geometry-utils";

const POINT_SNAP_THRESHOLD = 0.5;

export type SnapType = "coincident" | "midpoint" | "center" | "intersection";

export interface SnapResult {
  type: SnapType;
  position: SketchPoint2D;
  targetId?: string;
}

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

/**
 * Find the best snap target from multiple snap sources.
 * Priority: coincident > midpoint > center > intersection
 */
export function findSnapTarget(
  pos: SketchPoint2D,
  entities: SketchEntity2D[],
  threshold: number = POINT_SNAP_THRESHOLD
): SnapResult | null {
  let best: SnapResult | null = null;
  let bestDist = threshold;

  // 1. Point coincident (highest priority)
  for (const entity of entities) {
    if (entity.type !== "point") continue;
    const dx = pos.x - entity.position.x;
    const dy = pos.y - entity.position.y;
    const dist = Math.sqrt(dx * dx + dy * dy);
    if (dist < bestDist) {
      bestDist = dist;
      best = { type: "coincident", position: entity.position, targetId: entity.id };
    }
  }

  // If we found a coincident snap, return it (highest priority)
  if (best) return best;

  // 2. Line midpoints
  for (const entity of entities) {
    if (entity.type !== "line") continue;
    const endpoints = getLineEndpoints(entities, entity);
    if (!endpoints) continue;
    const mid: SketchPoint2D = {
      x: (endpoints.start.x + endpoints.end.x) / 2,
      y: (endpoints.start.y + endpoints.end.y) / 2,
    };
    const dx = pos.x - mid.x;
    const dy = pos.y - mid.y;
    const dist = Math.sqrt(dx * dx + dy * dy);
    if (dist < bestDist) {
      bestDist = dist;
      best = { type: "midpoint", position: mid, targetId: entity.id };
    }
  }

  // 3. Circle/arc centers
  for (const entity of entities) {
    if (entity.type === "circle" || entity.type === "arc") {
      const centerPos = getPointPosition(entities, entity.centerId);
      if (!centerPos) continue;
      const dx = pos.x - centerPos.x;
      const dy = pos.y - centerPos.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < bestDist) {
        bestDist = dist;
        best = { type: "center", position: centerPos, targetId: entity.centerId };
      }
    }
  }

  // 4. Line-line intersections
  const lines = entities.filter(e => e.type === "line");
  for (let i = 0; i < lines.length; i++) {
    const l1 = lines[i];
    if (l1.type !== "line") continue;
    const ep1 = getLineEndpoints(entities, l1);
    if (!ep1) continue;

    for (let j = i + 1; j < lines.length; j++) {
      const l2 = lines[j];
      if (l2.type !== "line") continue;
      const ep2 = getLineEndpoints(entities, l2);
      if (!ep2) continue;

      const ix = lineLineIntersection(ep1.start, ep1.end, ep2.start, ep2.end);
      if (!ix) continue;
      // Only snap to intersections within or near segments
      if (ix.t < -0.1 || ix.t > 1.1 || ix.u < -0.1 || ix.u > 1.1) continue;

      const dx = pos.x - ix.point.x;
      const dy = pos.y - ix.point.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < bestDist) {
        bestDist = dist;
        best = { type: "intersection", position: ix.point };
      }
    }
  }

  return best;
}
