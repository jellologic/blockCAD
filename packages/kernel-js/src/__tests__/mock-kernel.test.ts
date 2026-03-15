import { describe, it, expect } from "vitest";
import { initMockKernel, MOCK_BOX_MESH, MOCK_FEATURES } from "../mock-kernel";
import { generateBoxMesh } from "../mesh-generators";

describe("MockKernelClient", () => {
  it("initializes successfully", async () => {
    const client = await initMockKernel();
    expect(client).toBeDefined();
  });

  it("returns correct feature count", async () => {
    const client = await initMockKernel();
    expect(client.featureCount).toBe(2);
  });

  it("returns feature list with correct entries", async () => {
    const client = await initMockKernel();
    const features = client.featureList;
    expect(features).toHaveLength(2);
    expect(features[0].name).toBe("Base Sketch");
    expect(features[0].type).toBe("sketch");
    expect(features[1].name).toBe("Extrude Base");
    expect(features[1].type).toBe("extrude");
  });

  it("tessellates to 24 vertices and 12 triangles", async () => {
    const client = await initMockKernel();
    const mesh = client.tessellate();
    expect(mesh.vertexCount).toBe(24);
    expect(mesh.triangleCount).toBe(12);
    expect(mesh.positions).toHaveLength(72); // 24 * 3
    expect(mesh.normals).toHaveLength(72);
    expect(mesh.uvs).toHaveLength(48); // 24 * 2
    expect(mesh.indices).toHaveLength(36); // 12 * 3
  });

  it("mesh data uses typed arrays", async () => {
    const client = await initMockKernel();
    const mesh = client.tessellate();
    expect(mesh.positions).toBeInstanceOf(Float32Array);
    expect(mesh.normals).toBeInstanceOf(Float32Array);
    expect(mesh.uvs).toBeInstanceOf(Float32Array);
    expect(mesh.indices).toBeInstanceOf(Uint32Array);
    expect(mesh.faceIds).toBeInstanceOf(Uint32Array);
  });

  it("tessellated mesh includes faceIds", async () => {
    const client = await initMockKernel();
    const mesh = client.tessellate();
    expect(mesh.faceIds).toHaveLength(12); // one per triangle
    // Face 0 = bottom (2 tris), face 1 = top (2 tris), etc.
    expect(mesh.faceIds[0]).toBe(0);
    expect(mesh.faceIds[1]).toBe(0);
    expect(mesh.faceIds[2]).toBe(1);
    expect(mesh.faceIds[3]).toBe(1);
  });

  it("addFeature creates a new feature", async () => {
    const client = await initMockKernel();
    const id = client.addFeature("extrude", "Extrude 2", {
      type: "extrude",
      params: { direction: [0, 0, 1], depth: 15, symmetric: false, draft_angle: 0 },
    });
    expect(id).toMatch(/^feat-\d{3}$/);
    expect(client.featureCount).toBe(3);
    expect(client.featureList[2].name).toBe("Extrude 2");
  });

  it("tessellate uses last extrude depth", async () => {
    const client = await initMockKernel();
    client.addFeature("extrude", "Extrude 2", {
      type: "extrude",
      params: { direction: [0, 0, 1], depth: 20, symmetric: false, draft_angle: 0 },
    });
    const mesh = client.tessellate();
    // With depth=20, the top face z-coords should be 20
    // Check a top-face vertex (vertex index 4 = first top vertex, z component at positions[14])
    expect(mesh.positions[14]).toBe(20);
  });

  it("suppressFeature / unsuppressFeature", async () => {
    const client = await initMockKernel();
    client.suppressFeature(1);
    expect(client.featureList[1].suppressed).toBe(true);
    // With extrude suppressed, tessellate should return empty mesh
    const mesh = client.tessellate();
    expect(mesh.vertexCount).toBe(0);

    client.unsuppressFeature(1);
    expect(client.featureList[1].suppressed).toBe(false);
    const mesh2 = client.tessellate();
    expect(mesh2.vertexCount).toBe(24);
  });
});

describe("MOCK_BOX_MESH", () => {
  it("has valid index bounds", () => {
    const maxIndex = Math.max(...Array.from(MOCK_BOX_MESH.indices));
    expect(maxIndex).toBeLessThan(MOCK_BOX_MESH.vertexCount);
  });

  it("has consistent array sizes", () => {
    expect(MOCK_BOX_MESH.positions.length).toBe(MOCK_BOX_MESH.vertexCount * 3);
    expect(MOCK_BOX_MESH.normals.length).toBe(MOCK_BOX_MESH.vertexCount * 3);
    expect(MOCK_BOX_MESH.uvs.length).toBe(MOCK_BOX_MESH.vertexCount * 2);
    expect(MOCK_BOX_MESH.indices.length).toBe(MOCK_BOX_MESH.triangleCount * 3);
    expect(MOCK_BOX_MESH.faceIds.length).toBe(MOCK_BOX_MESH.triangleCount);
  });

  it("normals are unit vectors", () => {
    for (let i = 0; i < MOCK_BOX_MESH.normals.length; i += 3) {
      const nx = MOCK_BOX_MESH.normals[i];
      const ny = MOCK_BOX_MESH.normals[i + 1];
      const nz = MOCK_BOX_MESH.normals[i + 2];
      const len = Math.sqrt(nx * nx + ny * ny + nz * nz);
      expect(len).toBeCloseTo(1.0, 5);
    }
  });
});

describe("MOCK_FEATURES", () => {
  it("features have required fields", () => {
    for (const f of MOCK_FEATURES) {
      expect(f.id).toBeDefined();
      expect(f.name).toBeDefined();
      expect(f.type).toBeDefined();
      expect(typeof f.suppressed).toBe("boolean");
      expect(f.params).toBeDefined();
    }
  });
});

describe("generateBoxMesh", () => {
  it("generates mesh with correct vertex/triangle counts", () => {
    const mesh = generateBoxMesh(5, 3, 8);
    expect(mesh.vertexCount).toBe(24);
    expect(mesh.triangleCount).toBe(12);
    expect(mesh.faceIds).toHaveLength(12);
  });

  it("generates mesh with correct dimensions", () => {
    const mesh = generateBoxMesh(5, 3, 8);
    // Check a top-face vertex z-coord should be depth=8
    // Top face starts at vertex 4, z at positions[4*3+2] = positions[14]
    expect(mesh.positions[14]).toBe(8);
  });

  it("has valid face IDs", () => {
    const mesh = generateBoxMesh(2, 2, 2);
    const uniqueFaces = new Set(mesh.faceIds);
    expect(uniqueFaces.size).toBe(6);
  });
});
