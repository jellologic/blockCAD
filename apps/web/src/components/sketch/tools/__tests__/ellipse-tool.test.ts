import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleEllipseClick } from "../ellipse-tool";

describe("ellipse tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("ellipse");
  });

  it("first click sets center as pending", () => {
    handleEllipseClick({ x: 5, y: 5 });
    const session = useEditorStore.getState().sketchSession!;
    expect(session.pendingPoints).toHaveLength(1);
  });

  it("second click creates circle approximation", () => {
    handleEllipseClick({ x: 0, y: 0 });
    handleEllipseClick({ x: 4, y: 3 });
    const session = useEditorStore.getState().sketchSession!;
    const circles = session.entities.filter(e => e.type === "circle");
    const points = session.entities.filter(e => e.type === "point");
    expect(circles).toHaveLength(1);
    // Center point entity
    expect(points).toHaveLength(1);
  });
});
