import { create } from "zustand";
import type { MeshData, FeatureEntry, FeatureKind } from "@blockCAD/kernel";
import { initMockKernel, type MockKernelClient } from "@blockCAD/kernel";

type EditorMode = "view" | "sketch" | "select-face" | "select-edge";

interface EditorState {
  // Kernel state
  kernel: MockKernelClient | null;
  meshData: MeshData | null;
  features: FeatureEntry[];
  isLoading: boolean;
  error: Error | null;

  // Editor mode
  mode: EditorMode;

  // Selection
  selectedFeatureId: string | null;
  selectedFaceIndex: number | null;
  hoveredFaceIndex: number | null;

  // Display
  wireframe: boolean;
  showEdges: boolean;

  // Operations
  activeOperation: { type: FeatureKind; params: Record<string, any> } | null;

  // Actions
  initKernel: () => Promise<void>;
  addFeature: (kind: string, name: string, params: any) => void;
  selectFeature: (id: string | null) => void;
  selectFace: (index: number | null) => void;
  hoverFace: (index: number | null) => void;
  setMode: (mode: EditorMode) => void;
  toggleWireframe: () => void;
  toggleEdges: () => void;
  rebuild: () => void;
  startOperation: (type: FeatureKind) => void;
  updateOperationParams: (params: Record<string, any>) => void;
  confirmOperation: () => void;
  cancelOperation: () => void;
  suppressFeature: (index: number) => void;
  unsuppressFeature: (index: number) => void;
}

export const useEditorStore = create<EditorState>((set, get) => ({
  kernel: null,
  meshData: null,
  features: [],
  isLoading: true,
  error: null,
  mode: "view" as EditorMode,
  selectedFeatureId: null,
  selectedFaceIndex: null,
  hoveredFaceIndex: null,
  wireframe: false,
  showEdges: true,
  activeOperation: null,

  initKernel: async () => {
    try {
      const client = await initMockKernel();
      const mesh = client.tessellate();
      set({
        kernel: client,
        meshData: mesh,
        features: client.featureList,
        isLoading: false,
      });
    } catch (err) {
      set({
        error: err instanceof Error ? err : new Error(String(err)),
        isLoading: false,
      });
    }
  },

  addFeature: (kind, name, params) => {
    const { kernel } = get();
    if (!kernel) return;
    kernel.addFeature(kind, name, params);
    const mesh = kernel.tessellate();
    set({
      meshData: mesh,
      features: kernel.featureList,
    });
  },

  selectFeature: (id) => set({ selectedFeatureId: id }),
  selectFace: (index) => set({ selectedFaceIndex: index }),
  hoverFace: (index) => set({ hoveredFaceIndex: index }),
  setMode: (mode) => set({ mode }),
  toggleWireframe: () => set((s) => ({ wireframe: !s.wireframe })),
  toggleEdges: () => set((s) => ({ showEdges: !s.showEdges })),

  rebuild: () => {
    const { kernel } = get();
    if (!kernel) return;
    const mesh = kernel.tessellate();
    set({
      meshData: mesh,
      features: kernel.featureList,
    });
  },

  startOperation: (type: FeatureKind) => {
    const defaultParams: Partial<Record<FeatureKind, Record<string, any>>> = {
      extrude: { direction: [0, 0, 1], depth: 10, symmetric: false, draft_angle: 0 },
      revolve: { axis_origin: [0, 0, 0], axis_direction: [0, 0, 1], angle: 6.283185 },
    };
    set({ activeOperation: { type, params: defaultParams[type] || {} } });
  },

  updateOperationParams: (params) => {
    const { activeOperation } = get();
    if (!activeOperation) return;
    set({
      activeOperation: {
        ...activeOperation,
        params: { ...activeOperation.params, ...params },
      },
    });
  },

  confirmOperation: () => {
    const { kernel, activeOperation } = get();
    if (!kernel || !activeOperation) return;

    // For extrude, auto-add a sketch first if none exists
    const features = kernel.featureList;
    const hasSketch = features.some((f: any) => f.type === "sketch");
    if (!hasSketch && activeOperation.type === "extrude") {
      kernel.addFeature("sketch", "Sketch", { type: "placeholder" });
    }

    kernel.addFeature(activeOperation.type, activeOperation.type.charAt(0).toUpperCase() + activeOperation.type.slice(1), {
      type: activeOperation.type,
      params: activeOperation.params,
    } as any);

    const mesh = kernel.tessellate();
    set({
      activeOperation: null,
      meshData: mesh,
      features: kernel.featureList,
    });
  },

  cancelOperation: () => set({ activeOperation: null }),

  suppressFeature: (index) => {
    const { kernel } = get();
    if (!kernel) return;
    kernel.suppressFeature(index);
    const mesh = kernel.tessellate();
    set({ meshData: mesh, features: kernel.featureList });
  },

  unsuppressFeature: (index) => {
    const { kernel } = get();
    if (!kernel) return;
    kernel.unsuppressFeature(index);
    const mesh = kernel.tessellate();
    set({ meshData: mesh, features: kernel.featureList });
  },
}));
