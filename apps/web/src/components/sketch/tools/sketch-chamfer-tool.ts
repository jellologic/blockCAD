import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";
import { getLineEndpoints, lineLineIntersection } from "./geometry-utils";

/**
 * Sketch Chamfer tool: click near a line-line intersection to create
 * a chamfer (bevel line) between the two lines.
 *
 * Default chamfer distance; user can change via prompt.
 */
let chamferDistance = 1.0;

export function setChamferDistance(d: number): void {
  chamferDistance = d;
}

export function getChamferDistance(): number {
  return chamferDistance;
}

/**
 * Handle sketch chamfer click: find the nearest intersection of two lines
 * and insert a chamfer bevel.
 */
export function handleSketchChamferClick(clickPos: SketchPoint2D): void {
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
  const d = chamferDistance;

  // Compute unit direction vectors along each line away from intersection
  const dx1 = ep1.end.x - ep1.start.x;
  const dy1 = ep1.end.y - ep1.start.y;
  const len1 = Math.sqrt(dx1 * dx1 + dy1 * dy1);
  if (len1 < 1e-12) return;

  const dx2 = ep2.end.x - ep2.start.x;
  const dy2 = ep2.end.y - ep2.start.y;
  const len2 = Math.sqrt(dx2 * dx2 + dy2 * dy2);
  if (len2 < 1e-12) return;

  let u1x = dx1 / len1, u1y = dy1 / len1;
  let u2x = dx2 / len2, u2y = dy2 / len2;

  // Make sure directions point away from intersection
  if (bestIntersection.t1 > 0.5) { u1x = -u1x; u1y = -u1y; }
  if (bestIntersection.t2 > 0.5) { u2x = -u2x; u2y = -u2y; }

  // Chamfer points at distance d from intersection along each line
  const cp1: SketchPoint2D = { x: ix.x + u1x * d, y: ix.y + u1y * d };
  const cp2: SketchPoint2D = { x: ix.x + u2x * d, y: ix.y + u2y * d };

  store.beginUndoBatch();

  // Delete original lines
  store.deleteSelectedEntities([bestIntersection.line1Id, bestIntersection.line2Id]);

  // Create chamfer points
  const cp1Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: cp1Id, position: cp1 });

  const cp2Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: cp2Id, position: cp2 });

  // Chamfer line between the two chamfer points
  const chamferLineId = store.genSketchEntityId();
  store.addSketchEntity({ type: "line", id: chamferLineId, startId: cp1Id, endId: cp2Id });

  // Re-create the trimmed portions of line1 and line2
  const line1FarEnd = bestIntersection.t1 > 0.5 ? l1.startId : l1.endId;
  const newLine1Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "line", id: newLine1Id, startId: line1FarEnd, endId: cp1Id });

  const line2FarEnd = bestIntersection.t2 > 0.5 ? l2.startId : l2.endId;
  const newLine2Id = store.genSketchEntityId();
  store.addSketchEntity({ type: "line", id: newLine2Id, startId: line2FarEnd, endId: cp2Id });

  store.endUndoBatch();
}
