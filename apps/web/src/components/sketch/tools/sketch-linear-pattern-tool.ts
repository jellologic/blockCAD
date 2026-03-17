import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";
import { getLineEndpoints } from "./geometry-utils";

let patternCount = 3;

export function setLinearPatternCount(n: number): void {
  patternCount = Math.max(2, Math.round(n));
}

export function getLinearPatternCount(): number {
  return patternCount;
}

/**
 * Sketch linear pattern: click a line entity, then click to set
 * direction and spacing. Creates N copies along the direction.
 *
 * Step 1: Click near a line to select it for patterning
 * Step 2: Click to define direction + spacing (distance from first click)
 */
export function handleSketchLinearPatternClick(pos: SketchPoint2D): void {
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

  // Find the nearest line to the first click
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

  // Direction = from first click to second click
  const dx = pos.x - firstClick.x;
  const dy = pos.y - firstClick.y;
  const spacing = Math.sqrt(dx * dx + dy * dy);
  if (spacing < 0.1) { store.clearPendingPoints(); return; }

  const dirX = dx / spacing;
  const dirY = dy / spacing;

  store.beginUndoBatch();

  // Create copies
  for (let i = 1; i < patternCount; i++) {
    const ox = dirX * spacing * i;
    const oy = dirY * spacing * i;

    const p1Id = store.genSketchEntityId();
    store.addSketchEntity({
      type: "point", id: p1Id,
      position: { x: endpoints.start.x + ox, y: endpoints.start.y + oy },
    });

    const p2Id = store.genSketchEntityId();
    store.addSketchEntity({
      type: "point", id: p2Id,
      position: { x: endpoints.end.x + ox, y: endpoints.end.y + oy },
    });

    const newLineId = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: newLineId, startId: p1Id, endId: p2Id });

    // Add equal-length constraint with original
    const eqId = store.genSketchConstraintId();
    store.addSketchConstraint({ id: eqId, kind: "equal", entityIds: [nearestLineId, newLineId] });
  }

  store.endUndoBatch();
  store.clearPendingPoints();
}
