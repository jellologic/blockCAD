import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";

export function handleCircleClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  if (session.pendingPoints.length === 0) {
    // First click: set center
    store.addPendingPoint(clickPos);
  } else {
    // Second click: compute radius and create circle
    const center = session.pendingPoints[0]!;
    const dx = clickPos.x - center.x;
    const dy = clickPos.y - center.y;
    const radius = Math.sqrt(dx * dx + dy * dy);

    if (radius < 0.1) return; // Too small

    store.beginUndoBatch();
    const centerId = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: centerId, position: center });

    const circleId = store.genSketchEntityId();
    store.addSketchEntity({ type: "circle", id: circleId, centerId, radius });

    store.clearPendingPoints();
    store.endUndoBatch();
  }
}
