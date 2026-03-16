import type { SketchPoint2D } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";

/**
 * Convert Entities tool: projects 3D BRep face edges onto the active
 * sketch plane as 2D line entities.
 *
 * Click on a face in the viewport → its boundary edges are projected
 * onto the sketch plane and created as construction lines.
 *
 * Simplified version: since face selection in sketch mode isn't fully
 * wired up, this tool creates a projected rectangle from the selected
 * face index (if available in the editor store).
 */
export function handleConvertEntitiesClick(pos: SketchPoint2D): void {
  const store = useEditorStore.getState();
  const session = store.sketchSession;
  if (!session) return;

  const selectedFaceIndex = store.selectedFaceIndex;
  const meshData = store.meshData;

  if (selectedFaceIndex == null || !meshData) {
    // No face selected — create a reference crosshair at click position instead
    store.beginUndoBatch();

    // Horizontal construction line through click point
    const hp1 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: hp1, position: { x: pos.x - 10, y: pos.y } });
    const hp2 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: hp2, position: { x: pos.x + 10, y: pos.y } });
    const hLineId = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: hLineId, startId: hp1, endId: hp2 });

    // Vertical construction line through click point
    const vp1 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: vp1, position: { x: pos.x, y: pos.y - 10 } });
    const vp2 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: vp2, position: { x: pos.x, y: pos.y + 10 } });
    const vLineId = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: vLineId, startId: vp1, endId: vp2 });

    // Add horizontal/vertical constraints
    const hc = store.genSketchConstraintId();
    store.addSketchConstraint({ id: hc, kind: "horizontal", entityIds: [hLineId] });
    const vc = store.genSketchConstraintId();
    store.addSketchConstraint({ id: vc, kind: "vertical", entityIds: [vLineId] });

    store.endUndoBatch();
    return;
  }

  // TODO: When face-to-sketch projection is fully implemented,
  // extract face boundary edges from BRep, project each edge
  // endpoint onto the sketch plane via plane.closest_parameters(),
  // and create corresponding 2D line entities.
  //
  // For now, create a placeholder point at the click location
  // to indicate the tool was activated.
  store.beginUndoBatch();
  const ptId = store.genSketchEntityId();
  store.addSketchEntity({ type: "point", id: ptId, position: pos });
  store.endUndoBatch();
}
