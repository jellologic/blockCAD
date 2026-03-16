import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";

describe("editor store - sketch mode", () => {
  beforeEach(async () => {
    // Reset store and init kernel
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
  });

  it("enterSketchMode creates session and solver", () => {
    useEditorStore.getState().enterSketchMode("front");
    const state = useEditorStore.getState();
    expect(state.mode).toBe("sketch");
    expect(state.sketchSession).not.toBeNull();
    expect(state.sketchSolver).not.toBeNull();
    expect(state.sketchSession!.planeId).toBe("front");
    expect(state.sketchSession!.entities).toEqual([]);
    expect(state.sketchSession!.constraints).toEqual([]);
    expect(state.sketchSession!.activeTool).toBeNull();
  });

  it("enterSketchMode creates session with top plane", () => {
    useEditorStore.getState().enterSketchMode("top");
    const session = useEditorStore.getState().sketchSession!;
    expect(session.planeId).toBe("top");
    expect(session.plane.normal).toEqual([0, 1, 0]);
  });

  it("enterSketchMode clears active operation", () => {
    useEditorStore.getState().startOperation("extrude");
    expect(useEditorStore.getState().activeOperation).not.toBeNull();
    useEditorStore.getState().enterSketchMode("front");
    expect(useEditorStore.getState().activeOperation).toBeNull();
  });

  it("exitSketchMode(true) saves sketch feature", () => {
    useEditorStore.getState().enterSketchMode("front");
    // Add a closed rectangle (4 points + 4 lines)
    useEditorStore.getState().addSketchEntity({
      type: "point", id: "se-0", position: { x: 0, y: 0 }
    });
    useEditorStore.getState().addSketchEntity({
      type: "point", id: "se-1", position: { x: 10, y: 0 }
    });
    useEditorStore.getState().addSketchEntity({
      type: "point", id: "se-2", position: { x: 10, y: 5 }
    });
    useEditorStore.getState().addSketchEntity({
      type: "point", id: "se-3", position: { x: 0, y: 5 }
    });
    useEditorStore.getState().addSketchEntity({
      type: "line", id: "se-4", startId: "se-0", endId: "se-1"
    });
    useEditorStore.getState().addSketchEntity({
      type: "line", id: "se-5", startId: "se-1", endId: "se-2"
    });
    useEditorStore.getState().addSketchEntity({
      type: "line", id: "se-6", startId: "se-2", endId: "se-3"
    });
    useEditorStore.getState().addSketchEntity({
      type: "line", id: "se-7", startId: "se-3", endId: "se-0"
    });

    const featuresBefore = useEditorStore.getState().features.length;
    useEditorStore.getState().exitSketchMode(true);

    expect(useEditorStore.getState().mode).toBe("view");
    expect(useEditorStore.getState().sketchSession).toBeNull();
    expect(useEditorStore.getState().features.length).toBe(featuresBefore + 1);
    // Last feature should be a sketch
    const features = useEditorStore.getState().features;
    expect(features[features.length - 1].type).toBe("sketch");
  });

  it("exitSketchMode(false) discards without saving", () => {
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().addSketchEntity({
      type: "point", id: "se-0", position: { x: 5, y: 5 }
    });

    const featuresBefore = useEditorStore.getState().features.length;
    useEditorStore.getState().exitSketchMode(false);

    expect(useEditorStore.getState().mode).toBe("view");
    expect(useEditorStore.getState().sketchSession).toBeNull();
    expect(useEditorStore.getState().features.length).toBe(featuresBefore);
  });

  it("setSketchTool activates tool and clears pending", () => {
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().addPendingPoint({ x: 1, y: 2 });
    useEditorStore.getState().setSketchTool("line");

    const session = useEditorStore.getState().sketchSession!;
    expect(session.activeTool).toBe("line");
    expect(session.pendingPoints).toEqual([]);
  });

  it("addSketchEntity pushes entity to session", () => {
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().addSketchEntity({
      type: "point", id: "se-0", position: { x: 3, y: 4 }
    });

    const session = useEditorStore.getState().sketchSession!;
    expect(session.entities).toHaveLength(1);
    expect(session.entities[0].type).toBe("point");
  });

  it("addSketchConstraint pushes constraint to session", () => {
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().addSketchConstraint({
      id: "sc-0", kind: "horizontal", entityIds: ["se-0"]
    });

    const session = useEditorStore.getState().sketchSession!;
    expect(session.constraints).toHaveLength(1);
    expect(session.constraints[0].kind).toBe("horizontal");
  });

  it("genSketchEntityId returns incrementing IDs", () => {
    useEditorStore.getState().enterSketchMode("front");
    const id1 = useEditorStore.getState().genSketchEntityId();
    const id2 = useEditorStore.getState().genSketchEntityId();
    const id3 = useEditorStore.getState().genSketchEntityId();
    expect(id1).toBe("se-0");
    expect(id2).toBe("se-1");
    expect(id3).toBe("se-2");
  });

  it("addSketchConstraint triggers solver and updates DOF", () => {
    useEditorStore.getState().enterSketchMode("front");
    // Add two points
    useEditorStore.getState().addSketchEntity({
      type: "point", id: "se-0", position: { x: 0, y: 0 }
    });
    useEditorStore.getState().addSketchEntity({
      type: "point", id: "se-1", position: { x: 8, y: 0.5 }
    });
    useEditorStore.getState().addSketchEntity({
      type: "line", id: "se-2", startId: "se-0", endId: "se-1"
    });

    // Add a horizontal constraint — should trigger solve
    useEditorStore.getState().addSketchConstraint({
      id: "sc-0", kind: "horizontal", entityIds: ["se-2"]
    });

    const session = useEditorStore.getState().sketchSession!;
    // After solving, the second point's y should be close to the first point's y (horizontal)
    const p1 = session.entities.find(e => e.id === "se-1");
    if (p1?.type === "point") {
      expect(Math.abs(p1.position.y)).toBeLessThan(0.5);
    }
    // DOF status should be set
    expect(useEditorStore.getState().sketchDofStatus).not.toBeNull();
  });

  it("exitSketchMode disposes solver", () => {
    useEditorStore.getState().enterSketchMode("front");
    expect(useEditorStore.getState().sketchSolver).not.toBeNull();
    useEditorStore.getState().exitSketchMode(false);
    expect(useEditorStore.getState().sketchSolver).toBeNull();
  });

  it("addPendingPoint and clearPendingPoints work", () => {
    useEditorStore.getState().enterSketchMode("front");
    useEditorStore.getState().addPendingPoint({ x: 1, y: 2 });
    useEditorStore.getState().addPendingPoint({ x: 3, y: 4 });
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(2);

    useEditorStore.getState().clearPendingPoints();
    expect(useEditorStore.getState().sketchSession!.pendingPoints).toHaveLength(0);
  });
});

