import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleSlotClick } from "../slot-tool";

describe("slot tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("slot");
  });

  it("first click sets pending center", () => {
    handleSlotClick({ x: 0, y: 0 });
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(1);
  });

  it("second click creates full slot geometry", () => {
    handleSlotClick({ x: 0, y: 0 });
    handleSlotClick({ x: 10, y: 0 });

    const session = useEditorStore.getState().sketchSession!;
    const points = session.entities.filter(e => e.type === "point");
    const lines = session.entities.filter(e => e.type === "line");
    const arcs = session.entities.filter(e => e.type === "arc");

    // 4 corners + 2 centers = 6 points
    expect(points).toHaveLength(6);
    // 2 parallel side lines
    expect(lines).toHaveLength(2);
    // 2 semicircular end arcs
    expect(arcs).toHaveLength(2);
    // parallel constraint
    expect(session.constraints.some(c => c.kind === "parallel")).toBe(true);
  });

  it("too-close clicks rejected", () => {
    handleSlotClick({ x: 5, y: 5 });
    handleSlotClick({ x: 5.001, y: 5.001 });

    const session = useEditorStore.getState().sketchSession!;
    // Should have no slot entities (only distance < 0.01)
    const lines = session.entities.filter(e => e.type === "line");
    expect(lines).toHaveLength(0);
  });
});
