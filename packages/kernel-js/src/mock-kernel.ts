import type { MeshData } from "./mesh";
import type { FeatureEntry } from "./types";

/**
 * Mock kernel that returns pre-computed mesh data for a 10×5×7 extruded box.
 * Used until the real WASM build pipeline is set up.
 *
 * Box corners:
 *   Bottom: (0,0,0) (10,0,0) (10,5,0) (0,5,0)
 *   Top:    (0,0,7) (10,0,7) (10,5,7) (0,5,7)
 *
 * 6 faces, each with 4 vertices and 2 triangles = 24 vertices, 12 triangles.
 */

// prettier-ignore
const BOX_POSITIONS = new Float32Array([
  // Bottom face (z=0, normal 0,0,-1) — winding: 0,3,2,1
  0, 0, 0,  0, 5, 0,  10, 5, 0,  10, 0, 0,
  // Top face (z=7, normal 0,0,1) — winding: 4,5,6,7
  0, 0, 7,  10, 0, 7,  10, 5, 7,  0, 5, 7,
  // Front face (y=0, normal 0,-1,0) — winding: 0,1,5,4
  0, 0, 0,  10, 0, 0,  10, 0, 7,  0, 0, 7,
  // Back face (y=5, normal 0,1,0) — winding: 2,3,7,6
  10, 5, 0,  0, 5, 0,  0, 5, 7,  10, 5, 7,
  // Left face (x=0, normal -1,0,0) — winding: 3,0,4,7
  0, 5, 0,  0, 0, 0,  0, 0, 7,  0, 5, 7,
  // Right face (x=10, normal 1,0,0) — winding: 1,2,6,5
  10, 0, 0,  10, 5, 0,  10, 5, 7,  10, 0, 7,
]);

// prettier-ignore
const BOX_NORMALS = new Float32Array([
  // Bottom
  0, 0, -1,  0, 0, -1,  0, 0, -1,  0, 0, -1,
  // Top
  0, 0, 1,  0, 0, 1,  0, 0, 1,  0, 0, 1,
  // Front
  0, -1, 0,  0, -1, 0,  0, -1, 0,  0, -1, 0,
  // Back
  0, 1, 0,  0, 1, 0,  0, 1, 0,  0, 1, 0,
  // Left
  -1, 0, 0,  -1, 0, 0,  -1, 0, 0,  -1, 0, 0,
  // Right
  1, 0, 0,  1, 0, 0,  1, 0, 0,  1, 0, 0,
]);

// prettier-ignore
const BOX_UVS = new Float32Array([
  // Bottom
  0, 0,  0, 5,  10, 5,  10, 0,
  // Top
  0, 0,  10, 0,  10, 5,  0, 5,
  // Front
  0, 0,  10, 0,  10, 7,  0, 7,
  // Back
  0, 0,  10, 0,  10, 7,  0, 7,
  // Left
  0, 0,  5, 0,  5, 7,  0, 7,
  // Right
  0, 0,  5, 0,  5, 7,  0, 7,
]);

// prettier-ignore
const BOX_INDICES = new Uint32Array([
  // Bottom (2 tris)
  0, 1, 2,  0, 2, 3,
  // Top
  4, 5, 6,  4, 6, 7,
  // Front
  8, 9, 10,  8, 10, 11,
  // Back
  12, 13, 14,  12, 14, 15,
  // Left
  16, 17, 18,  16, 18, 19,
  // Right
  20, 21, 22,  20, 22, 23,
]);

export const MOCK_BOX_MESH: MeshData = {
  positions: BOX_POSITIONS,
  normals: BOX_NORMALS,
  uvs: BOX_UVS,
  indices: BOX_INDICES,
  vertexCount: 24,
  triangleCount: 12,
};

export const MOCK_FEATURES: FeatureEntry[] = [
  {
    id: "feat-001",
    name: "Base Sketch",
    type: "sketch",
    suppressed: false,
    params: { type: "placeholder" },
  },
  {
    id: "feat-002",
    name: "Extrude Base",
    type: "extrude",
    suppressed: false,
    params: {
      type: "extrude",
      params: {
        direction: [0, 0, 1],
        depth: 7,
        symmetric: false,
        draft_angle: 0,
      },
    },
  },
];

export class MockKernelClient {
  private features: FeatureEntry[] = [...MOCK_FEATURES];

  get featureCount(): number {
    return this.features.length;
  }

  get featureList(): FeatureEntry[] {
    return this.features;
  }

  tessellate(): MeshData {
    return MOCK_BOX_MESH;
  }
}

export async function initMockKernel(): Promise<MockKernelClient> {
  // Simulate async init (matching real WASM init pattern)
  return new MockKernelClient();
}
