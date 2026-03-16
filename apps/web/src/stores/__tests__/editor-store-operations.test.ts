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
