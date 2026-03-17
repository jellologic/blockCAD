import { useMemo } from "react";
import * as THREE from "three";
import type { MeshData } from "@blockCAD/kernel";

interface EdgesOverlayProps {
  meshData: MeshData;
}

export function EdgesOverlay({ meshData }: EdgesOverlayProps) {
  const edgesGeometry = useMemo(() => {
    const geo = new THREE.BufferGeometry();
    if (meshData.edgePositions && meshData.edgeCount > 0) {
      // Use pre-computed feature edges from kernel (fast path)
      geo.setAttribute(
        "position",
        new THREE.BufferAttribute(meshData.edgePositions, 3)
      );
    } else {
      // Fallback: compute edges on GPU side (legacy/empty case)
      const fallback = new THREE.BufferGeometry();
      fallback.setAttribute(
        "position",
        new THREE.BufferAttribute(meshData.positions, 3)
      );
      fallback.setIndex(new THREE.BufferAttribute(meshData.indices, 1));
      fallback.computeVertexNormals();
      const edges = new THREE.EdgesGeometry(fallback, 15);
      return edges;
    }
    return geo;
  }, [meshData]);

  return (
    <lineSegments geometry={edgesGeometry}>
      <lineBasicMaterial color="#1a1a2e" linewidth={1.5} />
    </lineSegments>
  );
}
