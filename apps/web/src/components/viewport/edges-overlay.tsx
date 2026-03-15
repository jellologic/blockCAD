import { useMemo } from "react";
import * as THREE from "three";
import type { MeshData } from "@blockCAD/kernel";

interface EdgesOverlayProps {
  meshData: MeshData;
}

export function EdgesOverlay({ meshData }: EdgesOverlayProps) {
  const edgesGeometry = useMemo(() => {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute(
      "position",
      new THREE.BufferAttribute(meshData.positions, 3)
    );
    geo.setIndex(new THREE.BufferAttribute(meshData.indices, 1));
    geo.computeVertexNormals();
    return new THREE.EdgesGeometry(geo, 15);
  }, [meshData]);

  return (
    <lineSegments geometry={edgesGeometry}>
      <lineBasicMaterial color="#1a1a2e" linewidth={1.5} />
    </lineSegments>
  );
}
