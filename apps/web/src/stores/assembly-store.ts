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

interface ComponentEntry {
  id: string;
  partId: string;
  name: string;
  suppressed: boolean;
}

interface MateEntry {
  id: string;
  kind: string;
  compA: string;
  compB: string;
}

interface BomEntry {
  part_id: string;
  part_name: string;
  quantity: number;
}

type AssemblyOp =
  | { type: "insert-component"; partId: string; name: string; x: number; y: number; z: number }
  | { type: "add-mate"; kind: string; compA: string; compB: string; faceA: number; faceB: number; value?: number }
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
  activeOp: AssemblyOp;
  isLoading: boolean;

  initAssembly: () => Promise<void>;
  exitAssemblyMode: () => void;
  addPart: (name: string) => string | null;
  addFeatureToPart: (partId: string, kind: string, params: any) => void;
  insertComponent: (partId: string, name: string, position?: [number, number, number]) => void;
  addMate: (kind: string, compA: string, compB: string, faceA: number, faceB: number, value?: number) => void;
  selectComponent: (id: string | null) => void;
  suppressComponent: (index: number) => void;
  unsuppressComponent: (index: number) => void;
  rebuild: () => void;
  toggleExploded: () => void;
  setExplosionSteps: (steps: Array<{ component_id: string; direction: [number, number, number]; distance: number }>) => void;
  showBom: () => void;
  hideBom: () => void;
  exportGLB: () => void;
  startOp: (op: AssemblyOp) => void;
  cancelOp: () => void;
  confirmOp: () => void;
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
  activeOp: null,
  isLoading: false,

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

  addMate: (kind, compA, compB, faceA, faceB, value) => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      const mateKind = (value !== undefined)
        ? { [kind]: { value } }
        : kind;
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
        mates: [...get().mates, { id, kind, compA, compB }],
      });
      get().rebuild();
      toast.success(`${kind} mate added`);
    } catch (err) {
      toast.error("Failed to add mate: " + String(err));
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

  startOp: (op) => set({ activeOp: op }),
  cancelOp: () => set({ activeOp: null }),

  confirmOp: () => {
    const { activeOp } = get();
    if (!activeOp) return;
    if (activeOp.type === "insert-component") {
      get().insertComponent(activeOp.partId, activeOp.name, [activeOp.x, activeOp.y, activeOp.z]);
    } else if (activeOp.type === "add-mate") {
      get().addMate(activeOp.kind, activeOp.compA, activeOp.compB, activeOp.faceA, activeOp.faceB, activeOp.value);
    }
    set({ activeOp: null });
  },
}));
