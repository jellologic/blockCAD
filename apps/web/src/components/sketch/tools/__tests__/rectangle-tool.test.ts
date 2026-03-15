import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleRectangleClick } from "../rectangle-tool";

describe("rectangle tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("rectangle");
  });

  it("first click sets pending point", () => {
    handleRectangleClick({ x: 0, y: 0 });
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(1);
  });

  it("second click creates 4 points + 4 lines + 4 constraints", () => {
    handleRectangleClick({ x: 0, y: 0 });
    handleRectangleClick({ x: 10, y: 5 });
    const session = useEditorStore.getState().sketchSession!;
    const points = session.entities.filter(e => e.type === "point");
    const lines = session.entities.filter(e => e.type === "line");
    expect(points).toHaveLength(4);
    expect(lines).toHaveLength(4);
    expect(session.constraints).toHaveLength(4);
    // Verify constraint types
    const hConstraints = session.constraints.filter(c => c.kind === "horizontal");
    const vConstraints = session.constraints.filter(c => c.kind === "vertical");
    expect(hConstraints).toHaveLength(2);
    expect(vConstraints).toHaveLength(2);
  });

  it("deactivates tool after rectangle creation", () => {
    handleRectangleClick({ x: 0, y: 0 });
    handleRectangleClick({ x: 10, y: 5 });
    expect(useEditorStore.getState().sketchSession!.activeTool).toBeNull();
  });
});
