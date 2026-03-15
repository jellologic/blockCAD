import type { MeshData } from "./mesh";

/**
 * Generate a box mesh with face IDs for selection.
 * Box goes from (0,0,0) to (width, height, depth).
 * 6 faces, each with 4 vertices and 2 triangles = 24 vertices, 12 triangles.
 */
export function generateBoxMesh(
  width: number,
  height: number,
  depth: number
): MeshData {
  const w = width,
    h = height,
    d = depth;

  // prettier-ignore
  const positions = new Float32Array([
    // Bottom (z=0, normal 0,0,-1)
    0,0,0,  0,h,0,  w,h,0,  w,0,0,
    // Top (z=d, normal 0,0,1)
    0,0,d,  w,0,d,  w,h,d,  0,h,d,
    // Front (y=0, normal 0,-1,0)
    0,0,0,  w,0,0,  w,0,d,  0,0,d,
    // Back (y=h, normal 0,1,0)
    w,h,0,  0,h,0,  0,h,d,  w,h,d,
    // Left (x=0, normal -1,0,0)
    0,h,0,  0,0,0,  0,0,d,  0,h,d,
    // Right (x=w, normal 1,0,0)
    w,0,0,  w,h,0,  w,h,d,  w,0,d,
  ]);

  // prettier-ignore
  const normals = new Float32Array([
    0,0,-1, 0,0,-1, 0,0,-1, 0,0,-1,
    0,0,1,  0,0,1,  0,0,1,  0,0,1,
    0,-1,0, 0,-1,0, 0,-1,0, 0,-1,0,
    0,1,0,  0,1,0,  0,1,0,  0,1,0,
    -1,0,0, -1,0,0, -1,0,0, -1,0,0,
    1,0,0,  1,0,0,  1,0,0,  1,0,0,
  ]);

  // prettier-ignore
  const uvs = new Float32Array([
    0,0, 0,h, w,h, w,0,
    0,0, w,0, w,h, 0,h,
    0,0, w,0, w,d, 0,d,
    0,0, w,0, w,d, 0,d,
    0,0, h,0, h,d, 0,d,
    0,0, h,0, h,d, 0,d,
  ]);

  // prettier-ignore
  const indices = new Uint32Array([
    0,1,2,   0,2,3,
    4,5,6,   4,6,7,
    8,9,10,  8,10,11,
    12,13,14, 12,14,15,
    16,17,18, 16,18,19,
    20,21,22, 20,22,23,
  ]);

  // Face IDs: 2 triangles per face, 6 faces (face 0 = bottom, 1 = top, etc.)
  // prettier-ignore
  const faceIds = new Uint32Array([0,0, 1,1, 2,2, 3,3, 4,4, 5,5]);

  return {
    positions,
    normals,
    uvs,
    indices,
    faceIds,
    vertexCount: 24,
    triangleCount: 12,
  };
}
