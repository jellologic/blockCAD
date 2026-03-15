import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleDimensionClick } from "../dimension-tool";

describe("dimension tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");

    // Create a line to dimension
    useEditorStore.getState().addSketchEntity({ type: "point", id: "p0", position: { x: 0, y: 0 } });
    useEditorStore.getState().addSketchEntity({ type: "point", id: "p1", position: { x: 10, y: 0 } });
    useEditorStore.getState().addSketchEntity({ type: "line", id: "l0", startId: "p0", endId: "p1" });

    useEditorStore.getState().setSketchTool("dimension");
  });

  it("clicking near a line shows dimension input", () => {
    handleDimensionClick({ x: 5, y: 0.5 }); // near the line midpoint
    const session = useEditorStore.getState().sketchSession!;
    expect(session.dimensionInput).not.toBeNull();
    expect(session.dimensionInput!.kind).toBe("distance");
    expect(session.dimensionInput!.entityIds).toEqual(["p0", "p1"]);
  });

  it("clicking far from line does not show input", () => {
    handleDimensionClick({ x: 50, y: 50 }); // far away
    const session = useEditorStore.getState().sketchSession!;
    expect(session.dimensionInput).toBeNull();
  });

  it("confirmDimension creates constraint with value", () => {
    handleDimensionClick({ x: 5, y: 0.5 });
    useEditorStore.getState().confirmDimension(25);
    const session = useEditorStore.getState().sketchSession!;
    expect(session.dimensionInput).toBeNull();
    expect(session.constraints).toHaveLength(1);
    expect(session.constraints[0]!.value).toBe(25);
    expect(session.constraints[0]!.kind).toBe("distance");
  });
});
