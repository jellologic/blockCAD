import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";

describe("editor store", () => {
  beforeEach(() => {
    // Reset store state between tests
    useEditorStore.setState({
      kernel: null,
      meshData: null,
      features: [],
      isLoading: true,
      error: null,
      mode: "view",
      selectedFeatureId: null,
      selectedFaceIndex: null,
      hoveredFaceIndex: null,
      wireframe: false,
      showEdges: true,
      activeOperation: null,
      sketchSolver: null,
      sketchDofStatus: null,
    });
  });

  it("starts in loading state", () => {
    const state = useEditorStore.getState();
    expect(state.isLoading).toBe(true);
    expect(state.meshData).toBeNull();
  });

  it("initializes kernel", async () => {
    await useEditorStore.getState().initKernel();
    const state = useEditorStore.getState();
    expect(state.isLoading).toBe(false);
    expect(state.kernel).not.toBeNull();
  });

  it("toggles wireframe", () => {
    useEditorStore.getState().toggleWireframe();
    expect(useEditorStore.getState().wireframe).toBe(true);
    useEditorStore.getState().toggleWireframe();
    expect(useEditorStore.getState().wireframe).toBe(false);
  });

  it("selects features", () => {
    useEditorStore.getState().selectFeature("feat-001");
    expect(useEditorStore.getState().selectedFeatureId).toBe("feat-001");
    useEditorStore.getState().selectFeature(null);
    expect(useEditorStore.getState().selectedFeatureId).toBeNull();
  });

  it("sets mode", () => {
    useEditorStore.getState().setMode("select-face");
    expect(useEditorStore.getState().mode).toBe("select-face");
  });

  it("toggles edges", () => {
    expect(useEditorStore.getState().showEdges).toBe(true);
    useEditorStore.getState().toggleEdges();
    expect(useEditorStore.getState().showEdges).toBe(false);
  });

  it("selects and deselects faces", () => {
    useEditorStore.getState().selectFace(3);
    expect(useEditorStore.getState().selectedFaceIndex).toBe(3);
    useEditorStore.getState().selectFace(null);
    expect(useEditorStore.getState().selectedFaceIndex).toBeNull();
  });

  it("hovers faces", () => {
    useEditorStore.getState().hoverFace(2);
    expect(useEditorStore.getState().hoveredFaceIndex).toBe(2);
    useEditorStore.getState().hoverFace(null);
    expect(useEditorStore.getState().hoveredFaceIndex).toBeNull();
  });
});
