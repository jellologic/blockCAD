import { describe, it, expect } from "vitest";
import { initMockKernel, MOCK_BOX_MESH, MOCK_FEATURES } from "../mock-kernel";

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
