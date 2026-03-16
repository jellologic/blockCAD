import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleMirrorClick } from "../mirror-tool";

describe("mirror tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("mirror");

    // Create source line (horizontal at y=2) and axis line (vertical at x=5)
    const store = useEditorStore.getState();
    const p0 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p0, position: { x: 0, y: 2 } });
    const p1 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p1, position: { x: 4, y: 2 } });
    const l1 = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: l1, startId: p0, endId: p1 });

    const p2 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p2, position: { x: 5, y: 0 } });
    const p3 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p3, position: { x: 5, y: 10 } });
    const l2 = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: l2, startId: p2, endId: p3 });
  });

  it("first click sets pending point", () => {
    handleMirrorClick({ x: 2, y: 2 }); // near source line
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(1);
  });

  it("second click creates mirrored line with symmetric constraints", () => {
    handleMirrorClick({ x: 2, y: 2 }); // near source line
    handleMirrorClick({ x: 5, y: 5 }); // near axis line

    const session = useEditorStore.getState().sketchSession!;
    const points = session.entities.filter(e => e.type === "point");
    const lines = session.entities.filter(e => e.type === "line");
    expect(points.length).toBeGreaterThanOrEqual(6); // 4 original + 2 mirrored
    expect(lines.length).toBeGreaterThanOrEqual(3); // 2 original + 1 mirrored

    // Should have symmetric constraints
    const symmetricConstraints = session.constraints.filter(c => c.kind === "symmetric");
    expect(symmetricConstraints).toHaveLength(2);
  });

  it("pending cleared after mirror completes", () => {
    handleMirrorClick({ x: 2, y: 2 });
    handleMirrorClick({ x: 5, y: 5 });
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(0);
  });
});
