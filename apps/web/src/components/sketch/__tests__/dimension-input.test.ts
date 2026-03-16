import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";

describe("dimension input - store integration", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
  });

  it("confirmDimension creates distance constraint for 2 points", () => {
    const store = useEditorStore.getState();
    // Add two points
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 10, y: 0 } });

    // Show dimension input for distance between the two points
    store.showDimensionInput({ x: 5, y: 2 }, ["se-0", "se-1"], "distance");

    // Verify dimension input is shown
    expect(useEditorStore.getState().sketchSession!.dimensionInput).not.toBeNull();

    // Confirm with value 10
    store.confirmDimension(10);

    // Verify constraint was created
    const session = useEditorStore.getState().sketchSession!;
    expect(session.constraints.length).toBeGreaterThanOrEqual(1);
    const distConstraint = session.constraints.find(c => c.kind === "distance");
    expect(distConstraint).toBeDefined();
    expect(distConstraint!.value).toBe(10);

    // Dimension input should be cleared
    expect(session.dimensionInput).toBeNull();
  });

  it("confirmDimension creates radius constraint for circle", () => {
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "circle", id: "se-1", centerId: "se-0", radius: 5 });

    store.showDimensionInput({ x: 5, y: 0 }, ["se-1"], "radius");
    store.confirmDimension(8);

    const session = useEditorStore.getState().sketchSession!;
    const radiusConstraint = session.constraints.find(c => c.kind === "radius");
    expect(radiusConstraint).toBeDefined();
    expect(radiusConstraint!.value).toBe(8);
  });

  it("cancelDimension clears dimension state", () => {
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 10, y: 0 } });

    store.showDimensionInput({ x: 5, y: 2 }, ["se-0", "se-1"], "distance");
    expect(useEditorStore.getState().sketchSession!.dimensionInput).not.toBeNull();

    store.cancelDimension();
    expect(useEditorStore.getState().sketchSession!.dimensionInput).toBeNull();

    // No constraint should have been created
    expect(useEditorStore.getState().sketchSession!.constraints).toHaveLength(0);
  });
});
