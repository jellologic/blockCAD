import { describe, it, expect, beforeEach } from "vitest";
import { useEditorStore } from "@/stores/editor-store";
import type { KernelClient } from "@blockCAD/kernel";

/**
 * Build a sketch + extrude so the kernel has geometry to export.
 * Reuses the same rectangle pattern as the sketch test.
 */
function buildBoxModel(kernel: KernelClient) {
  // Add sketch with a 10×5 rectangle
  kernel.addFeature("sketch", "Sketch 1", {
    type: "sketch",
    params: {
      plane: { origin: [0, 0, 0], normal: [0, 0, 1], uAxis: [1, 0, 0], vAxis: [0, 1, 0] },
      entities: [
        { type: "point", id: "se-0", position: { x: 0, y: 0 } },
        { type: "point", id: "se-1", position: { x: 10, y: 0 } },
        { type: "point", id: "se-2", position: { x: 10, y: 5 } },
        { type: "point", id: "se-3", position: { x: 0, y: 5 } },
        { type: "line", id: "se-4", startId: "se-0", endId: "se-1" },
        { type: "line", id: "se-5", startId: "se-1", endId: "se-2" },
        { type: "line", id: "se-6", startId: "se-2", endId: "se-3" },
        { type: "line", id: "se-7", startId: "se-3", endId: "se-0" },
      ],
      constraints: [
        { id: "sc-0", kind: "fixed", entityIds: ["se-0"] },
        { id: "sc-1", kind: "horizontal", entityIds: ["se-4"] },
        { id: "sc-2", kind: "horizontal", entityIds: ["se-6"] },
        { id: "sc-3", kind: "vertical", entityIds: ["se-5"] },
        { id: "sc-4", kind: "vertical", entityIds: ["se-7"] },
        { id: "sc-5", kind: "distance", entityIds: ["se-0", "se-1"], value: 10 },
        { id: "sc-6", kind: "distance", entityIds: ["se-1", "se-2"], value: 5 },
      ],
    },
  });

  // Add extrude
  kernel.addFeature("extrude", "Extrude 1", {
    type: "extrude",
    params: {
      direction: [0, 0, 1],
      depth: 7,
      symmetric: false,
      draft_angle: 0,
      end_condition: "blind",
      direction2_enabled: false,
      depth2: 0,
      draft_angle2: 0,
      end_condition2: "blind",
      from_offset: 0,
      thin_feature: false,
      thin_wall_thickness: 0,
      flip_side_to_cut: false,
      cap_ends: false,
      from_condition: "sketch_plane",
    },
  });
}

describe("editor store - export", () => {
  beforeEach(async () => {
    useEditorStore.setState({
      kernel: null, meshData: null, features: [], isLoading: true, error: null,
      mode: "view", selectedFeatureId: null, selectedFaceIndex: null,
      hoveredFaceIndex: null, wireframe: false, showEdges: true,
      activeOperation: null, sketchSession: null, sketchSolver: null, sketchDofStatus: null,
    });
    await useEditorStore.getState().initKernel();
  });

  it("exportSTLBinary produces valid binary with geometry", () => {
    const kernel = useEditorStore.getState().kernel!;
    buildBoxModel(kernel);
    const bytes = kernel.exportSTLBinary();
    expect(bytes).toBeInstanceOf(Uint8Array);
    expect(bytes.length).toBeGreaterThan(84); // Header + at least 1 triangle
    // Triangle count at offset 80
    const view = new DataView(bytes.buffer, bytes.byteOffset);
    const triCount = view.getUint32(80, true);
    expect(triCount).toBe(12); // Box = 6 faces × 2 tris
    expect(bytes.length).toBe(84 + 50 * triCount);
  });

  it("exportSTLAscii produces valid ASCII STL", () => {
    const kernel = useEditorStore.getState().kernel!;
    buildBoxModel(kernel);
    const text = kernel.exportSTLAscii({});
    expect(text).toContain("solid");
    expect(text).toContain("facet normal");
    expect(text).toContain("vertex");
    expect(text).toContain("endsolid");
    const facetCount = (text.match(/facet normal/g) || []).length;
    expect(facetCount).toBe(12);
  });

  it("exportOBJ produces valid OBJ", () => {
    const kernel = useEditorStore.getState().kernel!;
    buildBoxModel(kernel);
    const text = kernel.exportOBJ({});
    expect(text).toContain("# blockCAD OBJ export");
    expect(text).toContain("v ");
    expect(text).toContain("vn ");
    expect(text).toContain("f ");
    const vertexCount = (text.match(/\nv /g) || []).length;
    expect(vertexCount).toBe(24); // 6 faces × 4 vertices
    const faceCount = (text.match(/\nf /g) || []).length;
    expect(faceCount).toBe(12);
  });

  it("export3MF produces valid ZIP", () => {
    const kernel = useEditorStore.getState().kernel!;
    buildBoxModel(kernel);
    const bytes = kernel.export3MF({});
    expect(bytes).toBeInstanceOf(Uint8Array);
    // ZIP magic: PK (0x50, 0x4B)
    expect(bytes[0]).toBe(0x50);
    expect(bytes[1]).toBe(0x4B);
  });

  it("exportGLB produces valid GLB", () => {
    const kernel = useEditorStore.getState().kernel!;
    buildBoxModel(kernel);
    const bytes = kernel.exportGLB({});
    expect(bytes).toBeInstanceOf(Uint8Array);
    // glTF magic: 0x46546C67
    const view = new DataView(bytes.buffer, bytes.byteOffset);
    expect(view.getUint32(0, true)).toBe(0x46546C67);
    expect(view.getUint32(4, true)).toBe(2); // version
  });

  it("exportSTLAscii respects precision option", () => {
    const kernel = useEditorStore.getState().kernel!;
    buildBoxModel(kernel);
    const text3 = kernel.exportSTLAscii({ precision: 3 });
    // With precision 3, should not see 6 decimal places
    expect(text3).not.toMatch(/\d+\.\d{6}/);
    // But should have 3 decimal places
    expect(text3).toMatch(/\d+\.\d{3}/);
  });

  it("export without geometry produces empty/minimal file", () => {
    const kernel = useEditorStore.getState().kernel!;
    // No features added — should produce a valid but empty STL (header only, 0 triangles)
    const bytes = kernel.exportSTLBinary();
    expect(bytes.length).toBe(84); // Just the header, 0 triangles
    const view = new DataView(bytes.buffer, bytes.byteOffset);
    expect(view.getUint32(80, true)).toBe(0); // 0 triangles
  });
});
