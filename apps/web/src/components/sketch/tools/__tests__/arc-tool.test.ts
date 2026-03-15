import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleArcClick } from "../arc-tool";

describe("arc tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("arc");
  });

  it("first two clicks set pending points", () => {
    handleArcClick({ x: 0, y: 0 });
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(1);
    handleArcClick({ x: 10, y: 0 });
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(2);
  });

  it("third click creates arc with center, start, end points", () => {
    handleArcClick({ x: 0, y: 0 });
    handleArcClick({ x: 10, y: 0 });
    handleArcClick({ x: 5, y: 5 }); // point on arc
    const session = useEditorStore.getState().sketchSession!;
    const points = session.entities.filter(e => e.type === "point");
    const arcs = session.entities.filter(e => e.type === "arc");
    expect(points).toHaveLength(3); // center, start, end
    expect(arcs).toHaveLength(1);
    if (arcs[0]!.type === "arc") {
      expect(arcs[0]!.radius).toBeGreaterThan(0);
    }
  });

  it("rejects collinear points", () => {
    handleArcClick({ x: 0, y: 0 });
    handleArcClick({ x: 10, y: 0 });
    handleArcClick({ x: 5, y: 0 }); // collinear!
    const session = useEditorStore.getState().sketchSession!;
    const arcs = session.entities.filter(e => e.type === "arc");
    expect(arcs).toHaveLength(0);
  });
});
