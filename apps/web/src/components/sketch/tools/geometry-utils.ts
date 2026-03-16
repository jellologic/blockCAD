import type { SketchPoint2D, SketchEntity2D } from "@blockCAD/kernel";

/** Result of a line-line intersection. */
export interface IntersectionResult {
  point: SketchPoint2D;
  /** Parameter along first line (0=start, 1=end) */
  t: number;
  /** Parameter along second line (0=start, 1=end) */
  u: number;
}

/**
 * Compute intersection of two line segments (or their extensions).
 * Returns null if lines are parallel.
 * t,u in [0,1] means intersection is within the segment.
 */
export function lineLineIntersection(
  p1: SketchPoint2D, p2: SketchPoint2D,
  p3: SketchPoint2D, p4: SketchPoint2D,
): IntersectionResult | null {
  const dx1 = p2.x - p1.x;
  const dy1 = p2.y - p1.y;
  const dx2 = p4.x - p3.x;
  const dy2 = p4.y - p3.y;

  const denom = dx1 * dy2 - dy1 * dx2;
  if (Math.abs(denom) < 1e-12) return null; // Parallel

  const dx3 = p3.x - p1.x;
  const dy3 = p3.y - p1.y;

  const t = (dx3 * dy2 - dy3 * dx2) / denom;
  const u = (dx3 * dy1 - dy3 * dx1) / denom;

  return {
    point: { x: p1.x + t * dx1, y: p1.y + t * dy1 },
    t,
    u,
  };
}

/**
 * Find a point entity's position from the entity list.
 */
export function getPointPosition(
  entities: SketchEntity2D[],
  pointId: string,
): SketchPoint2D | null {
  const entity = entities.find(e => e.id === pointId);
  if (entity?.type === "point") return entity.position;
  return null;
}

/**
 * Get line endpoint positions.
 */
export function getLineEndpoints(
  entities: SketchEntity2D[],
  lineEntity: SketchEntity2D & { type: "line" },
): { start: SketchPoint2D; end: SketchPoint2D } | null {
  const start = getPointPosition(entities, lineEntity.startId);
  const end = getPointPosition(entities, lineEntity.endId);
  if (!start || !end) return null;
  return { start, end };
}

/**
 * Find all intersections of a line with other lines in the sketch.
 * Returns intersections sorted by parameter t along the query line.
 */
export function findIntersectionsWithLine(
  lineId: string,
  entities: SketchEntity2D[],
): Array<{ point: SketchPoint2D; t: number; otherLineId: string }> {
  const line = entities.find(e => e.id === lineId && e.type === "line") as
    (SketchEntity2D & { type: "line" }) | undefined;
  if (!line) return [];

  const endpoints = getLineEndpoints(entities, line);
  if (!endpoints) return [];

  const results: Array<{ point: SketchPoint2D; t: number; otherLineId: string }> = [];

  for (const other of entities) {
    if (other.type !== "line" || other.id === lineId) continue;
    const otherEndpoints = getLineEndpoints(entities, other as SketchEntity2D & { type: "line" });
    if (!otherEndpoints) continue;

    const ix = lineLineIntersection(
      endpoints.start, endpoints.end,
      otherEndpoints.start, otherEndpoints.end,
    );

    if (ix && ix.u >= -0.001 && ix.u <= 1.001) {
      // Intersection is on the other line segment
      results.push({ point: ix.point, t: ix.t, otherLineId: other.id });
    }
  }

  results.sort((a, b) => a.t - b.t);
  return results;
}

/**
 * Reflect a point across a line (mirror axis).
 */
export function reflectPointAcrossLine(
  point: SketchPoint2D,
  lineStart: SketchPoint2D,
  lineEnd: SketchPoint2D,
): SketchPoint2D {
  const dx = lineEnd.x - lineStart.x;
  const dy = lineEnd.y - lineStart.y;
  const len2 = dx * dx + dy * dy;
  if (len2 < 1e-12) return point;

  // Project point onto line
  const t = ((point.x - lineStart.x) * dx + (point.y - lineStart.y) * dy) / len2;
  const projX = lineStart.x + t * dx;
  const projY = lineStart.y + t * dy;

  // Reflect: mirrored = 2 * projection - original
  return {
    x: 2 * projX - point.x,
    y: 2 * projY - point.y,
  };
}

/**
 * Offset a line by a perpendicular distance.
 * Positive distance = left side (relative to start→end direction).
 */
export function offsetLine(
  start: SketchPoint2D,
  end: SketchPoint2D,
  distance: number,
): { start: SketchPoint2D; end: SketchPoint2D } {
  const dx = end.x - start.x;
  const dy = end.y - start.y;
  const len = Math.sqrt(dx * dx + dy * dy);
  if (len < 1e-12) return { start, end };

  // Perpendicular direction (left normal)
  const nx = -dy / len;
  const ny = dx / len;

  return {
    start: { x: start.x + nx * distance, y: start.y + ny * distance },
    end: { x: end.x + nx * distance, y: end.y + ny * distance },
  };
}
