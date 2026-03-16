import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleDimensionClick } from "../dimension-tool";

describe("dimension tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");

    // Create a line to dimension
    useEditorStore.getState().addSketchEntity({ type: "point", id: "p0", position: { x: 0, y: 0 } });
    useEditorStore.getState().addSketchEntity({ type: "point", id: "p1", position: { x: 10, y: 0 } });
    useEditorStore.getState().addSketchEntity({ type: "line", id: "l0", startId: "p0", endId: "p1" });

    useEditorStore.getState().setSketchTool("dimension");
  });

  it("clicking near a line sets dimensionPending, second click shows input", () => {
    // First click: detect entity → sets pending
    handleDimensionClick({ x: 5, y: 0.5 });
    const session1 = useEditorStore.getState().sketchSession!;
    expect(session1.dimensionPending).not.toBeNull();
    expect(session1.dimensionPending!.kind).toBe("distance");

    // Second click: placement → shows input
    handleDimensionClick({ x: 5, y: 3 });
    const session2 = useEditorStore.getState().sketchSession!;
    expect(session2.dimensionInput).not.toBeNull();
    expect(session2.dimensionInput!.kind).toBe("distance");
  });

  it("clicking far from line does not set pending", () => {
    handleDimensionClick({ x: 50, y: 50 }); // far away
    const session = useEditorStore.getState().sketchSession!;
    expect(session.dimensionPending).toBeNull();
    expect(session.dimensionInput).toBeNull();
  });

  it("confirmDimension creates constraint with value", () => {
    // First click: entity detection
    handleDimensionClick({ x: 5, y: 0.5 });
    // Second click: placement
    handleDimensionClick({ x: 5, y: 3 });
    // Confirm with value
    useEditorStore.getState().confirmDimension(25);
    const session = useEditorStore.getState().sketchSession!;
    expect(session.dimensionInput).toBeNull();
    expect(session.constraints).toHaveLength(1);
    expect(session.constraints[0]!.value).toBe(25);
    expect(session.constraints[0]!.kind).toBe("distance");
  });
});
