import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleSketchLinearPatternClick } from "../sketch-linear-pattern-tool";

describe("sketch linear pattern tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("sketch-linear-pattern");

    // Create a horizontal line
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 5, y: 0 } });
    store.addSketchEntity({ type: "line", id: "se-2", startId: "se-0", endId: "se-1" });
  });

  it("first click sets pending", () => {
    handleSketchLinearPatternClick({ x: 2.5, y: 0 }); // near line midpoint
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(1);
  });

  it("second click creates pattern copies with equal constraints", () => {
    handleSketchLinearPatternClick({ x: 2.5, y: 0 }); // select line
    handleSketchLinearPatternClick({ x: 2.5, y: 5 }); // direction: upward, spacing: 5

    const session = useEditorStore.getState().sketchSession!;
    const lines = session.entities.filter(e => e.type === "line");
    // Original 1 line + 2 copies (DEFAULT_COUNT=3, so 3-1=2 copies) = 3 lines
    expect(lines).toHaveLength(3);

    // 2 equal constraints (one per copy)
    const equalConstraints = session.constraints.filter(c => c.kind === "equal");
    expect(equalConstraints).toHaveLength(2);

    // Pending cleared
    expect(session.pendingPoints).toHaveLength(0);
  });
});
