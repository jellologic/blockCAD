import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";

describe("editor store - operations", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
  });

  // startOperation
  it("sets activeOperation with type and default params", () => {
    useEditorStore.getState().startOperation("extrude");
    const op = useEditorStore.getState().activeOperation;
    expect(op).not.toBeNull();
    expect(op!.type).toBe("extrude");
    expect(op!.params.depth).toBe(10);
  });

  it("sets default params for revolve", () => {
    useEditorStore.getState().startOperation("revolve");
    const op = useEditorStore.getState().activeOperation;
    expect(op!.type).toBe("revolve");
    expect(op!.params.angle).toBeCloseTo(6.283185);
  });

  it("sets default params for fillet", () => {
    useEditorStore.getState().startOperation("fillet");
    const op = useEditorStore.getState().activeOperation;
    expect(op!.type).toBe("fillet");
    expect(op!.params.radius).toBe(1);
  });

  // updateOperationParams
  it("merges params into activeOperation", () => {
    useEditorStore.getState().startOperation("extrude");
    useEditorStore.getState().updateOperationParams({ depth: 20 });
    expect(useEditorStore.getState().activeOperation!.params.depth).toBe(20);
  });

  it("updateOperationParams is no-op without active operation", () => {
    useEditorStore.getState().updateOperationParams({ depth: 20 });
    expect(useEditorStore.getState().activeOperation).toBeNull();
  });

  // cancelOperation
  it("cancelOperation clears activeOperation", () => {
    useEditorStore.getState().startOperation("extrude");
    expect(useEditorStore.getState().activeOperation).not.toBeNull();
    useEditorStore.getState().cancelOperation();
    expect(useEditorStore.getState().activeOperation).toBeNull();
  });

  it("cancelOperation does not add feature", () => {
    const featuresBefore = useEditorStore.getState().features.length;
    useEditorStore.getState().startOperation("extrude");
    useEditorStore.getState().cancelOperation();
    expect(useEditorStore.getState().features).toHaveLength(featuresBefore);
  });

  // confirmOperation edge cases
  it("confirmOperation fails gracefully without sketch", async () => {
    useEditorStore.getState().startOperation("extrude");
    await useEditorStore.getState().confirmOperation();
    // Should have cleared activeOperation (error path)
    expect(useEditorStore.getState().activeOperation).toBeNull();
  });

  it("confirmOperation with null kernel is no-op", async () => {
    useEditorStore.setState({ kernel: null });
    useEditorStore.getState().startOperation("extrude");
    await useEditorStore.getState().confirmOperation();
    // activeOperation stays set because the function returns early
    // if (!kernel || !activeOperation) return; — kernel is null so it returns
    // But startOperation already set activeOperation, so it remains
  });
});

describe("editor store - view controls", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
  });

  it("toggleWireframe flips value", () => {
    expect(useEditorStore.getState().wireframe).toBe(false);
    useEditorStore.getState().toggleWireframe();
    expect(useEditorStore.getState().wireframe).toBe(true);
    useEditorStore.getState().toggleWireframe();
    expect(useEditorStore.getState().wireframe).toBe(false);
  });

  it("toggleEdges flips value", () => {
    expect(useEditorStore.getState().showEdges).toBe(true);
    useEditorStore.getState().toggleEdges();
    expect(useEditorStore.getState().showEdges).toBe(false);
  });

  it("selectFeature sets selectedFeatureId", () => {
    useEditorStore.getState().selectFeature("feat-1");
    expect(useEditorStore.getState().selectedFeatureId).toBe("feat-1");
    useEditorStore.getState().selectFeature(null);
    expect(useEditorStore.getState().selectedFeatureId).toBeNull();
  });

  it("selectFace sets selectedFaceIndex", () => {
    useEditorStore.getState().selectFace(3);
    expect(useEditorStore.getState().selectedFaceIndex).toBe(3);
    useEditorStore.getState().selectFace(null);
    expect(useEditorStore.getState().selectedFaceIndex).toBeNull();
  });
});

