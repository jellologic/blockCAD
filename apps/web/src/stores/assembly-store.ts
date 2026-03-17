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

function downloadText(text: string, filename: string, mimeType: string) {
  const blob = new Blob([text], { type: mimeType });
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

interface DofInfo {
  component_id: string;
  component_name: string;
  status: string | { UnderConstrained: { dof: number } } | { OverConstrained: { redundant: number } };
  mate_count: number;
  grounded: boolean;
}

type AssemblyOp =
  | { type: "insert-component"; partId: string; name: string; x: number; y: number; z: number }
  | { type: "add-mate"; kind: string; compA: string; compB: string; faceA: number; faceB: number; value?: number }
  | { type: "measure"; }
  | null;

// Undo/redo snapshot
interface AssemblySnapshot {
  json: string;
  parts: PartEntry[];
  components: ComponentEntry[];
  mates: MateEntry[];
}

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

  // C1: Configurations
  configurations: string[];
  activeConfigIndex: number | null;

  // C3: Section plane
  hasSectionPlane: boolean;

  // C7: DOF analysis
  dofAnalysis: DofInfo[] | null;

  // C8: Clipboard
  clipboard: string | null;

  // C9: Undo/redo
  undoStack: AssemblySnapshot[];
  redoStack: AssemblySnapshot[];

  // D4: Report
  reportHtml: string | null;

  // Actions
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

  // C1: Configurations
  addConfiguration: (name: string) => void;
  activateConfiguration: (index: number) => void;

  // C3: Section plane
  setSectionPlane: (normal: [number, number, number], offset: number) => void;
  clearSectionPlane: () => void;

  // C6: Remove component
  removeComponent: (compId: string) => void;

  // C7: DOF analysis
  refreshDofAnalysis: () => void;

  // C8: Copy/paste
  copyComponents: (ids: string[]) => void;
  pasteComponents: (offset?: [number, number, number]) => void;

  // C9: Undo/redo
  undo: () => void;
  redo: () => void;
  pushUndoSnapshot: () => void;

  // C10: Measure
  measureDistance: (compA: string, faceA: number, compB: string, faceB: number) => { distance: number; point_a: number[]; point_b: number[] } | null;

  // D1: STEP export
  exportSTEP: () => void;

  // D2: Advanced BOM + CSV
  exportBomCsv: () => void;

  // D4: Report
  generateReport: () => void;
  hideReport: () => void;

  // D5: File open
  openAssemblyFile: (json: string) => void;
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
  configurations: [],
  activeConfigIndex: null,
  hasSectionPlane: false,
  dofAnalysis: null,
  clipboard: null,
  undoStack: [],
  redoStack: [],
  reportHtml: null,

  initAssembly: async () => {
    set({ isLoading: true });
    try {
      await initKernel();
      const assembly = new AssemblyClient();
      set({ assembly, isAssemblyMode: true, isLoading: false, parts: [], components: [], mates: [], undoStack: [], redoStack: [] });
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
      configurations: [],
      activeConfigIndex: null,
      hasSectionPlane: false,
      dofAnalysis: null,
      clipboard: null,
      undoStack: [],
      redoStack: [],
      reportHtml: null,
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
      get().pushUndoSnapshot();
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
      get().pushUndoSnapshot();
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
    get().pushUndoSnapshot();
    assembly.suppressComponent(index);
    const comps = [...get().components];
    if (comps[index]) comps[index] = { ...comps[index], suppressed: true };
    set({ components: comps });
    get().rebuild();
  },

  unsuppressComponent: (index) => {
    const { assembly } = get();
    if (!assembly) return;
    get().pushUndoSnapshot();
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

  // C1: Configurations
  addConfiguration: (name) => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      assembly.addConfiguration(name);
      set({ configurations: [...get().configurations, name] });
      toast.success(`Configuration "${name}" added`);
    } catch (err) {
      toast.error("Failed to add configuration: " + String(err));
    }
  },

  activateConfiguration: (index) => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      assembly.activateConfiguration(index);
      set({ activeConfigIndex: index });
      get().rebuild();
      toast.info(`Activated configuration "${get().configurations[index]}"`);
    } catch (err) {
      toast.error("Failed to activate configuration: " + String(err));
    }
  },

  // C3: Section plane
  setSectionPlane: (normal, offset) => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      assembly.setSectionPlane(JSON.stringify({ normal, offset }));
      set({ hasSectionPlane: true });
      get().rebuild();
    } catch (err) {
      toast.error("Failed to set section plane: " + String(err));
    }
  },

  clearSectionPlane: () => {
    const { assembly } = get();
    if (!assembly) return;
    assembly.clearSectionPlane();
    set({ hasSectionPlane: false });
    get().rebuild();
  },

  // C6: Remove component
  removeComponent: (compId) => {
    const { assembly } = get();
    if (!assembly) return;
    get().pushUndoSnapshot();
    const removed = assembly.removeComponent(compId);
    if (removed) {
      set({
        components: get().components.filter((c) => c.id !== compId),
        mates: get().mates.filter((m) => m.compA !== compId && m.compB !== compId),
      });
      get().rebuild();
      toast.success("Component removed");
    }
  },

  // C7: DOF analysis
  refreshDofAnalysis: () => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      const json = assembly.getDofAnalysisJson();
      const analysis = JSON.parse(json);
      set({ dofAnalysis: analysis });
    } catch (err) {
      console.warn("DOF analysis failed:", err);
    }
  },

  // C8: Copy/paste
  copyComponents: (ids) => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      const snapshot = assembly.copyComponents(JSON.stringify(ids));
      set({ clipboard: snapshot });
      toast.info(`${ids.length} component(s) copied`);
    } catch (err) {
      toast.error("Copy failed: " + String(err));
    }
  },

  pasteComponents: (offset = [10, 0, 0]) => {
    const { assembly, clipboard } = get();
    if (!assembly || !clipboard) return;
    try {
      get().pushUndoSnapshot();
      const newIdsJson = assembly.pasteComponents(clipboard, JSON.stringify(offset));
      const newIds: string[] = JSON.parse(newIdsJson);
      // Refresh components list from the assembly
      const currentComps = get().components;
      const newEntries: ComponentEntry[] = newIds.map((id) => ({
        id,
        partId: "pasted",
        name: `Pasted ${id}`,
        suppressed: false,
      }));
      set({ components: [...currentComps, ...newEntries] });
      get().rebuild();
      toast.success(`${newIds.length} component(s) pasted`);
    } catch (err) {
      toast.error("Paste failed: " + String(err));
    }
  },

  // C9: Undo/redo
  pushUndoSnapshot: () => {
    const { assembly, parts, components, mates } = get();
    if (!assembly) return;
    try {
      const json = assembly.serialize();
      const snapshot: AssemblySnapshot = { json, parts: [...parts], components: [...components], mates: [...mates] };
      set((s) => ({
        undoStack: [...s.undoStack.slice(-49), snapshot],
        redoStack: [],
      }));
    } catch {
      // Ignore serialization errors for undo
    }
  },

  undo: () => {
    const { assembly, undoStack, parts, components, mates } = get();
    if (!assembly || undoStack.length === 0) return;

    // Save current state to redo stack
    try {
      const currentJson = assembly.serialize();
      const currentSnapshot: AssemblySnapshot = {
        json: currentJson,
        parts: [...parts],
        components: [...components],
        mates: [...mates],
      };

      const prev = undoStack[undoStack.length - 1];
      // Restore from snapshot
      const restored = AssemblyClient.deserialize(prev.json);
      set({
        assembly: restored,
        parts: prev.parts,
        components: prev.components,
        mates: prev.mates,
        undoStack: undoStack.slice(0, -1),
        redoStack: [...get().redoStack, currentSnapshot],
      });
      get().rebuild();
      toast.info("Undo");
    } catch (err) {
      toast.error("Undo failed: " + String(err));
    }
  },

  redo: () => {
    const { assembly, redoStack, parts, components, mates } = get();
    if (!assembly || redoStack.length === 0) return;

    try {
      const currentJson = assembly.serialize();
      const currentSnapshot: AssemblySnapshot = {
        json: currentJson,
        parts: [...parts],
        components: [...components],
        mates: [...mates],
      };

      const next = redoStack[redoStack.length - 1];
      const restored = AssemblyClient.deserialize(next.json);
      set({
        assembly: restored,
        parts: next.parts,
        components: next.components,
        mates: next.mates,
        redoStack: redoStack.slice(0, -1),
        undoStack: [...get().undoStack, currentSnapshot],
      });
      get().rebuild();
      toast.info("Redo");
    } catch (err) {
      toast.error("Redo failed: " + String(err));
    }
  },

  // C10: Measure
  measureDistance: (compA, faceA, compB, faceB) => {
    const { assembly } = get();
    if (!assembly) return null;
    try {
      const json = assembly.measureDistance(JSON.stringify({
        comp_a: compA,
        geom_a: { face: faceA },
        comp_b: compB,
        geom_b: { face: faceB },
      }));
      return JSON.parse(json);
    } catch (err) {
      toast.error("Measure failed: " + String(err));
      return null;
    }
  },

  // D1: STEP export
  exportSTEP: () => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      const step = assembly.exportStep();
      downloadText(step, "assembly.step", "application/step");
      toast.success("STEP file exported");
    } catch (err) {
      toast.error("STEP export failed: " + String(err));
    }
  },

  // D3: Advanced BOM CSV export
  exportBomCsv: () => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      const csv = assembly.getBomCsv();
      downloadText(csv, "bom.csv", "text/csv");
      toast.success("BOM CSV exported");
    } catch (err) {
      toast.error("BOM CSV export failed: " + String(err));
    }
  },

  // D4: Report
  generateReport: () => {
    const { assembly } = get();
    if (!assembly) return;
    try {
      const html = assembly.generateReportHtml();
      set({ reportHtml: html });
    } catch (err) {
      toast.error("Report generation failed: " + String(err));
    }
  },

  hideReport: () => set({ reportHtml: null }),

  // D5: File open
  openAssemblyFile: (json) => {
    try {
      const restored = AssemblyClient.deserialize(json);
      set({
        assembly: restored,
        isAssemblyMode: true,
        parts: [],
        components: [],
        mates: [],
        undoStack: [],
        redoStack: [],
      });
      get().rebuild();
      toast.success("Assembly loaded from file");
    } catch (err) {
      toast.error("Failed to open assembly: " + String(err));
    }
  },
}));
