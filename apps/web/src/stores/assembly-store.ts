import { create } from "zustand";
import type { MeshData } from "@blockCAD/kernel";
import {
  initKernel,
  AssemblyClient,
} from "@blockCAD/kernel";
import { toast } from "sonner";

function downloadFile(data: Uint8Array, filename: string, mimeType: string) {
  const blob = new Blob([new ArrayBuffer(data.byteLength)].map((buf) => { new Uint8Array(buf).set(data); return buf; }), { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

interface PartEntry {
  id: string;
  name: string;
}

export interface ComponentEntry {
  id: string;
  partId: string;
  name: string;
  suppressed: boolean;
}

export interface MateEntry {
  id: string;
  kind: string;
  compA: string;
  compB: string;
  faceA: number;
  faceB: number;
  value?: number;
}

interface BomEntry {
  part_id: string;
  part_name: string;
  quantity: number;
}

interface PatternEntry {
  id: string;
  type: "linear" | "circular";
  sourceComponentIds: string[];
  createdComponentIds: string[];
}

type AssemblyOp =
  | { type: "insert-component"; partId: string; name: string; x: number; y: number; z: number }
  | { type: "add-mate"; kind: string; compA: string; compB: string; faceA: number; faceB: number; value?: number; params?: Record<string, number | [number, number, number]> }
  | { type: "edit-mate"; mateId: string; kind: string; compA: string; compB: string; faceA: number; faceB: number; value?: number }
  | { type: "add-pattern" }
  | null;

interface AssemblyState {
  assembly: AssemblyClient | null;
  meshData: MeshData | null;
  parts: PartEntry[];
  components: ComponentEntry[];
  mates: MateEntry[];
  selectedComponentId: string | null;
  isAssemblyMode: boolean;
  isExploded: boolean;
  bomData: BomEntry[] | null;
  patterns: PatternEntry[];
  activeOp: AssemblyOp;
  isLoading: boolean;
  // Motion study state (from agent-ab236430)
  motionFrames: any[];
  currentFrame: number;
  isPlaying: boolean;
  // Gizmo state (from agent-a8393ac5)
  gizmoMode: "translate" | "rotate" | null;

  initAssembly: () => Promise<void>;
  exitAssemblyMode: () => void;
  addPart: (name: string) => string | null;
  addFeatureToPart: (partId: string, kind: string, params: any) => void;
  insertComponent: (partId: string, name: string, position?: [number, number, number]) => void;
  addMate: (kind: string, compA: string, compB: string, faceA: number, faceB: number, value?: number, params?: Record<string, number | [number, number, number]>) => void;
  editMate: (mateId: string) => void;
  deleteMate: (mateId: string) => void;
  updateMate: (mateId: string, kind: string, compA: string, compB: string, faceA: number, faceB: number, value?: number) => void;
  selectComponent: (id: string | null) => void;
  suppressComponent: (index: number) => void;
  unsuppressComponent: (index: number) => void;
  rebuild: () => void;
  toggleExploded: () => void;
  setExplosionSteps: (steps: Array<{ component_id: string; direction: [number, number, number]; distance: number }>) => void;
  showBom: () => void;
  hideBom: () => void;
  exportGLB: () => void;
  createLinearPattern: (sourceComponentIds: string[], direction: [number, number, number], spacing: number, count: number) => void;
  createCircularPattern: (sourceComponentIds: string[], axisOrigin: [number, number, number], axisDirection: [number, number, number], angleSpacing: number, count: number) => void;
  removePattern: (patternId: string) => void;
  startOp: (op: AssemblyOp) => void;
  cancelOp: () => void;
  confirmOp: () => void;
  // Motion study actions
  runMotionStudy: (driverMateId: string, startValue: number, endValue: number, numSteps: number) => void;
  playMotion: () => void;
  pauseMotion: () => void;
  stopMotion: () => void;
  setFrame: (frame: number) => void;
  // Gizmo actions
  moveComponent: (componentIndex: number, transform: number[]) => void;
  getComponentTransform: (componentIndex: number) => number[] | null;
}

export const useAssemblyStore = create<AssemblyState>((set, get) => ({
  assembly: null,
  meshData: null,
  parts: [],
  components: [],
  mates: [],
  selectedComponentId: null,
  isAssemblyMode: false,
  isExploded: false,
  bomData: null,
  patterns: [],
  activeOp: null,
  isLoading: false,
  motionFrames: [],
  currentFrame: 0,
  isPlaying: false,
  gizmoMode: null,

  initAssembly: async () => {
    set({ isLoading: true });
    try {
      await initKernel();
      const assembly = new AssemblyClient();
      set({ assembly, isAssemblyMode: true, isLoading: false, parts: [], components: [], mates: [] });
    } catch (err) {
      toast.error("Failed to initialize assembly: " + String(err));
      set({ isLoading: false });
    }
  },

  exitAssemblyMode: () => {
    set({
      isAssemblyMode: false,
      assembly: null,
      meshData: null,
      parts: [],
      components: [],
      mates: [],
      bomData: null,
      patterns: [],
      activeOp: null,
    });
  },

  addPart: (name) => {
    const { assembly } = get();
    if (!assembly) return null;
    const id = assembly.addPart(name);
    set({ parts: [...get().parts, { id, name }] });
    return id;
  },

  addFeatureToPart: (partId, kind, params) => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      assembly.addFeatureToPart(partId, kind, params);
    } catch (err) {
      toast.error("Failed to add feature: " + String(err));
    }
  },

  insertComponent: (partId, name, position) => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      const transform = position
        ? [1,0,0,0, 0,1,0,0, 0,0,1,0, position[0],position[1],position[2],1]
        : undefined;
      const id = assembly.addComponent(partId, name, transform);
      set({
        components: [...get().components, { id, partId, name, suppressed: false }],
      });
      get().rebuild();
      toast.success(`Component "${name}" inserted`);
    } catch (err) {
      toast.error("Failed to insert component: " + String(err));
    }
  },

  addMate: (kind, compA, compB, faceA, faceB, value, params) => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      let mateKind: unknown;
      if (params && Object.keys(params).length > 0) {
        mateKind = { [kind]: params };
      } else if (value !== undefined) {
        mateKind = { [kind]: { value } };
      } else {
        mateKind = kind;
      }
      const id = assembly.addMate({
        id: `mate-${Date.now()}`,
        kind: mateKind,
        component_a: compA,
        component_b: compB,
        geometry_ref_a: { face: faceA },
        geometry_ref_b: { face: faceB },
        suppressed: false,
      });
      set({
        mates: [...get().mates, { id, kind, compA, compB, faceA, faceB, value }],
      });
      get().rebuild();
      toast.success(`${kind} mate added`);
    } catch (err) {
      toast.error("Failed to add mate: " + String(err));
    }
  },

  editMate: (mateId) => {
    const mate = get().mates.find((m) => m.id === mateId);
    if (!mate) return;
    set({
      activeOp: {
        type: "edit-mate",
        mateId,
        kind: mate.kind,
        compA: mate.compA,
        compB: mate.compB,
        faceA: mate.faceA,
        faceB: mate.faceB,
        value: mate.value,
      },
    });
  },

  deleteMate: (mateId) => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      assembly.removeMate(mateId);
      set({ mates: get().mates.filter((m) => m.id !== mateId) });
      get().rebuild();
      toast.success("Mate deleted");
    } catch (err) {
      toast.error("Failed to delete mate: " + String(err));
    }
  },

  updateMate: (mateId, kind, compA, compB, faceA, faceB, value) => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      const mateKind = (value !== undefined)
        ? { [kind]: { value } }
        : kind;
      assembly.updateMate(mateId, {
        id: mateId,
        kind: mateKind as any,
        component_a: compA,
        component_b: compB,
        suppressed: false,
      });
      set({
        mates: get().mates.map((m) =>
          m.id === mateId ? { ...m, kind, compA, compB, faceA, faceB, value } : m
        ),
      });
      get().rebuild();
      toast.success("Mate updated");
    } catch (err) {
      toast.error("Failed to update mate: " + String(err));
    }
  },

  selectComponent: (id) => set({ selectedComponentId: id }),

  suppressComponent: (index) => {
    const { assembly } = get();
    if (!assembly) return;
    assembly.suppressComponent(index);
    const comps = [...get().components];
    if (comps[index]) comps[index] = { ...comps[index], suppressed: true };
    set({ components: comps });
    get().rebuild();
  },

  unsuppressComponent: (index) => {
    const { assembly } = get();
    if (!assembly) return;
    assembly.unsuppressComponent(index);
    const comps = [...get().components];
    if (comps[index]) comps[index] = { ...comps[index], suppressed: false };
    set({ components: comps });
    get().rebuild();
  },

  rebuild: () => {
    const { assembly, isExploded } = get();
    if (!assembly) return;
    try {
      const mesh = isExploded ? assembly.tessellateExploded() : assembly.tessellate();
      set({ meshData: mesh });
    } catch (err) {
      console.warn("[blockCAD] Assembly tessellate failed:", err);
    }
  },

  toggleExploded: () => {
    const wasExploded = get().isExploded;
    set({ isExploded: !wasExploded });
    get().rebuild();
    toast.info(wasExploded ? "Normal view" : "Exploded view");
  },

  setExplosionSteps: (steps) => {
    const { assembly } = get();
    if (!assembly) return;
    assembly.setExplosionSteps(steps);
    if (get().isExploded) get().rebuild();
  },

  showBom: () => {
    const { assembly } = get();
    if (!assembly) return;
    const bom = assembly.getBom();
    set({ bomData: bom });
  },

  hideBom: () => set({ bomData: null }),

  exportGLB: () => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      const bytes = assembly.exportGLB({});
      downloadFile(bytes, "assembly.glb", "model/gltf-binary");
      toast.success("Assembly GLB exported");
    } catch (err) {
      toast.error("Export failed: " + String(err));
    }
  },

  createLinearPattern: (sourceComponentIds, direction, spacing, count) => {
    const { assembly, components } = get();
    if (!assembly) return;
    try {
      const sources = sourceComponentIds.map((cid) => {
        const comp = components.find((c) => c.id === cid);
        if (!comp) throw new Error(`Component "${cid}" not found`);
        return { partId: comp.partId, name: comp.name };
      });
      const newIds = assembly.addLinearPattern(sources, direction, spacing, count);
      const patternId = `pattern-${Date.now()}`;
      const newComponents = newIds.map((id: string, idx: number) => {
        const srcIdx = Math.floor(idx / (count - 1));
        const src = sources[srcIdx];
        const instanceNum = (idx % (count - 1)) + 2;
        return {
          id,
          partId: src.partId,
          name: `${src.name} (Linear ${instanceNum})`,
          suppressed: false,
        };
      });
      set({
        components: [...get().components, ...newComponents],
        patterns: [...get().patterns, { id: patternId, type: "linear" as const, sourceComponentIds, createdComponentIds: newIds }],
      });
      get().rebuild();
      toast.success(`Linear pattern created (${newIds.length} instances)`);
    } catch (err) {
      toast.error("Failed to create linear pattern: " + String(err));
    }
  },

  createCircularPattern: (sourceComponentIds, axisOrigin, axisDirection, angleSpacing, count) => {
    const { assembly, components } = get();
    if (!assembly) return;
    try {
      const sources = sourceComponentIds.map((cid) => {
        const comp = components.find((c) => c.id === cid);
        if (!comp) throw new Error(`Component "${cid}" not found`);
        return { partId: comp.partId, name: comp.name };
      });
      const newIds = assembly.addCircularPattern(sources, axisOrigin, axisDirection, angleSpacing, count);
      const patternId = `pattern-${Date.now()}`;
      const newComponents = newIds.map((id: string, idx: number) => {
        const srcIdx = Math.floor(idx / (count - 1));
        const src = sources[srcIdx];
        const instanceNum = (idx % (count - 1)) + 2;
        return {
          id,
          partId: src.partId,
          name: `${src.name} (Circular ${instanceNum})`,
          suppressed: false,
        };
      });
      set({
        components: [...get().components, ...newComponents],
        patterns: [...get().patterns, { id: patternId, type: "circular" as const, sourceComponentIds, createdComponentIds: newIds }],
      });
      get().rebuild();
      toast.success(`Circular pattern created (${newIds.length} instances)`);
    } catch (err) {
      toast.error("Failed to create circular pattern: " + String(err));
    }
  },

  removePattern: (patternId) => {
    const { assembly, patterns, components } = get();
    if (!assembly) return;
    const pattern = patterns.find((p) => p.id === patternId);
    if (!pattern) return;
    try {
      // Find indices of the pattern's created components and suppress them
      const indices = pattern.createdComponentIds
        .map((cid) => components.findIndex((c) => c.id === cid))
        .filter((i) => i >= 0);
      assembly.removePattern(indices);
      // Mark them suppressed in state
      const updatedComponents = components.map((c) =>
        pattern.createdComponentIds.includes(c.id) ? { ...c, suppressed: true } : c,
      );
      set({
        components: updatedComponents,
        patterns: patterns.filter((p) => p.id !== patternId),
      });
      get().rebuild();
      toast.success("Pattern removed");
    } catch (err) {
      toast.error("Failed to remove pattern: " + String(err));
    }
  },

  // Motion study actions
  runMotionStudy: (driverMateId, startValue, endValue, numSteps) => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      const frames = assembly.runMotionStudy({ driverMateId, startValue, endValue, numSteps });
      set({ motionFrames: frames, currentFrame: 0 });
      toast.success(`Motion study: ${frames.length} frames generated`);
    } catch (err) {
      toast.error("Motion study failed: " + String(err));
    }
  },

  playMotion: () => set({ isPlaying: true }),
  pauseMotion: () => set({ isPlaying: false }),
  stopMotion: () => set({ isPlaying: false, currentFrame: 0 }),
  setFrame: (frame) => {
    const { motionFrames } = get();
    if (frame >= 0 && frame < motionFrames.length) {
      set({ currentFrame: frame, meshData: motionFrames[frame]?.mesh ?? null });
    }
  },

  // Gizmo actions
  moveComponent: (componentIndex, transform) => {
    const { assembly } = get();
    if (!assembly || componentIndex < 0) return;
    try {
      assembly.setComponentTransform(componentIndex, transform);
      get().rebuild();
    } catch (err) {
      toast.error("Failed to move component: " + String(err));
    }
  },

  getComponentTransform: (componentIndex) => {
    const { assembly } = get();
    if (!assembly || componentIndex < 0) return null;
    try {
      return assembly.getComponentTransform(componentIndex);
    } catch {
      return null;
    }
  },

  startOp: (op) => set({ activeOp: op }),
  cancelOp: () => set({ activeOp: null }),

  confirmOp: () => {
    const { activeOp } = get();
    if (!activeOp) return;
    if (activeOp.type === "insert-component") {
      get().insertComponent(activeOp.partId, activeOp.name, [activeOp.x, activeOp.y, activeOp.z]);
    } else if (activeOp.type === "add-mate") {
      get().addMate(activeOp.kind, activeOp.compA, activeOp.compB, activeOp.faceA, activeOp.faceB, activeOp.value, activeOp.params);
    } else if (activeOp.type === "edit-mate") {
      get().updateMate(activeOp.mateId, activeOp.kind, activeOp.compA, activeOp.compB, activeOp.faceA, activeOp.faceB, activeOp.value);
    }
    set({ activeOp: null });
  },
}));