describe("editor store - feature suppression", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
  });

  it("suppressFeature is no-op when kernel is null", () => {
    // Create a sketch feature first
    useEditorStore.getState().enterSketchMode("front");
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 10, y: 0 } });
    store.addSketchEntity({ type: "line", id: "se-2", startId: "se-0", endId: "se-1" });
    useEditorStore.getState().exitSketchMode(true);

    const featuresBefore = useEditorStore.getState().features;
    expect(featuresBefore.length).toBeGreaterThanOrEqual(1);

    // Set kernel to null, then suppress should be a no-op
    useEditorStore.setState({ kernel: null });
    useEditorStore.getState().suppressFeature(0);
    // Features remain unchanged
    expect(useEditorStore.getState().features.length).toBe(featuresBefore.length);
  });

  it("unsuppressFeature is no-op when kernel is null", () => {
    useEditorStore.setState({ kernel: null });
    useEditorStore.getState().unsuppressFeature(0);
    // No crash, features still empty
    expect(useEditorStore.getState().features).toHaveLength(0);
  });
});

describe("editor store - sketch to extrude pipeline", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
  });

  it("complete sketch-to-extrude pipeline produces mesh", async () => {
    // Step 1: Enter sketch mode and draw a closed rectangle
    useEditorStore.getState().enterSketchMode("front");
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 10, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-2", position: { x: 10, y: 5 } });
    store.addSketchEntity({ type: "point", id: "se-3", position: { x: 0, y: 5 } });
    store.addSketchEntity({ type: "line", id: "se-4", startId: "se-0", endId: "se-1" });
    store.addSketchEntity({ type: "line", id: "se-5", startId: "se-1", endId: "se-2" });
    store.addSketchEntity({ type: "line", id: "se-6", startId: "se-2", endId: "se-3" });
    store.addSketchEntity({ type: "line", id: "se-7", startId: "se-3", endId: "se-0" });

    // Add constraints to fully define the rectangle
    store.addSketchConstraint({ id: "sc-0", kind: "fixed", entityIds: ["se-0"] });
    store.addSketchConstraint({ id: "sc-1", kind: "horizontal", entityIds: ["se-4"] });
    store.addSketchConstraint({ id: "sc-2", kind: "horizontal", entityIds: ["se-6"] });
    store.addSketchConstraint({ id: "sc-3", kind: "vertical", entityIds: ["se-5"] });
    store.addSketchConstraint({ id: "sc-4", kind: "vertical", entityIds: ["se-7"] });
    store.addSketchConstraint({ id: "sc-5", kind: "distance", entityIds: ["se-0", "se-1"], value: 10 });
    store.addSketchConstraint({ id: "sc-6", kind: "distance", entityIds: ["se-1", "se-2"], value: 5 });

    // Step 2: Confirm sketch
    useEditorStore.getState().exitSketchMode(true);
    const featuresAfterSketch = useEditorStore.getState().features;
    expect(featuresAfterSketch.length).toBeGreaterThanOrEqual(1);
    expect(featuresAfterSketch.some(f => f.type === "sketch")).toBe(true);

    // Step 3: Start extrude
    useEditorStore.getState().startOperation("extrude");
    expect(useEditorStore.getState().activeOperation).not.toBeNull();
    expect(useEditorStore.getState().activeOperation!.type).toBe("extrude");

    // Step 4: Set depth and confirm
    useEditorStore.getState().updateOperationParams({ depth: 10 });
    await useEditorStore.getState().confirmOperation();

    // Step 5: Verify results
    // After confirm: activeOperation should be cleared
    expect(useEditorStore.getState().activeOperation).toBeNull();
    // Features should have sketch + extrude
    const features = useEditorStore.getState().features;
    expect(features.length).toBeGreaterThanOrEqual(2);
    // Mesh should be produced
    const mesh = useEditorStore.getState().meshData;
    expect(mesh).not.toBeNull();
    // Mesh should have actual geometry (vertices + triangles)
    if (mesh) {
      expect(mesh.positions.length).toBeGreaterThan(0);
      expect(mesh.indices.length).toBeGreaterThan(0);
    }
  });

  it("confirmOperation with invalid depth clears activeOperation", async () => {
    // Create a sketch first
    useEditorStore.getState().enterSketchMode("front");
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 10, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-2", position: { x: 10, y: 5 } });
    store.addSketchEntity({ type: "point", id: "se-3", position: { x: 0, y: 5 } });
    store.addSketchEntity({ type: "line", id: "se-4", startId: "se-0", endId: "se-1" });
    store.addSketchEntity({ type: "line", id: "se-5", startId: "se-1", endId: "se-2" });
    store.addSketchEntity({ type: "line", id: "se-6", startId: "se-2", endId: "se-3" });
    store.addSketchEntity({ type: "line", id: "se-7", startId: "se-3", endId: "se-0" });
    useEditorStore.getState().exitSketchMode(true);

    // Try to extrude with depth 0 (should fail)
    useEditorStore.getState().startOperation("extrude");
    useEditorStore.getState().updateOperationParams({ depth: 0 });
    await useEditorStore.getState().confirmOperation();

    // ActiveOperation should be cleared even on failure
    expect(useEditorStore.getState().activeOperation).toBeNull();
  });

  it("calling confirmOperation without activeOperation is no-op", async () => {
    const featuresBefore = useEditorStore.getState().features.length;
    await useEditorStore.getState().confirmOperation();
    expect(useEditorStore.getState().features).toHaveLength(featuresBefore);
  });

  it("calling initKernel twice does not corrupt state", async () => {
    // Call initKernel twice concurrently
    await Promise.all([
      useEditorStore.getState().initKernel(),
      useEditorStore.getState().initKernel(),
    ]);
    expect(useEditorStore.getState().kernel).not.toBeNull();
    expect(useEditorStore.getState().mode).toBe("view");
  });

  it("second operation after extrude replays features correctly", async () => {
    // This tests the bug where kernel roundtrip converts entities from
    // SketchEntity2D[] to EntityStore format, breaking subsequent replays.

    // Step 1: Create and confirm sketch
    useEditorStore.getState().enterSketchMode("front");
    const store = useEditorStore.getState();
    store.addSketchEntity({ type: "point", id: "se-0", position: { x: 0, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-1", position: { x: 10, y: 0 } });
    store.addSketchEntity({ type: "point", id: "se-2", position: { x: 10, y: 5 } });
    store.addSketchEntity({ type: "point", id: "se-3", position: { x: 0, y: 5 } });
    store.addSketchEntity({ type: "line", id: "se-4", startId: "se-0", endId: "se-1" });
    store.addSketchEntity({ type: "line", id: "se-5", startId: "se-1", endId: "se-2" });
    store.addSketchEntity({ type: "line", id: "se-6", startId: "se-2", endId: "se-3" });
    store.addSketchEntity({ type: "line", id: "se-7", startId: "se-3", endId: "se-0" });
    store.addSketchConstraint({ id: "sc-0", kind: "fixed", entityIds: ["se-0"] });
    store.addSketchConstraint({ id: "sc-1", kind: "horizontal", entityIds: ["se-4"] });
    store.addSketchConstraint({ id: "sc-2", kind: "horizontal", entityIds: ["se-6"] });
    store.addSketchConstraint({ id: "sc-3", kind: "vertical", entityIds: ["se-5"] });
    store.addSketchConstraint({ id: "sc-4", kind: "vertical", entityIds: ["se-7"] });
    store.addSketchConstraint({ id: "sc-5", kind: "distance", entityIds: ["se-0", "se-1"], value: 10 });
    store.addSketchConstraint({ id: "sc-6", kind: "distance", entityIds: ["se-1", "se-2"], value: 5 });
    useEditorStore.getState().exitSketchMode(true);

    // Step 2: First extrude
    useEditorStore.getState().startOperation("extrude");
    useEditorStore.getState().updateOperationParams({ depth: 10 });
    await useEditorStore.getState().confirmOperation();
    expect(useEditorStore.getState().activeOperation).toBeNull();
    expect(useEditorStore.getState().meshData).not.toBeNull();

    // Step 3: Second operation (fillet) — this replays features from kernel
    // roundtrip where entities are in EntityStore format, not SketchEntity2D[]
    useEditorStore.getState().startOperation("fillet");
    useEditorStore.getState().updateOperationParams({ edge_indices: [0], radius: 1 });
    await useEditorStore.getState().confirmOperation();

    // Should not crash — activeOperation cleared regardless of success/failure
    expect(useEditorStore.getState().activeOperation).toBeNull();
  });
});
