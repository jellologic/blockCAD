import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import { handleExtendClick } from "../extend-tool";

describe("extend tool", () => {
  let line1Id: string;

  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().setSketchTool("extend");

    // Line 1: horizontal from (0,0) to (3,0) -- needs extending to reach line 2
    const store = useEditorStore.getState();
    const p0 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p0, position: { x: 0, y: 0 } });
    const p1 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p1, position: { x: 3, y: 0 } });
    line1Id = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: line1Id, startId: p0, endId: p1 });

    // Line 2: vertical at x=5 from (5,-5) to (5,5)
    const p2 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p2, position: { x: 5, y: -5 } });
    const p3 = store.genSketchEntityId();
    store.addSketchEntity({ type: "point", id: p3, position: { x: 5, y: 5 } });
    const l2 = store.genSketchEntityId();
    store.addSketchEntity({ type: "line", id: l2, startId: p2, endId: p3 });
  });

  it("click near endpoint extends line to intersection", () => {
    const entitiesBefore = useEditorStore.getState().sketchSession!.entities.length;
    // Click near the end of line 1 (at x=3)
    handleExtendClick({ x: 3, y: 0 });

    const session = useEditorStore.getState().sketchSession!;
    // Original line should be replaced with extended version
    expect(session.entities.find(e => e.id === line1Id)).toBeUndefined();

    // Entity count should have changed (new point + new line added, original line removed)
    expect(session.entities.length).not.toBe(entitiesBefore);
  });

  it("click far from any line does nothing", () => {
    const entitiesBefore = useEditorStore.getState().sketchSession!.entities.length;
    handleExtendClick({ x: 50, y: 50 });
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(entitiesBefore);
  });
});
