import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleCircleClick } from "../circle-tool";

describe("circle tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("circle");
  });

  it("first click sets center as pending", () => {
    handleCircleClick({ x: 5, y: 5 });
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(1);
  });

  it("second click creates center point + circle entity", () => {
    handleCircleClick({ x: 5, y: 5 });
    handleCircleClick({ x: 8, y: 9 }); // radius = 5
    const session = useEditorStore.getState().sketchSession!;
    const points = session.entities.filter(e => e.type === "point");
    const circles = session.entities.filter(e => e.type === "circle");
    expect(points).toHaveLength(1);
    expect(circles).toHaveLength(1);
    if (circles[0].type === "circle") {
      expect(circles[0].radius).toBeCloseTo(5, 1);
    }
  });

  it("rejects too-small radius", () => {
    handleCircleClick({ x: 5, y: 5 });
    handleCircleClick({ x: 5.01, y: 5.01 }); // very small
    const session = useEditorStore.getState().sketchSession!;
    // Should not create circle (radius < 0.1)
    const circles = session.entities.filter(e => e.type === "circle");
    expect(circles).toHaveLength(0);
  });
});
