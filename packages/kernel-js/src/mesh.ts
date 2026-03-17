/**
 * Typed mesh data extracted from the WASM kernel.
 * Uses Float32Array / Uint32Array for direct GPU buffer compatibility.
 */
export interface MeshData {
  positions: Float32Array;
  normals: Float32Array;
  uvs: Float32Array;
  indices: Uint32Array;
  faceIds: Uint32Array;
  vertexCount: number;
  triangleCount: number;
  /** Pre-computed sharp/feature edge line segments: [x0,y0,z0, x1,y1,z1, ...] */
  edgePositions: Float32Array;
  /** Number of edge line segments */
  edgeCount: number;
}

/**
 * Parse a raw byte buffer from the kernel into structured MeshData.
 *
 * Rust serialization layout (TriMesh::to_bytes):
 *   [vertex_count: u32]
 *   [positions: f32 × vc×3]
 *   [normals:   f32 × vc×3]
 *   [uvs:       f32 × vc×2]
 *   [triangle_count: u32]
 *   [indices:   u32 × tc×3]
 *   [face_ids:  u32 × tc]
 *
 * Edge data is NOT included in the current Rust serialization.
 * If additional bytes remain after face_ids, they are parsed as:
 *   [edge_count: u32]
 *   [edge_positions: f32 × ec×6]
 */
export function parseMeshBytes(buffer: ArrayBuffer): MeshData {
  const view = new DataView(buffer);
  let offset = 0;

  const vertexCount = view.getUint32(offset, true);
  offset += 4;

  const positions = new Float32Array(buffer, offset, vertexCount * 3);
  offset += vertexCount * 3 * 4;

  const normals = new Float32Array(buffer, offset, vertexCount * 3);
  offset += vertexCount * 3 * 4;

  const uvs = new Float32Array(buffer, offset, vertexCount * 2);
  offset += vertexCount * 2 * 4;

  const triangleCount = view.getUint32(offset, true);
  offset += 4;

  const indices = new Uint32Array(buffer, offset, triangleCount * 3);
  offset += triangleCount * 3 * 4;

  const faceIds = new Uint32Array(buffer, offset, triangleCount);
  offset += triangleCount * 4;

  // Edge data is optional — only present if bytes remain
  let edgeCount = 0;
  let edgePositions = new Float32Array(0);
  if (offset + 4 <= buffer.byteLength) {
    edgeCount = view.getUint32(offset, true);
    offset += 4;
    if (edgeCount > 0 && offset + edgeCount * 6 * 4 <= buffer.byteLength) {
      edgePositions = new Float32Array(buffer, offset, edgeCount * 6);
    }
  }

  return {
    positions,
    normals,
    uvs,
    indices,
    faceIds,
    vertexCount,
    triangleCount,
    edgePositions,
    edgeCount,
  };
}
