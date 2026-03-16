import type { SketchPoint2D, SketchEntity2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";

function getPointPos(id: string, entities: SketchEntity2D[]): SketchPoint2D | null {
  const pt = entities.find((e) => e.type === "point" && e.id === id);
  return pt?.type === "point" ? pt.position : null;
}

/** Find the nearest entity to a click position. Returns entity and distance. */
function findNearestEntity(
  clickPos: SketchPoint2D,
  entities: SketchEntity2D[]
): { entity: SketchEntity2D; dist: number } | null {
  let best: { entity: SketchEntity2D; dist: number } | null = null;

  for (const entity of entities) {
    let dist = Infinity;

    if (entity.type === "line") {
      const startPt = getPointPos(entity.startId, entities);
      const endPt = getPointPos(entity.endId, entities);
      if (startPt && endPt) {
        // Distance to line midpoint
        const mx = (startPt.x + endPt.x) / 2;
        const my = (startPt.y + endPt.y) / 2;
        dist = Math.sqrt((clickPos.x - mx) ** 2 + (clickPos.y - my) ** 2);
      }
    } else if (entity.type === "circle") {
      const centerPt = getPointPos(entity.centerId, entities);
      if (centerPt) {
        // Distance to circle edge
        const dx = clickPos.x - centerPt.x;
        const dy = clickPos.y - centerPt.y;
        dist = Math.abs(Math.sqrt(dx * dx + dy * dy) - entity.radius);
      }
    } else if (entity.type === "point") {
      dist = Math.sqrt(
        (clickPos.x - entity.position.x) ** 2 +
        (clickPos.y - entity.position.y) ** 2
      );
    }

    if (dist < (best?.dist ?? Infinity)) {
      best = { entity, dist };
    }
  }

  return best && best.dist < 5 ? best : null;
}

/**
 * Handle a click in dimension mode.
 *
 * Flow:
 * 1. First click → detect entity → set dimensionPending
 * 2. If dimensionPending exists and user clicks again:
 *    - If clicking another entity → multi-entity dimension (angle, point-point distance)
 *    - If clicking empty space → placement click → show dimension input
 */
export function handleDimensionClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const pending = session.dimensionPending;

  if (!pending) {
    // FIRST CLICK: detect what to dimension
    const hit = findNearestEntity(clickPos, session.entities);
    if (!hit) return; // Nothing near cursor

    if (hit.entity.type === "line") {
      // Line → distance dimension. Set pending, wait for placement click.
      store.setDimensionPending({
        entityIds: [hit.entity.startId, hit.entity.endId],
        kind: "distance",
      });
    } else if (hit.entity.type === "circle") {
      // Circle → radius dimension
      store.setDimensionPending({
        entityIds: [hit.entity.id],
        kind: "radius",
      });
    } else if (hit.entity.type === "point") {
      // Point → wait for second click (another point or placement)
      store.setDimensionPending({
        entityIds: [hit.entity.id],
        kind: "distance",
      });
    }
  } else {
    // SECOND+ CLICK
    const hit = findNearestEntity(clickPos, session.entities);

    if (hit && pending.entityIds.length === 1) {
      // Second entity click — multi-entity dimension
      const firstEntity = session.entities.find((e) => e.id === pending.entityIds[0]);

      if (hit.entity.type === "line" && firstEntity?.type === "line") {
        // Two lines → angle dimension
        store.showDimensionInput(clickPos, [firstEntity.id, hit.entity.id], "angle");
        store.setDimensionPending(null);
        return;
      }

      if (hit.entity.type === "point" && firstEntity?.type === "point") {
        // Two points → distance between
        store.setDimensionPending({
          entityIds: [pending.entityIds[0]!, hit.entity.id],
          kind: "distance",
        });
        // Now wait for placement click
        return;
      }
    }

    // Placement click — show dimension input at this position
    store.showDimensionInput(clickPos, pending.entityIds, pending.kind);
    store.setDimensionPending(null);
  }
}
