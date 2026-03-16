import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";
import { getLineEndpoints, lineLineIntersection } from "./geometry-utils";

/**
 * Sketch Fillet tool: click near a line-line intersection to create
 * a fillet arc tangent to both lines.
 *
 * Default fillet radius; user can change via prompt.
 */
let filletRadius = 1.0;

export function setFilletRadius(r: number): void {
  filletRadius = r;
}

export function getFilletRadius(): number {
  return filletRadius;
}

/**
 * Handle sketch fillet click: find the nearest intersection of two lines
 * and insert a fillet arc.
 */
export function handleSketchFilletClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const entities = session.entities;
  const lines = entities.filter(e => e.type === "line");

  // Find the intersection nearest to the click
  let bestDist = Infinity;
  let bestIntersection: {
    point: SketchPoint2D;
    line1Id: string;
    line2Id: string;
    t1: number;
    t2: number;
  } | null = null;

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

      // Only consider intersections that are on or near both segments
      if (ix.t < -0.1 || ix.t > 1.1 || ix.u < -0.1 || ix.u > 1.1) continue;

      const dist = Math.sqrt(
        (clickPos.x - ix.point.x) ** 2 + (clickPos.y - ix.point.y) ** 2
      );

      if (dist < bestDist) {
        bestDist = dist;
        bestIntersection = {
          point: ix.point,
          line1Id: l1.id,
          line2Id: l2.id,
          t1: ix.t,
          t2: ix.u,
        };
      }
    }
  }

  if (!bestIntersection || bestDist > 3.0) return;

  const l1 = entities.find(e => e.id === bestIntersection!.line1Id);
  const l2 = entities.find(e => e.id === bestIntersection!.line2Id);
  if (!l1 || l1.type !== "line" || !l2 || l2.type !== "line") return;

  const ep1 = getLineEndpoints(entities, l1);
  const ep2 = getLineEndpoints(entities, l2);
  if (!ep1 || !ep2) return;

  const ix = bestIntersection.point;
  const r = filletRadius;

  // Compute unit direction vectors along each line away from intersection
  const dx1 = ep1.end.x - ep1.start.x;
  const dy1 = ep1.end.y - ep1.start.y;
  const len1 = Math.sqrt(dx1 * dx1 + dy1 * dy1);
  if (len1 < 1e-12) return;

  const dx2 = ep2.end.x - ep2.start.x;
  const dy2 = ep2.end.y - ep2.start.y;
  const len2 = Math.sqrt(dx2 * dx2 + dy2 * dy2);
  if (len2 < 1e-12) return;

  // Unit vectors
  let u1x = dx1 / len1, u1y = dy1 / len1;
  let u2x = dx2 / len2, u2y = dy2 / len2;

  // Make sure directions point away from intersection
  // If t is near 0, the intersection is at the start → direction goes toward end (positive)
  // If t is near 1, the intersection is at the end → direction goes toward start (negative)
  if (bestIntersection.t1 > 0.5) { u1x = -u1x; u1y = -u1y; }
  if (bestIntersection.t2 > 0.5) { u2x = -u2x; u2y = -u2y; }

  // Half-angle between the two lines
  const dot = u1x * u2x + u1y * u2y;
  const halfAngle = Math.acos(Math.max(-1, Math.min(1, dot))) / 2;
  if (Math.abs(Math.sin(halfAngle)) < 1e-9) return; // Lines are parallel

  // Distance from intersection to tangent points
  const trimDist = r / Math.tan(halfAngle);
  if (trimDist < 1e-9) return;

  // Tangent points on each line
  const tp1: SketchPoint2D = { x: ix.x + u1x * trimDist, y: ix.y + u1y * trimDist };
  const tp2: SketchPoint2D = { x: ix.x + u2x * trimDist, y: ix.y + u2y * trimDist };

  // Fillet center: offset from intersection along bisector
  const bisX = u1x + u2x;
  const bisY = u1y + u2y;
  const bisLen = Math.sqrt(bisX * bisX + bisY * bisY);
  if (bisLen < 1e-12) return;

  const centerDist = r / Math.sin(halfAngle);
  const center: SketchPoint2D = {
    x: ix.x + (bisX / bisLen) * centerDist,
    y: ix.y + (bisY / bisLen) * centerDist,
  };

  store.beginUndoBatch();

  // Delete original lines
  store.deleteSelectedEntities([bestIntersection.line1Id, bestIntersection.line2Id]);

  // Create new trimmed lines and fillet arc
  // Tangent point 1
  const tp1Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: tp1Id, position: tp1 });

  // Tangent point 2
  const tp2Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: tp2Id, position: tp2 });

  // Center point
  const centerId = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: centerId, position: center });

  // Fillet arc from tp1 to tp2 around center
  const arcId = store.genSketchEntityId();
  store.addSketchEntity({
    type: "arc",
    id: arcId,
    centerId,
    startId: tp1Id,
    endId: tp2Id,
    radius: r,
  });

  // Re-create the trimmed portions of line1 and line2
  // Line1: from its far end to tangent point 1
  const line1FarEnd = bestIntersection.t1 > 0.5 ? l1.startId : l1.endId;
  const newLine1Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "line", id: newLine1Id, startId: line1FarEnd, endId: tp1Id });

  // Line2: from its far end to tangent point 2
  const line2FarEnd = bestIntersection.t2 > 0.5 ? l2.startId : l2.endId;
  const newLine2Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "line", id: newLine2Id, startId: line2FarEnd, endId: tp2Id });

  store.endUndoBatch();
}
