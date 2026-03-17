import { create } from "zustand";
import type {
  MeshData,
  MassProperties,
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
  SketchClient,
  FRONT_PLANE,
  TOP_PLANE,
  RIGHT_PLANE,
} from "@blockCAD/kernel";
import { toast } from "sonner";
// Lazy-loaded to avoid circular dependency with @blockCAD/kernel
let _sampleModules: typeof import("@/lib/samples") | null = null;
async function getSampleModels() {
  if (!_sampleModules) _sampleModules = await import("@/lib/samples");
  return _sampleModules.SAMPLE_MODELS;
}

type EditorMode = "view" | "sketch" | "select-face" | "select-edge" | "select-plane";

function downloadFile(data: Uint8Array | string, filename: string, mimeType: string) {
  const blob = typeof data === "string"
    ? new Blob([data], { type: mimeType })
    : new Blob([new ArrayBuffer(data.byteLength)].map((buf) => { new Uint8Array(buf).set(data); return buf; }), { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

type SketchToolId =
  | "line" | "circle" | "rectangle" | "arc" | "dimension" | "measure"
  | "ellipse" | "polygon" | "slot"
  | "trim" | "extend" | "offset" | "mirror"
  | "sketch-fillet" | "sketch-chamfer"
  | "sketch-linear-pattern" | "sketch-circular-pattern"
  | "convert-entities" | "block"
  | null;

type DofStatus = "fully_constrained" | "under_constrained" | "over_constrained" | null;

interface DimensionInput {
  position: SketchPoint2D;
  entityIds: string[];
  kind: string;
}

interface DimensionPending {
  entityIds: string[];
  kind: "distance" | "angle" | "radius" | "diameter";
}

interface MeasureResult {
  from: SketchPoint2D;
  to: SketchPoint2D;
  distance: number;
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
  dimensionPending: DimensionPending | null;
  measureResult: MeasureResult | null;
}

/** Parse entity index from ID string like "se-3" → 3 */
function parseEntityIndex(id: string): number {
  return parseInt(id.replace(/\D+/g, ""), 10);
}

/** Push a frontend SketchEntity2D into the WASM SketchClient */
function mirrorEntityToSolver(solver: SketchClient, entity: SketchEntity2D): void {
  switch (entity.type) {
    case "point":
      solver.addPoint(entity.position.x, entity.position.y);
      break;
    case "line":
      solver.addLine(parseEntityIndex(entity.startId), parseEntityIndex(entity.endId));
      break;
    case "circle":
      solver.addCircle(parseEntityIndex(entity.centerId), entity.radius);
      break;
    case "arc":
      solver.addArc(
        parseEntityIndex(entity.centerId),
        parseEntityIndex(entity.startId),
        parseEntityIndex(entity.endId)
      );
      break;
  }
}

/** Push a frontend SketchConstraint2D into the WASM SketchClient */
function mirrorConstraintToSolver(solver: SketchClient, constraint: SketchConstraint2D): void {
  const indices = constraint.entityIds.map(parseEntityIndex);
  solver.addConstraint(constraint.kind, indices, constraint.value);
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
  hoveredPlaneId: SketchPlaneId | null;

  // Display
  wireframe: boolean;
  showEdges: boolean;

  // Camera
  cameraTarget: [number, number, number] | null;

  // Operations
  activeOperation: { type: FeatureKind; params: Record<string, any> } | null;

  // Sketch editing (non-null when mode === "sketch")
  sketchSession: SketchSession | null;
  sketchSolver: SketchClient | null;
  sketchDofStatus: DofStatus;
  sketchHistory: SketchSession[];
  sketchRedoStack: SketchSession[];
  sketchUndoBatching: boolean;

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
  deleteFeature: (index: number) => void;
  renameFeature: (index: number, name: string) => void;
  moveFeatureUp: (index: number) => void;
  moveFeatureDown: (index: number) => void;
  editFeature: (index: number) => void;
  rollbackTo: (index: number) => void;
  rollForward: () => void;

  // Camera actions
  setCameraTarget: (position: [number, number, number] | null) => void;
  fitAll: () => void;

  // Sketch actions
  applyConstraint: (kind: string) => void;
  enterSketchMode: (planeId: SketchPlaneId) => void;
  exitSketchMode: (confirm: boolean) => void;
  setSketchTool: (tool: SketchToolId) => void;
  addSketchEntity: (entity: SketchEntity2D) => void;
  addSketchConstraint: (constraint: SketchConstraint2D) => void;
  solveSketch: () => void;
  addPendingPoint: (pt: SketchPoint2D) => void;
  clearPendingPoints: () => void;
  setSketchCursorPos: (pos: SketchPoint2D | null) => void;
  genSketchEntityId: () => string;
  genSketchConstraintId: () => string;
  showDimensionInput: (position: SketchPoint2D, entityIds: string[], kind: string) => void;
  setDimensionPending: (pending: DimensionPending | null) => void;
  confirmDimension: (value: number) => void;
  cancelDimension: () => void;
  editDimension: (constraintId: string) => void;
  updateConstraintValue: (constraintId: string, value: number) => void;
  hoverPlane: (planeId: SketchPlaneId | null) => void;
  startSketchFlow: () => void;
  undoSketch: () => void;
  redoSketch: () => void;
  beginUndoBatch: () => void;
  endUndoBatch: () => void;
  deleteSelectedEntities: (entityIds: string[]) => void;
  setMeasureResult: (result: MeasureResult | null) => void;

  // Sample models
  loadSample: (sampleId: string) => Promise<void>;

  // Export actions
  exportSTL: (binary?: boolean) => void;
  exportOBJ: () => void;
  export3MF: () => void;
  exportGLB: () => void;
  exportSTEP: (options?: { schema?: string; author?: string; organization?: string }) => void;

  // Mass properties
  massProperties: MassProperties | null;
  showMassProperties: boolean;
  computeMassProperties: () => void;
  setShowMassProperties: (show: boolean) => void;
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
  hoveredPlaneId: null,
  wireframe: false,
  showEdges: true,
  cameraTarget: null,
  activeOperation: null,
  sketchSession: null,
  sketchSolver: null,
  sketchDofStatus: null,
  sketchHistory: [],
  sketchRedoStack: [],
  sketchUndoBatching: false,
  massProperties: null,
  showMassProperties: false,

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
  selectFace: (index) => {
    set({ selectedFaceIndex: index });
    // If extrude operation is active, update target_face_index
    const { activeOperation } = get();
    if (activeOperation && (activeOperation.type === "extrude" || activeOperation.type === "cut_extrude")) {
      const endCond = activeOperation.params.end_condition;
      if (endCond === "up_to_surface" || endCond === "offset_from_surface" || endCond === "up_to_vertex") {
        get().updateOperationParams({ target_face_index: index });
      }
      // Also handle From: Surface face selection
      const fromCond = activeOperation.params.from_condition;
      if (fromCond === "surface") {
        get().updateOperationParams({ from_face_index: index });
      }
    }
    // If face_fillet is active, toggle face indices
    if (activeOperation && activeOperation.type === "face_fillet") {
      if (index != null) {
        const currentFaces: number[] = activeOperation.params.face_indices || [];
        const next = currentFaces.includes(index)
          ? currentFaces.filter((i: number) => i !== index)
          : [...currentFaces, index];
        get().updateOperationParams({ face_indices: next });
      }
    }
    // If fillet/chamfer/variable_fillet operation is active, derive edge indices from face selection.
    // Each face of a box-like solid has edges shared with adjacent faces.
    // We map face index → the edges that border that face in the BRep.
    if (activeOperation && (activeOperation.type === "fillet" || activeOperation.type === "chamfer" || activeOperation.type === "variable_fillet")) {
      if (index != null) {
        const currentEdges: number[] = activeOperation.params.edge_indices || [];
        // For a simple extruded box, edges are indexed 0–11.
        // Map each face to its bordering edges based on typical BRep topology:
        // Face 0 (bottom): edges 0,1,2,3
        // Face 1 (top): edges 4,5,6,7
        // Face 2-5 (sides): edges shared between top/bottom
        const faceEdgeMap: Record<number, number[]> = {
          0: [0, 1, 2, 3],
          1: [4, 5, 6, 7],
          2: [0, 4, 8, 9],
          3: [1, 5, 9, 10],
          4: [2, 6, 10, 11],
          5: [3, 7, 8, 11],
        };
        const faceEdges = faceEdgeMap[index] ?? [index];

        // Toggle: add edges if not present, remove if already selected
        const edgeSet = new Set(currentEdges);
        const allPresent = faceEdges.every(e => edgeSet.has(e));
        if (allPresent) {
          faceEdges.forEach(e => edgeSet.delete(e));
        } else {
          faceEdges.forEach(e => edgeSet.add(e));
        }
        get().updateOperationParams({ edge_indices: Array.from(edgeSet) });
      }
    }
    // If dome operation is active, set the face_index from face selection
    if (activeOperation && activeOperation.type === "dome" && index != null) {
      get().updateOperationParams({ face_index: index });
    }
  },
  hoverFace: (index) => set({ hoveredFaceIndex: index }),
  setMode: (mode) => set({ mode }),
  toggleWireframe: () => set((s) => ({ wireframe: !s.wireframe })),
  toggleEdges: () => set((s) => ({ showEdges: !s.showEdges })),

  setCameraTarget: (position) => set({ cameraTarget: position }),

  fitAll: () => {
    // Compute a camera position that fits all geometry
    // For now, just go to isometric at a reasonable distance
    set({ cameraTarget: [30, 25, 30] });
  },

  applyConstraint: (kind) => {
    const { sketchSession } = get();
    if (!sketchSession) return;

    const entities = sketchSession.entities;

    if (kind === "fixed") {
      // Find the last point entity
      const lastPoint = [...entities].reverse().find((e) => e.type === "point");
      if (!lastPoint) return;
      const id = get().genSketchConstraintId();
      get().addSketchConstraint({ id, kind: "fixed", entityIds: [lastPoint.id], value: undefined });
    } else if (kind === "horizontal" || kind === "vertical") {
      // Find the last line entity
      const lastLine = [...entities].reverse().find((e) => e.type === "line");
      if (!lastLine) return;
      const id = get().genSketchConstraintId();
      get().addSketchConstraint({ id, kind, entityIds: [lastLine.id], value: undefined });
    }
  },

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
        end_condition: "blind",
        direction2_enabled: false,
        depth2: 10,
        draft_angle2: 0,
        end_condition2: "blind",
        from_offset: 0,
        thin_feature: false,
        thin_wall_thickness: 1,
        target_face_index: null,
        surface_offset: 0,
        target_vertex_position: null,
        flip_side_to_cut: false,
        cap_ends: false,
        from_condition: "sketch_plane",
        from_face_index: null,
        from_vertex_position: null,
        contour_index: null,
      },
      cut_extrude: {
        direction: [0, 0, 1],
        depth: 10,
        symmetric: false,
        draft_angle: 0,
        end_condition: "blind",
        direction2_enabled: false,
        depth2: 10,
        draft_angle2: 0,
        end_condition2: "blind",
        from_offset: 0,
        thin_feature: false,
        thin_wall_thickness: 1,
        target_face_index: null,
        surface_offset: 0,
        target_vertex_position: null,
        flip_side_to_cut: false,
        cap_ends: false,
        from_condition: "sketch_plane",
        from_face_index: null,
        from_vertex_position: null,
        contour_index: null,
      },
      revolve: {
        axis_origin: [0, 0, 0],
        axis_direction: [0, 0, 1],
        angle: 6.283185,
        direction2_enabled: false,
        angle2: 0,
        symmetric: false,
        thin_feature: false,
        thin_wall_thickness: 1,
        flip_side_to_cut: false,
      },
      cut_revolve: {
        axis_origin: [0, 0, 0],
        axis_direction: [0, 0, 1],
        angle: 6.283185,
        direction2_enabled: false,
        angle2: 0,
        symmetric: false,
        thin_feature: false,
        thin_wall_thickness: 1,
        flip_side_to_cut: false,
      },
      fillet: {
        edge_indices: [],
        radius: 1,
      },
      chamfer: {
        edge_indices: [],
        distance: 1,
        distance2: null,
      },
      linear_pattern: {
        direction: [1, 0, 0],
        spacing: 10,
        count: 2,
        direction2: null,
        spacing2: 10,
        count2: 2,
      },
      circular_pattern: {
        axis_origin: [0, 0, 0],
        axis_direction: [0, 0, 1],
        count: 4,
        total_angle: 6.283185,
      },
      mirror: {
        plane_origin: [0, 0, 0],
        plane_normal: [1, 0, 0],
      },
      shell: {
        faces_to_remove: [],
        thickness: 1,
      },
      variable_fillet: {
        edge_indices: [],
        control_points: [
          { parameter: 0, radius: 1 },
          { parameter: 1, radius: 1 },
        ],
        smooth_transition: true,
      },
      face_fillet: {
        face_indices: [],
        radius: 1,
      },
      hole_wizard: {
        hole_type: "simple",
        diameter: 5,
        depth: 10,
        through_all: false,
        position: [0, 0, 0],
        direction: [0, -1, 0],
        cbore_diameter: 8,
        cbore_depth: 3,
        csink_diameter: 10,
        csink_angle: 82,
      },
      move_copy: {
        transform_type: "translate",
        translate_x: 0,
        translate_y: 0,
        translate_z: 0,
        rotate_axis_direction: [0, 0, 1],
        rotate_angle: 0,
        rotate_center: [0, 0, 0],
        copy: false,
      },
      scale: {
        uniform: true,
        scale_factor: 1,
        scale_x: 1,
        scale_y: 1,
        scale_z: 1,
        center: [0, 0, 0],
        copy: false,
      },
      sweep: {
        guide_curves: [],
        orientation: "FollowPath",
        total_twist: 0,
      },
      loft: {
        guide_curves: [],
        start_tangency: { type: "None" },
        end_tangency: { type: "None" },
      },
      dome: {
        face_index: null,
        height: 5,
        elliptical: false,
        direction: null,
      },
      rib: {
        thickness: 1,
        direction: [0, 0, 1],
        flip: false,
        both_sides: false,
      },
    };
    set({ activeOperation: { type, params: defaultParams[type] || {} } });
  },

  updateOperationParams: (() => {
    let rafId: number | null = null;
    let pendingParams: Record<string, any> = {};
    return (params: Record<string, any>) => {
      pendingParams = { ...pendingParams, ...params };
      if (rafId !== null) cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(() => {
        const { activeOperation } = get();
        if (!activeOperation) { pendingParams = {}; rafId = null; return; }
        set({
          activeOperation: {
            ...activeOperation,
            params: { ...activeOperation.params, ...pendingParams },
          },
        });
        pendingParams = {};
        rafId = null;
      });
    };
  })(),

  confirmOperation: async () => {
    const { kernel, activeOperation, features: localFeatures } = get();
    if (!kernel || !activeOperation) return;

    // For extrude/cut_extrude/revolve/cut_revolve: use a fresh kernel to avoid WASM borrow conflicts
    if (activeOperation.type === "extrude" || activeOperation.type === "cut_extrude" || activeOperation.type === "revolve" || activeOperation.type === "cut_revolve" || activeOperation.type === "fillet" || activeOperation.type === "variable_fillet" || activeOperation.type === "face_fillet" || activeOperation.type === "chamfer" || activeOperation.type === "linear_pattern" || activeOperation.type === "circular_pattern" || activeOperation.type === "mirror" || activeOperation.type === "shell" || activeOperation.type === "hole_wizard" || activeOperation.type === "move_copy" || activeOperation.type === "scale" || activeOperation.type === "sweep" || activeOperation.type === "loft" || activeOperation.type === "dome" || activeOperation.type === "rib") {
      const hasSketch = localFeatures.some((f) => f.type === "sketch" && f.params.type === "sketch");
      if (!hasSketch) {
        toast.error(`Cannot ${activeOperation.type}: no sketch found. Draw a sketch first.`);
        set({ activeOperation: null });
        return;
      }

      const featureName = {
        extrude: "Extrude",
        cut_extrude: "Cut Extrude",
        revolve: "Revolve",
        cut_revolve: "Cut Revolve",
        fillet: "Fillet",
        variable_fillet: "Variable Fillet",
        face_fillet: "Face Fillet",
        chamfer: "Chamfer",
        linear_pattern: "Linear Pattern",
        circular_pattern: "Circular Pattern",
        mirror: "Mirror",
        shell: "Shell",
        hole_wizard: "Hole Wizard",
        move_copy: "Move/Copy",
        scale: "Scale",
        sweep: "Sweep",
        loft: "Loft",
        dome: "Dome",
        rib: "Rib",
      }[activeOperation.type] || "Feature";

      // Create a fresh KernelClient to avoid "recursive use" wasm-bindgen error
      const freshKernel = new KernelClient();
      try {
        // Replay all existing features (sketches, extrudes, etc.)
        for (const feat of localFeatures) {
          freshKernel.addFeature(feat.type, feat.name, feat.params);
        }
        // Strip internal UI-only params (prefixed with _) and convert draft angle to radians
        const { _draftEnabled, _draftOutward, _lastDraftAngle, _draftEnabled2, _draftOutward2, _lastDraftAngle2, _fromOffset, ...kernelParams } = activeOperation.params as any;
        const finalParams = {
          ...kernelParams,
          draft_angle: (kernelParams.draft_angle ?? 0) * (Math.PI / 180),
          draft_angle2: (kernelParams.draft_angle2 ?? 0) * (Math.PI / 180),
        };
        freshKernel.addFeature(
          activeOperation.type,
          featureName,
          { type: activeOperation.type, params: finalParams } as any
        );
        const mesh = freshKernel.tessellate();
        set({
          kernel: freshKernel,
          activeOperation: null,
          meshData: mesh,
          features: freshKernel.featureList,
        });
        toast.success(`${featureName} applied`);
        if (activeOperation.type === "cut_revolve") {
          toast.info("Note: Cut Revolve is approximate — full boolean subtract not yet implemented", { duration: 5000 });
        }
        return;
      } catch (err) {
        console.warn(`[blockCAD] ${activeOperation.type} failed:`, err);
        const msg = err instanceof Error ? err.message : String(err);
        if (msg.includes("profile") || msg.includes("closed")) {
          toast.error(`${featureName} failed: sketch must be a closed shape (rectangle, closed loop of lines)`);
        } else {
          toast.error(`${featureName} failed: ` + msg);
        }
        set({ activeOperation: null });
        return;
      }
    }

    // Non-extrude operations
    try {
      kernel.addFeature(
        activeOperation.type,
        activeOperation.type.charAt(0).toUpperCase() + activeOperation.type.slice(1),
        { type: activeOperation.type, params: activeOperation.params } as any
      );
    } catch (err) {
      console.warn("[blockCAD] Operation failed:", err);
      toast.error(`${activeOperation.type} failed`);
      set({ activeOperation: null });
      return;
    }

    try {
      const mesh = kernel.tessellate();
      set({
        activeOperation: null,
        meshData: mesh,
        features: kernel.featureList,
      });
    } catch {
      set({ activeOperation: null });
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

  deleteFeature: (index) => {
    const { kernel } = get();
    if (!kernel) return;
    const features = kernel.featureList;
    if (index < 0 || index >= features.length) return;

    const json = kernel.serialize();
    const doc = JSON.parse(json);
    doc.features.splice(index, 1);

    try {
      const fresh = KernelClient.deserialize(JSON.stringify(doc));
      const mesh = fresh.tessellate();
      set({ kernel: fresh, meshData: mesh, features: fresh.featureList, selectedFeatureId: null });
      toast.success("Feature deleted");
    } catch (err) {
      toast.error("Delete failed: " + (err instanceof Error ? err.message : String(err)));
    }
  },

  renameFeature: (index, name) => {
    const { kernel } = get();
    if (!kernel) return;
    const json = kernel.serialize();
    const doc = JSON.parse(json);
    if (doc.features[index]) {
      doc.features[index].name = name;
    }
    try {
      const fresh = KernelClient.deserialize(JSON.stringify(doc));
      set({ kernel: fresh, features: fresh.featureList });
    } catch {
      toast.error("Rename failed");
    }
  },

  moveFeatureUp: (index) => {
    if (index <= 0) return;
    const { kernel } = get();
    if (!kernel) return;
    const json = kernel.serialize();
    const doc = JSON.parse(json);
    [doc.features[index], doc.features[index - 1]] = [doc.features[index - 1], doc.features[index]];
    try {
      const fresh = KernelClient.deserialize(JSON.stringify(doc));
      const mesh = fresh.tessellate();
      set({ kernel: fresh, meshData: mesh, features: fresh.featureList });
    } catch (err) {
      toast.error("Move failed: " + (err instanceof Error ? err.message : String(err)));
    }
  },

  moveFeatureDown: (index) => {
    const { kernel, features } = get();
    if (!kernel || index >= features.length - 1) return;
    const json = kernel.serialize();
    const doc = JSON.parse(json);
    [doc.features[index], doc.features[index + 1]] = [doc.features[index + 1], doc.features[index]];
    try {
      const fresh = KernelClient.deserialize(JSON.stringify(doc));
      const mesh = fresh.tessellate();
      set({ kernel: fresh, meshData: mesh, features: fresh.featureList });
    } catch (err) {
      toast.error("Move failed: " + (err instanceof Error ? err.message : String(err)));
    }
  },

  editFeature: (index) => {
    const { features } = get();
    const feature = features[index];
    if (!feature) return;
    // Load the feature's params into activeOperation for editing
    const kind = feature.type;
    if (kind === "sketch") {
      // For sketch features, re-enter sketch mode is not supported here
      toast.info("To edit a sketch, double-click it and re-enter sketch mode");
      return;
    }
    const params = feature.params?.params ?? {};
    set({ activeOperation: { type: kind, params: { ...params } } });
  },

  rollbackTo: (index) => {
    const { kernel, features } = get();
    if (!kernel) return;
    // Suppress all features after index
    for (let i = features.length - 1; i > index; i--) {
      if (!features[i].suppressed) {
        kernel.suppressFeature(i);
      }
    }
    try {
      const mesh = kernel.tessellate();
      set({ meshData: mesh, features: kernel.featureList });
    } catch {
      set({ features: kernel.featureList });
    }
  },

  rollForward: () => {
    const { kernel, features } = get();
    if (!kernel) return;
    for (let i = 0; i < features.length; i++) {
      if (features[i].suppressed) {
        kernel.unsuppressFeature(i);
      }
    }
    try {
      const mesh = kernel.tessellate();
      set({ meshData: mesh, features: kernel.featureList });
    } catch {
      set({ features: kernel.featureList });
    }
  },

  // --- Sketch actions ---

  enterSketchMode: (planeId) => {
    const plane = getPlane(planeId);
    let solver: SketchClient | null = null;
    try {
      solver = new SketchClient(plane);
    } catch (err) {
      console.warn("[blockCAD] Failed to create sketch solver:", err);
    }
    set({
      mode: "sketch",
      activeOperation: null,
      sketchSolver: solver,
      sketchDofStatus: null,
      sketchSession: {
        plane,
        planeId,
        entities: [],
        constraints: [],
        activeTool: null,
        nextEntityId: 0,
        nextConstraintId: 0,
        pendingPoints: [],
        cursorPos: null,
        dimensionInput: null,
        dimensionPending: null,
        measureResult: null,
      },
    });
  },

  exitSketchMode: (confirm) => {
    const { sketchSession, sketchSolver, features: existingFeatures } = get();
    if (confirm && sketchSession) {
      const sketchCount = existingFeatures.filter((f) => f.type === "sketch").length;
      const sketchName = `Sketch ${sketchCount + 1}`;

      sketchSolver?.dispose();

      // Save sketch as a local feature entry
      // The kernel will process it when an extrude is applied
      const updatedFeatures = [
        ...existingFeatures,
        {
          id: `sketch-${Date.now()}`,
          name: sketchName,
          type: "sketch" as const,
          suppressed: false,
          params: {
            type: "sketch" as const,
            params: {
              plane: sketchSession.plane,
              entities: sketchSession.entities,
              constraints: sketchSession.constraints,
            },
          },
        },
      ];

      const meshData = get().meshData;

      set({
        mode: "view",
        sketchSession: null,
        sketchSolver: null,
        sketchDofStatus: null,
        meshData,
        features: updatedFeatures,
      });
      toast.success(`${sketchName} saved`);
    } else {
      sketchSolver?.dispose();
      set({ mode: "view", sketchSession: null, sketchSolver: null, sketchDofStatus: null });
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
    const { sketchSession, sketchSolver, sketchHistory, sketchUndoBatching } = get();
    if (!sketchSession) return;
    // Push history for undo (skip if inside a batch — batch pushes once at start)
    const updates: any = {
      sketchSession: {
        ...sketchSession,
        entities: [...sketchSession.entities, entity],
      },
    };
    if (!sketchUndoBatching) {
      updates.sketchHistory = [...sketchHistory, sketchSession].slice(-50);
      updates.sketchRedoStack = [];
    }
    // Mirror to WASM solver
    if (sketchSolver) {
      try {
        mirrorEntityToSolver(sketchSolver, entity);
      } catch (err) {
        console.warn("[blockCAD] Failed to mirror entity to solver:", err);
      }
    }
    set(updates);
  },

  addSketchConstraint: (constraint) => {
    const { sketchSession, sketchSolver } = get();
    if (!sketchSession) return;
    // Mirror to WASM solver
    if (sketchSolver) {
      try {
        mirrorConstraintToSolver(sketchSolver, constraint);
      } catch (err) {
        console.warn("[blockCAD] Failed to mirror constraint to solver:", err);
      }
    }
    set({
      sketchSession: {
        ...sketchSession,
        constraints: [...sketchSession.constraints, constraint],
      },
    });
    // Auto-solve after adding constraint
    get().solveSketch();
  },

  solveSketch: () => {
    const { sketchSolver, sketchSession } = get();
    if (!sketchSolver || !sketchSession) return;
    try {
      const result = sketchSolver.solve();
      if (!result.converged) return;
      // Update entity positions from solved result
      const updatedEntities = sketchSession.entities.map((entity, i) => {
        const solved = result.entities[i];
        if (entity.type === "point" && solved?.type === "point") {
          return { ...entity, position: { x: solved.x, y: solved.y } };
        }
        if (entity.type === "circle" && solved?.type === "circle") {
          return { ...entity, radius: solved.radius };
        }
        return entity;
      });
      // Update DOF status
      let dofStatus: DofStatus = null;
      try {
        dofStatus = sketchSolver.getDofStatus().status;
      } catch { /* ignore */ }
      set({
        sketchSession: { ...sketchSession, entities: updatedEntities },
        sketchDofStatus: dofStatus,
      });
    } catch (err) {
      console.warn("[blockCAD] Sketch solve failed:", err);
    }
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
      sketchSession: { ...sketchSession, dimensionInput: null, dimensionPending: null },
    });
  },

  setDimensionPending: (pending) => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    set({
      sketchSession: { ...sketchSession, dimensionPending: pending },
    });
  },

  editDimension: (constraintId) => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    const constraint = sketchSession.constraints.find((c) => c.id === constraintId);
    if (!constraint || constraint.value === undefined) return;
    // Find position for the input — use first entity's position
    const entityId = constraint.entityIds[0];
    const entity = sketchSession.entities.find((e) => e.id === entityId);
    let position: SketchPoint2D = { x: 0, y: 0 };
    if (entity?.type === "point") {
      position = entity.position;
    } else if (entity?.type === "line") {
      const startPt = sketchSession.entities.find((e) => e.id === entity.startId);
      const endPt = sketchSession.entities.find((e) => e.id === entity.endId);
      if (startPt?.type === "point" && endPt?.type === "point") {
        position = {
          x: (startPt.position.x + endPt.position.x) / 2,
          y: (startPt.position.y + endPt.position.y) / 2,
        };
      }
    }
    set({
      sketchSession: {
        ...sketchSession,
        dimensionInput: { position, entityIds: constraint.entityIds, kind: constraint.kind },
      },
    });
  },

  updateConstraintValue: (constraintId, value) => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    const updatedConstraints = sketchSession.constraints.map((c) =>
      c.id === constraintId ? { ...c, value } : c
    );
    set({
      sketchSession: { ...sketchSession, constraints: updatedConstraints },
    });
    get().solveSketch();
  },

  hoverPlane: (planeId) => set({ hoveredPlaneId: planeId }),

  startSketchFlow: () => {
    set({ mode: "select-plane", hoveredPlaneId: null });
  },

  beginUndoBatch: () => {
    const { sketchSession, sketchHistory } = get();
    if (!sketchSession) return;
    // Save snapshot at batch start — all adds during batch will be undone together
    set({
      sketchUndoBatching: true,
      sketchHistory: [...sketchHistory, sketchSession].slice(-50),
      sketchRedoStack: [],
    });
  },

  endUndoBatch: () => {
    set({ sketchUndoBatching: false });
  },

  undoSketch: () => {
    const { sketchHistory, sketchSession, sketchRedoStack } = get();
    if (sketchHistory.length === 0 || !sketchSession) return;
    const prev = sketchHistory[sketchHistory.length - 1]!;
    set({
      sketchSession: prev,
      sketchHistory: sketchHistory.slice(0, -1),
      sketchRedoStack: [sketchSession, ...sketchRedoStack].slice(0, 50),
    });
  },

  redoSketch: () => {
    const { sketchRedoStack, sketchSession, sketchHistory } = get();
    if (sketchRedoStack.length === 0 || !sketchSession) return;
    const next = sketchRedoStack[0]!;
    set({
      sketchSession: next,
      sketchRedoStack: sketchRedoStack.slice(1),
      sketchHistory: [...sketchHistory, sketchSession].slice(-50),
    });
  },

  deleteSelectedEntities: (entityIds) => {
    const { sketchSession, sketchHistory } = get();
    if (!sketchSession || entityIds.length === 0) return;
    // Push current state to history for undo
    const newHistory = [...sketchHistory, sketchSession].slice(-50);
    const idSet = new Set(entityIds);
    const entities = sketchSession.entities.filter((e) => !idSet.has(e.id));
    // Remove constraints that reference deleted entities
    const constraints = sketchSession.constraints.filter(
      (c) => !c.entityIds.some((eid) => idSet.has(eid))
    );
    set({
      sketchSession: { ...sketchSession, entities, constraints },
      sketchHistory: newHistory,
      sketchRedoStack: [],
    });
  },

  setMeasureResult: (result) => {
    const { sketchSession } = get();
    if (!sketchSession) return;
    set({
      sketchSession: { ...sketchSession, measureResult: result },
    });
  },

  loadSample: async (sampleId) => {
    const models = await getSampleModels();
    const sample = models.find((s) => s.id === sampleId);
    if (!sample) return;
    const fresh = new KernelClient();
    try {
      sample.build(fresh);
      const mesh = fresh.tessellate();
      set({
        kernel: fresh,
        meshData: mesh,
        features: fresh.featureList,
        activeOperation: null,
        selectedFeatureId: null,
        selectedFaceIndex: null,
      });
      toast.success(`Loaded: ${sample.name}`);
    } catch (err) {
      console.error("[blockCAD] Failed to load sample:", err);
      toast.error("Failed to load sample: " + (err instanceof Error ? err.message : String(err)));
    }
  },

  exportSTL: (binary = true) => {
    const { kernel } = get();
    if (!kernel) { toast.error("No model loaded"); return; }
    try {
      if (binary) {
        const bytes = kernel.exportSTLBinary();
        downloadFile(bytes, "model.stl", "application/sla");
      } else {
        const text = kernel.exportSTLAscii({});
        downloadFile(text, "model.stl", "text/plain");
      }
      toast.success("STL exported");
    } catch (err) {
      toast.error("Export failed: " + (err instanceof Error ? err.message : String(err)));
    }
  },

  exportOBJ: () => {
    const { kernel } = get();
    if (!kernel) { toast.error("No model loaded"); return; }
    try {
      const text = kernel.exportOBJ({});
      downloadFile(text, "model.obj", "text/plain");
      toast.success("OBJ exported");
    } catch (err) {
      toast.error("Export failed: " + (err instanceof Error ? err.message : String(err)));
    }
  },

  export3MF: () => {
    const { kernel } = get();
    if (!kernel) { toast.error("No model loaded"); return; }
    try {
      const bytes = kernel.export3MF({});
      downloadFile(bytes, "model.3mf", "application/vnd.ms-package.3dmanufacturing-3dmodel+xml");
      toast.success("3MF exported");
    } catch (err) {
      toast.error("Export failed: " + (err instanceof Error ? err.message : String(err)));
    }
  },

  exportGLB: () => {
    const { kernel } = get();
    if (!kernel) { toast.error("No model loaded"); return; }
    try {
      const bytes = kernel.exportGLB({});
      downloadFile(bytes, "model.glb", "model/gltf-binary");
      toast.success("GLB exported");
    } catch (err) {
      toast.error("Export failed: " + (err instanceof Error ? err.message : String(err)));
    }
  },

  exportSTEP: (options = {}) => {
    const { kernel } = get();
    if (!kernel) { toast.error("No model loaded"); return; }
    try {
      const text = kernel.exportSTEP(options);
      downloadFile(text, "model.step", "application/step");
      toast.success("STEP exported");
    } catch (err) {
      toast.error("Export failed: " + (err instanceof Error ? err.message : String(err)));
    }
  },

  computeMassProperties: () => {
    const { kernel } = get();
    if (!kernel) { toast.error("No model loaded"); return; }
    try {
      const props = kernel.massProperties();
      set({ massProperties: props });
    } catch (err) {
      toast.error("Mass properties failed: " + (err instanceof Error ? err.message : String(err)));
    }
  },

  setShowMassProperties: (show) => {
    if (show) {
      // Auto-compute when opening
      get().computeMassProperties();
    }
    set({ showMassProperties: show });
  },
}));
