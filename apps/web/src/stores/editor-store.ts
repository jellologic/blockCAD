import { create } from "zustand";
import type {
  MeshData,
  FeatureEntry,
  FeatureKind,
  SketchPlane,
  SketchPlaneId,
  SketchEntity2D,
  SketchConstraint2D,
  SketchPoint2D,
} from "@blockCAD/kernel";
import {
  initKernel,
  KernelClient,
  FRONT_PLANE,
  TOP_PLANE,
  RIGHT_PLANE,
} from "@blockCAD/kernel";

type EditorMode = "view" | "sketch" | "select-face" | "select-edge";

type SketchToolId = "line" | "circle" | "rectangle" | "arc" | "dimension" | null;

interface DimensionInput {
  position: SketchPoint2D;
  entityIds: string[];
  kind: string;
}

interface SketchSession {
  plane: SketchPlane;
  planeId: SketchPlaneId;
  entities: SketchEntity2D[];
  constraints: SketchConstraint2D[];
  activeTool: SketchToolId;
  nextEntityId: number;
  nextConstraintId: number;
  pendingPoints: SketchPoint2D[];
  cursorPos: SketchPoint2D | null;
  dimensionInput: DimensionInput | null;
}

interface EditorState {
  // Kernel state
  kernel: KernelClient | null;
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

  // Sketch editing (non-null when mode === "sketch")
  sketchSession: SketchSession | null;

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

  // Sketch actions
  enterSketchMode: (planeId: SketchPlaneId) => void;
  exitSketchMode: (confirm: boolean) => void;
  setSketchTool: (tool: SketchToolId) => void;
  addSketchEntity: (entity: SketchEntity2D) => void;
  addSketchConstraint: (constraint: SketchConstraint2D) => void;
  addPendingPoint: (pt: SketchPoint2D) => void;
  clearPendingPoints: () => void;
  setSketchCursorPos: (pos: SketchPoint2D | null) => void;
  genSketchEntityId: () => string;
  genSketchConstraintId: () => string;
  showDimensionInput: (position: SketchPoint2D, entityIds: string[], kind: string) => void;
  confirmDimension: (value: number) => void;
  cancelDimension: () => void;
}