describe("editor store - delete and undo/redo", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
      sketchHistory: [], sketchRedoStack: [], sketchUndoBatching: false,
    });
    await useEditorStore.getState().initKernel();
    useEditorStore.getState().enterSketchMode("front");
  });

  // deleteSelectedEntities tests
  it("deleting entity removes it from session", () => {
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 1, y: 2 } });
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(1);

    store.deleteSelectedEntities(["se-0"]);
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(0);
  });

  it("deleting line removes orphaned constraints", () => {
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 10, y: 0 } });
    store.addSketchEntity({ type: "line", id: "se-2", startId: "se-0", endId: "se-1" });
    store.addSketchConstraint({ id: "sc-0", kind: "horizontal", entityIds: ["se-2"] });

    expect(useEditorStore.getState().sketchSession!.constraints).toHaveLength(1);
    store.deleteSelectedEntities(["se-2"]);
    expect(useEditorStore.getState().sketchSession!.constraints).toHaveLength(0);
  });

  it("delete with empty IDs is no-op", () => {
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.deleteSelectedEntities([]);
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(1);
  });

  // undoSketch tests
  it("undo restores previous entity state", () => {
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 5, y: 5 } });

    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(2);
    store.undoSketch();
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(1);
  });

  it("redo restores undone state", () => {
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 5, y: 5 } });

    store.undoSketch();
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(1);
    store.redoSketch();
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(2);
  });

  it("multiple undo/redo cycle", () => {
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 1, y: 1 } });
    store.addSketchEntity({ type: "point", id: "se-2", position: { x: 2, y: 2 } });

    // 3 entities -> undo -> 2 -> undo -> 1
    store.undoSketch();
    store.undoSketch();
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(1);

    // redo -> 2
    store.redoSketch();
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(2);
  });

  it("undo when history empty is no-op", () => {
    // Manually clear history that may leak from prior tests
    useEditorStore.setState({ sketchHistory: [], sketchRedoStack: [] });
    const session = useEditorStore.getState().sketchSession!;
    const countBefore = session.entities.length;
    useEditorStore.getState().undoSketch();
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(countBefore);
  });

  it("redo when stack empty is no-op", () => {
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    const entitiesAfter = useEditorStore.getState().sketchSession!.entities.length;
    store.redoSketch(); // nothing to redo
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(entitiesAfter);
  });

  // Undo batching
  it("batched operations undo as single step", () => {
    const store = useEditorStore.getState();

    store.beginUndoBatch();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 1, y: 1 } });
    store.addSketchEntity({ type: "point", id: "se-2", position: { x: 2, y: 2 } });
    store.endUndoBatch();

    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(3);

    // ONE undo should remove ALL 3 entities (the batch)
    store.undoSketch();
    expect(useEditorStore.getState().sketchSession!.entities).toHaveLength(0);
  });
});
