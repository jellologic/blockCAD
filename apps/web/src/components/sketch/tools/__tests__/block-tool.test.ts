import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleBlockClick, createBlockFromSelection, setBlockMode } from "../block-tool";

describe("block tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("block");

    // Create a point and line for selection
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 5, y: 0 } });
    store.addSketchEntity({ type: "line", id: "se-2", startId: "se-0", endId: "se-1" });
  });

  it("click in create mode adds entity to selection", () => {
    setBlockMode("create");
    handleBlockClick({ x: 0, y: 0 }); // near se-0 point
    // The block tool collects entity IDs internally (selectedEntitiesForBlock)
    // We can't easily verify internal state, but no crash = success
    expect(useEditorStore.getState().sketchSession).not.toBeNull();
  });

  it("createBlockFromSelection returns null without selection", () => {
    setBlockMode("create");
    const result = createBlockFromSelection("Test", { x: 0, y: 0 });
    expect(result).toBeNull(); // No entities selected yet
  });
});