function getPlane(planeId: SketchPlaneId): SketchPlane {
  switch (planeId) {
    case "front":
      return FRONT_PLANE;
    case "top":
      return TOP_PLANE;
    case "right":
      return RIGHT_PLANE;
  }
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
  sketchSession: null,

  initKernel: async () => {
    try {
      await initKernel();
      const client = new KernelClient();
      set({
        kernel: client,
        meshData: null,
        features: [],
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
    try {
      const mesh = kernel.tessellate();
      set({
        meshData: mesh,
        features: kernel.featureList,
      });
    } catch {
      set({ features: kernel.featureList });
    }
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
    try {
      const mesh = kernel.tessellate();
      set({
        meshData: mesh,
        features: kernel.featureList,
      });
    } catch {
      set({ features: kernel.featureList });
    }
  },

  startOperation: (type: FeatureKind) => {
    const defaultParams: Partial<Record<FeatureKind, Record<string, any>>> = {
      extrude: {
        direction: [0, 0, 1],
        depth: 10,
        symmetric: false,
        draft_angle: 0,
      },
      revolve: {
        axis_origin: [0, 0, 0],
        axis_direction: [0, 0, 1],
        angle: 6.283185,
      },
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

    const features = kernel.featureList;
    const hasSketch = features.some((f: any) => f.type === "sketch");
    if (!hasSketch && activeOperation.type === "extrude") {
      kernel.addFeature("sketch", "Sketch", { type: "placeholder" });
    }

    kernel.addFeature(
      activeOperation.type,
      activeOperation.type.charAt(0).toUpperCase() +
        activeOperation.type.slice(1),
      {
        type: activeOperation.type,
        params: activeOperation.params,
      } as any
    );

    try {
      const mesh = kernel.tessellate();
      set({
        activeOperation: null,
        meshData: mesh,
        features: kernel.featureList,
      });
    } catch {
      set({
        activeOperation: null,
        features: kernel.featureList,
      });
    }
  },

  cancelOperation: () => set({ activeOperation: null }),

  suppressFeature: (index) => {
    const { kernel } = get();
    if (!kernel) return;
    kernel.suppressFeature(index);
    try {
      const mesh = kernel.tessellate();
      set({ meshData: mesh, features: kernel.featureList });
    } catch {
      set({ features: kernel.featureList });
    }
  },

  unsuppressFeature: (index) => {
    const { kernel } = get();
    if (!kernel) return;
    kernel.unsuppressFeature(index);
    try {
      const mesh = kernel.tessellate();
      set({ meshData: mesh, features: kernel.featureList });
    } catch {
      set({ features: kernel.featureList });
    }
  },

  // --- Sketch actions ---

  enterSketchMode: (planeId) => {
    set({
      mode: "sketch",
      activeOperation: null,
      sketchSession: {
        plane: getPlane(planeId),
        planeId,
        entities: [],
        constraints: [],
        activeTool: null,
        nextEntityId: 0,
        nextConstraintId: 0,
        pendingPoints: [],
        cursorPos: null,
        dimensionInput: null,
      },
    });
  },

  exitSketchMode: (confirm) => {
    const { sketchSession, kernel } = get();
    if (confirm && sketchSession && kernel) {
      const sketchName = `Sketch ${kernel.featureList.filter((f) => f.type === "sketch").length + 1}`;
      try {
        kernel.addFeature("sketch", sketchName, {
          type: "sketch",
          params: {
            plane: sketchSession.plane,
            entities: sketchSession.entities,
            constraints: sketchSession.constraints,
          },
        });
      } catch (err) {
        console.warn("[blockCAD] Failed to add sketch feature:", err);
      }
      try {
        const mesh = kernel.tessellate();
        set({
          mode: "view",
          sketchSession: null,
          meshData: mesh,
          features: kernel.featureList,
        });
      } catch {
        set({
          mode: "view",
          sketchSession: null,
          features: kernel.featureList,
        });
      }
    } else {
      set({ mode: "view", sketchSession: null });
    }
  },

  setSketchTool: (tool) => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    set({
      sketchSession: {
        ...sketchSession,
        activeTool: tool,
        pendingPoints: [],
      },
    });
  },

  addSketchEntity: (entity) => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    set({
      sketchSession: {
        ...sketchSession,
        entities: [...sketchSession.entities, entity],
      },
    });
  },

  addSketchConstraint: (constraint) => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    set({
      sketchSession: {
        ...sketchSession,
        constraints: [...sketchSession.constraints, constraint],
      },
    });
  },

  addPendingPoint: (pt) => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    set({
      sketchSession: {
        ...sketchSession,
        pendingPoints: [...sketchSession.pendingPoints, pt],
      },
    });
  },

  clearPendingPoints: () => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    set({
      sketchSession: { ...sketchSession, pendingPoints: [] },
    });
  },

  setSketchCursorPos: (pos) => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    set({
      sketchSession: { ...sketchSession, cursorPos: pos },
    });
  },

  genSketchEntityId: () => {
    const { sketchSession } = get();
    if (!sketchSession) return "se-0";
    const id = `se-${sketchSession.nextEntityId}`;
    set({
      sketchSession: {
        ...sketchSession,
        nextEntityId: sketchSession.nextEntityId + 1,
      },
    });
    return id;
  },

  genSketchConstraintId: () => {
    const { sketchSession } = get();
    if (!sketchSession) return "sc-0";
    const id = `sc-${sketchSession.nextConstraintId}`;
    set({
      sketchSession: {
        ...sketchSession,
        nextConstraintId: sketchSession.nextConstraintId + 1,
      },
    });
    return id;
  },

  showDimensionInput: (position, entityIds, kind) => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    set({
      sketchSession: {
        ...sketchSession,
        dimensionInput: { position, entityIds, kind },
      },
    });
  },

  confirmDimension: (value) => {
    const { sketchSession } = get();
    if (!sketchSession?.dimensionInput) return;
    const { entityIds, kind } = sketchSession.dimensionInput;
    const id = get().genSketchConstraintId();
    get().addSketchConstraint({ id, kind, entityIds, value });
    set({
      sketchSession: {
        ...get().sketchSession!,
        dimensionInput: null,
      },
    });
  },

  cancelDimension: () => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    set({
      sketchSession: { ...sketchSession, dimensionInput: null },
    });
  },
}));
