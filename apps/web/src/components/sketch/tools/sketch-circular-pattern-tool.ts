import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";
import { getLineEndpoints } from "./geometry-utils";

let patternCount = 4;

export function setCircularPatternCount(n: number): void {
  patternCount = Math.max(2, Math.round(n));
}

export function getCircularPatternCount(): number {
  return patternCount;
}

/**
 * Sketch circular pattern: click a line, then click a center point.
 * Creates N rotated copies around the center.
 *
 * Step 1: Click near a line to select
 * Step 2: Click to set rotation center
 */
export function handleSketchCircularPatternClick(pos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const pending = session.pendingPoints;
  const entities = session.entities;

  if (pending.length === 0) {
    store.addPendingPoint(pos);
    return;
  }

  const firstClick = pending[0]!;

  // Find nearest line to first click
  let nearestLineId: string | null = null;
  let nearestDist = Infinity;

  for (const entity of entities) {
    if (entity.type !== "line") continue;
    const endpoints = getLineEndpoints(entities, entity);
    if (!endpoints) continue;
    const mx = (endpoints.start.x + endpoints.end.x) / 2;
    const my = (endpoints.start.y + endpoints.end.y) / 2;
    const d = Math.sqrt((firstClick.x - mx) ** 2 + (firstClick.y - my) ** 2);
    if (d < nearestDist) { nearestDist = d; nearestLineId = entity.id; }
  }

  if (!nearestLineId) { store.clearPendingPoints(); return; }

  const lineEntity = entities.find(e => e.id === nearestLineId && e.type === "line") as any;
  if (!lineEntity) { store.clearPendingPoints(); return; }
  const endpoints = getLineEndpoints(entities, lineEntity);
  if (!endpoints) { store.clearPendingPoints(); return; }

  const center = pos; // Second click = rotation center
  const angleStep = (2 * Math.PI) / patternCount;

  store.beginUndoBatch();

  for (let i = 1; i < patternCount; i++) {
    const angle = angleStep * i;
    const cosA = Math.cos(angle);
    const sinA = Math.sin(angle);

    const rotatePoint = (pt: SketchPoint2D): SketchPoint2D => {
      const dx = pt.x - center.x;
      const dy = pt.y - center.y;
      return {
        x: center.x + dx * cosA - dy * sinA,
        y: center.y + dx * sinA + dy * cosA,
      };
    };

    const p1 = rotatePoint(endpoints.start);
    const p2 = rotatePoint(endpoints.end);

    const p1Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p1Id, position: p1 });

    const p2Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p2Id, position: p2 });

    const newLineId = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: newLineId, startId: p1Id, endId: p2Id });

    // Equal length to original
    const eqId = store.genSketchConstraintId();
    store.addSketchConstraint({ id: eqId, kind: "equal", entityIds: [nearestLineId, newLineId] });
  }

  store.endUndoBatch();
  store.clearPendingPoints();
}
