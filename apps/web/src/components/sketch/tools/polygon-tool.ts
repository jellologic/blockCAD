import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";

/** Default number of sides for polygon tool */
const DEFAULT_SIDES = 6;

/**
 * Polygon tool: creates a regular N-sided polygon.
 *
 * Step 1: Click → center point
 * Step 2: Click → sets radius (distance from center to vertex)
 *
 * Generates N points + N lines + equal-length constraints.
 */
export function handlePolygonClick(pos: SketchPoint2D, sides: number = DEFAULT_SIDES): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const pending = session.pendingPoints;

  if (pending.length === 0) {
    // First click: set center
    store.addPendingPoint(pos);
    return;
  }

  // Second click: compute radius and generate polygon
  const center = pending[0]!;
  const dx = pos.x - center.x;
  const dy = pos.y - center.y;
  const radius = Math.sqrt(dx * dx + dy * dy);

  if (radius < 0.01) {
    store.clearPendingPoints();
    return;
  }

  const n = Math.max(3, Math.min(sides, 100)); // Clamp sides 3-100
  const startAngle = Math.atan2(dy, dx);

  store.beginUndoBatch();

  // Generate N vertex points
  const pointIds: string[] = [];
  for (let i = 0; i < n; i++) {
    const angle = startAngle + (2 * Math.PI * i) / n;
    const px = center.x + radius * Math.cos(angle);
    const py = center.y + radius * Math.sin(angle);
    const ptId = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: ptId, position: { x: px, y: py } });
    pointIds.push(ptId);
  }

  // Generate N lines connecting adjacent points
  const lineIds: string[] = [];
  for (let i = 0; i < n; i++) {
    const j = (i + 1) % n;
    const lineId = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: lineId, startId: pointIds[i]!, endId: pointIds[j]! });
    lineIds.push(lineId);
  }

  // Add equal-length constraints between all edges
  if (lineIds.length > 1) {
    for (let i = 1; i < lineIds.length; i++) {
      const cId = store.genSketchConstraintId();
      store.addSketchConstraint({
        id: cId,
        kind: "equal",
        entityIds: [lineIds[0]!, lineIds[i]!],
      });
    }
  }

  // Fix the first point to anchor the polygon
  const fixId = store.genSketchConstraintId();
  store.addSketchConstraint({
    id: fixId,
    kind: "fixed",
    entityIds: [pointIds[0]!],
  });

  store.endUndoBatch();
  store.clearPendingPoints();
}
