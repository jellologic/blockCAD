import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleMeasureClick } from "../measure-tool";

describe("measure tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("measure");
  });

  it("first click sets pending point", () => {
    handleMeasureClick({ x: 0, y: 0 });
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(1);
  });

  it("second click sets measureResult with correct distance", () => {
    handleMeasureClick({ x: 0, y: 0 });
    handleMeasureClick({ x: 3, y: 4 });

    const session = useEditorStore.getState().sketchSession!;
    expect(session.measureResult).not.toBeNull();
    expect(session.measureResult!.distance).toBeCloseTo(5); // 3-4-5 triangle
    expect(session.measureResult!.from).toEqual({ x: 0, y: 0 });
    expect(session.measureResult!.to).toEqual({ x: 3, y: 4 });

    // Pending cleared after measurement
    expect(session.pendingPoints).toHaveLength(0);
  });
});
