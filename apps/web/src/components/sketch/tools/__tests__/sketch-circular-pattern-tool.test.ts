import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleSketchCircularPatternClick } from "../sketch-circular-pattern-tool";

describe("sketch circular pattern tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("sketch-circular-pattern");

    // Create a horizontal line
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 5, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 10, y: 0 } });
    store.addSketchEntity({ type: "line", id: "se-2", startId: "se-0", endId: "se-1" });
  });

  it("first click sets pending", () => {
    handleSketchCircularPatternClick({ x: 7.5, y: 0 });
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(1);
  });

  it("second click creates rotated copies with equal constraints", () => {
    handleSketchCircularPatternClick({ x: 7.5, y: 0 }); // select line
    handleSketchCircularPatternClick({ x: 0, y: 0 }); // center at origin

    const session = useEditorStore.getState().sketchSession!;
    const lines = session.entities.filter(e => e.type === "line");
    // Original 1 + 3 copies (DEFAULT_COUNT=4, so 4-1=3) = 4 lines
    expect(lines).toHaveLength(4);

    // 3 equal constraints
    const equalConstraints = session.constraints.filter(c => c.kind === "equal");
    expect(equalConstraints).toHaveLength(3);

    expect(session.pendingPoints).toHaveLength(0);
  });
});
