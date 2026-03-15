import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";

export function handleRectangleClick(clickPos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  if (session.pendingPoints.length === 0) {
    store.addPendingPoint(clickPos);
  } else {
    const corner1 = session.pendingPoints[0]!;
    const corner2 = clickPos;

    // Create 4 corner points
    const p0Id = store.genSketchEntityId();
    const p1Id = store.genSketchEntityId();
    const p2Id = store.genSketchEntityId();
    const p3Id = store.genSketchEntityId();

    store.addSketchEntity({
      type: "point",
      id: p0Id,
      position: { x: corner1.x, y: corner1.y },
    });
    store.addSketchEntity({
      type: "point",
      id: p1Id,
      position: { x: corner2.x, y: corner1.y },
    });
    store.addSketchEntity({
      type: "point",
      id: p2Id,
      position: { x: corner2.x, y: corner2.y },
    });
    store.addSketchEntity({
      type: "point",
      id: p3Id,
      position: { x: corner1.x, y: corner2.y },
    });

    // Create 4 lines
    const l0Id = store.genSketchEntityId();
    const l1Id = store.genSketchEntityId();
    const l2Id = store.genSketchEntityId();
    const l3Id = store.genSketchEntityId();

    store.addSketchEntity({ type: "line", id: l0Id, startId: p0Id, endId: p1Id });
    store.addSketchEntity({ type: "line", id: l1Id, startId: p1Id, endId: p2Id });
    store.addSketchEntity({ type: "line", id: l2Id, startId: p2Id, endId: p3Id });
    store.addSketchEntity({ type: "line", id: l3Id, startId: p3Id, endId: p0Id });

    // Add horizontal/vertical constraints
    store.addSketchConstraint({
      id: store.genSketchConstraintId(),
      kind: "horizontal",
      entityIds: [l0Id],
    });
    store.addSketchConstraint({
      id: store.genSketchConstraintId(),
      kind: "horizontal",
      entityIds: [l2Id],
    });
    store.addSketchConstraint({
      id: store.genSketchConstraintId(),
      kind: "vertical",
      entityIds: [l1Id],
    });
    store.addSketchConstraint({
      id: store.genSketchConstraintId(),
      kind: "vertical",
      entityIds: [l3Id],
    });

    store.clearPendingPoints();
    store.setSketchTool(null);
  }
}
