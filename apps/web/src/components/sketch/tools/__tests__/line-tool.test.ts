import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleLineClick } from "../line-tool";

describe("line tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("line");
  });

  it("first click creates a point and adds to pending", () => {
    handleLineClick({ x: 5, y: 10 });
    const session = useEditorStore.getState().sketchSession!;
    const points = session.entities.filter(e => e.type === "point");
    expect(points).toHaveLength(1);
    expect(session.pendingPoints).toHaveLength(1);
  });

  it("second click creates end point and line", () => {
    handleLineClick({ x: 0, y: 0 });
    handleLineClick({ x: 10, y: 5 });
    const session = useEditorStore.getState().sketchSession!;
    const points = session.entities.filter(e => e.type === "point");
    const lines = session.entities.filter(e => e.type === "line");
    expect(points).toHaveLength(2);
    expect(lines).toHaveLength(1);
  });

  it("chain mode continues from last point", () => {
    handleLineClick({ x: 0, y: 0 });
    handleLineClick({ x: 10, y: 0 });
    handleLineClick({ x: 10, y: 10 });
    const session = useEditorStore.getState().sketchSession!;
    const points = session.entities.filter(e => e.type === "point");
    const lines = session.entities.filter(e => e.type === "line");
    expect(points).toHaveLength(3);
    expect(lines).toHaveLength(2);
    // Should still have 1 pending point for chain continuation
    expect(session.pendingPoints).toHaveLength(1);
  });
});
