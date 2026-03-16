import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handlePolygonClick } from "../polygon-tool";

describe("polygon tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("polygon");
  });

  it("first click sets center as pending", () => {
    handlePolygonClick({ x: 5, y: 5 });
    const session = useEditorStore.getState().sketchSession!;
    expect(session.pendingPoints).toHaveLength(1);
  });

  it("second click creates polygon with default sides", () => {
    handlePolygonClick({ x: 0, y: 0 });
    handlePolygonClick({ x: 5, y: 0 });
    const session = useEditorStore.getState().sketchSession!;
    const points = session.entities.filter(e => e.type === "point");
    const lines = session.entities.filter(e => e.type === "line");
    // Default sides = 6
    expect(points).toHaveLength(6);
    expect(lines).toHaveLength(6);
  });

  it("polygon has equal-length constraints", () => {
    handlePolygonClick({ x: 0, y: 0 });
    handlePolygonClick({ x: 5, y: 0 });
    const session = useEditorStore.getState().sketchSession!;
    const equalConstraints = session.constraints.filter(c => c.kind === "equal");
    expect(equalConstraints.length).toBeGreaterThanOrEqual(1);
  });
});
