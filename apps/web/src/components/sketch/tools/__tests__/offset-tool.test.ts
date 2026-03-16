import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleOffsetClick } from "../offset-tool";

describe("offset tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("offset");

    // Create a horizontal line to offset using generated IDs
    const store = useEditorStore.getState();
    const p0 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p0, position: { x: 0, y: 0 } });
    const p1 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p1, position: { x: 10, y: 0 } });
    const l1 = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: l1, startId: p0, endId: p1 });
  });

  it("first click selects line (adds pending)", () => {
    handleOffsetClick({ x: 5, y: 0 }); // on the line
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(1);
  });

  it("second click creates offset line with parallel constraint", () => {
    handleOffsetClick({ x: 5, y: 0 }); // select line
    handleOffsetClick({ x: 5, y: 3 }); // offset to y=3

    const session = useEditorStore.getState().sketchSession!;
    const points = session.entities.filter(e => e.type === "point");
    const lines = session.entities.filter(e => e.type === "line");

    // Original: 2 points + 1 line. New: 2 points + 1 line = total 4 points, 2 lines
    expect(points).toHaveLength(4);
    expect(lines).toHaveLength(2);

    // Should have a parallel constraint
    expect(session.constraints.some(c => c.kind === "parallel")).toBe(true);
  });
});
