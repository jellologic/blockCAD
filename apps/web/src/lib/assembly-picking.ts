import * as THREE from "three";

export interface FacePickResult {
  componentId: string;
  faceIndex: number;
}

/**
 * Pick a face on an assembly component using Three.js raycasting.
 *
 * @param raycaster - A configured Three.js raycaster (from mouse/pointer event)
 * @param meshes - Map of component_id to Three.js Mesh
 * @param faceIdBuffers - Map of component_id to per-triangle face IDs (Uint32Array, one entry per triangle)
 * @returns The picked component ID and face index, or null if nothing was hit
 */
export function pickAssemblyFace(
  raycaster: THREE.Raycaster,
  meshes: Map<string, THREE.Mesh>,
  faceIdBuffers: Map<string, Uint32Array>,
): FacePickResult | null {
  // Collect all meshes for intersection testing
  const meshArray: THREE.Mesh[] = [];
  const meshToComponentId = new Map<THREE.Mesh, string>();

  for (const [componentId, mesh] of meshes) {
    meshArray.push(mesh);
    meshToComponentId.set(mesh, componentId);
  }

  if (meshArray.length === 0) return null;

  const intersections = raycaster.intersectObjects(meshArray, false);
  if (intersections.length === 0) return null;

  const hit = intersections[0];
  const hitMesh = hit.object as THREE.Mesh;
  const componentId = meshToComponentId.get(hitMesh);
  if (!componentId) return null;

  const faceIds = faceIdBuffers.get(componentId);
  if (!faceIds) return null;

  // hit.faceIndex is the triangle index in the geometry
  const triangleIndex = hit.faceIndex;
  if (triangleIndex === undefined || triangleIndex === null || triangleIndex >= faceIds.length) return null;

  return {
    componentId,
    faceIndex: faceIds[triangleIndex as number],
  };
}

/**
 * Create a highlight material for selected faces.
 * Returns a semi-transparent overlay material.
 */
export function createFaceHighlightMaterial(): THREE.MeshBasicMaterial {
  return new THREE.MeshBasicMaterial({
    color: 0x00aaff,
    transparent: true,
    opacity: 0.4,
    side: THREE.DoubleSide,
    depthTest: true,
    depthWrite: false,
  });
}

/**
 * Build a highlight mesh for a specific face on a component.
 * Extracts only the triangles belonging to the given face index.
 *
 * @param sourceMesh - The component mesh to extract from
 * @param faceIds - Per-triangle face IDs for this component
 * @param targetFaceIndex - The face index to highlight
 * @returns A new mesh containing only the highlighted face triangles, or null if no triangles match
 */
export function buildFaceHighlightMesh(
  sourceMesh: THREE.Mesh,
  faceIds: Uint32Array,
  targetFaceIndex: number,
): THREE.Mesh | null {
  const sourceGeometry = sourceMesh.geometry;
  const sourcePositions = sourceGeometry.getAttribute("position");
  const sourceIndex = sourceGeometry.getIndex();

  if (!sourcePositions || !sourceIndex) return null;

  // Collect triangles that belong to the target face
  const matchingTriangles: number[] = [];
  for (let i = 0; i < faceIds.length; i++) {
    if (faceIds[i] === targetFaceIndex) {
      matchingTriangles.push(i);
    }
  }

  if (matchingTriangles.length === 0) return null;

  // Build new geometry with only the matching triangles
  const positions: number[] = [];
  const indexArray = sourceIndex.array;

  // Remap vertices: old index -> new index
  const vertexMap = new Map<number, number>();
  let nextIndex = 0;

  const newIndices: number[] = [];

  for (const triIdx of matchingTriangles) {
    for (let v = 0; v < 3; v++) {
      const oldIdx = indexArray[triIdx * 3 + v];
      if (!vertexMap.has(oldIdx)) {
        vertexMap.set(oldIdx, nextIndex);
        positions.push(
          sourcePositions.getX(oldIdx),
          sourcePositions.getY(oldIdx),
          sourcePositions.getZ(oldIdx),
        );
        nextIndex++;
      }
      newIndices.push(vertexMap.get(oldIdx)!);
    }
  }

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
  geometry.setIndex(newIndices);
  geometry.computeVertexNormals();

  const mesh = new THREE.Mesh(geometry, createFaceHighlightMaterial());
  // Copy the source mesh's world transform
  mesh.matrixAutoUpdate = false;
  mesh.matrix.copy(sourceMesh.matrixWorld);
  mesh.matrixWorldNeedsUpdate = true;

  return mesh;
}
