import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleTrimClick } from "../trim-tool";

describe("trim tool", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("trim");

    // Create two crossing lines
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 5 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 10, y: 5 } });
    store.addSketchEntity({ type: "line", id: "se-2", startId: "se-0", endId: "se-1" });
    store.addSketchEntity({ type: "point", id: "se-3", position: { x: 5, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-4", position: { x: 5, y: 10 } });
    store.addSketchEntity({ type: "line", id: "se-5", startId: "se-3", endId: "se-4" });
  });

  it("click near intersection trims the line", () => {
    const beforeCount = useEditorStore.getState().sketchSession!.entities.length;
    // Click on the horizontal line near the intersection at (5, 5)
    handleTrimClick({ x: 6, y: 5 });
    const session = useEditorStore.getState().sketchSession!;
    const lines = session.entities.filter(e => e.type === "line");
    // The original horizontal line should be removed and replaced with shorter segments
    expect(lines.find(l => l.id === "se-2")).toBeUndefined();
    // Entity count should have changed (line removed, new segments added)
    expect(session.entities.length).not.toBe(beforeCount);
  });

  it("click far from any line does nothing", () => {
    const beforeCount = useEditorStore.getState().sketchSession!.entities.length;
    handleTrimClick({ x: 50, y: 50 });
    const session = useEditorStore.getState().sketchSession!;
    expect(session.entities.length).toBe(beforeCount);
  });
});
