import { create } from "zustand";
import type { MeshData, FeatureEntry } from "@blockCAD/kernel";
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
}));
