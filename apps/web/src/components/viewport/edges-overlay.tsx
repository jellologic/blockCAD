import { useMemo, useEffect, useRef, memo } from "react";
import * as THREE from "three";
import type { MeshData } from "@blockCAD/kernel";

interface EdgesOverlayProps {
  meshData: MeshData;
}

export const EdgesOverlay = memo(function EdgesOverlay({ meshData }: EdgesOverlayProps) {
  const geometryRef = useRef<THREE.BufferGeometry | null>(null);

  const edgesGeometry = useMemo(() => {
    // Use pre-computed edge positions if available from the kernel
    if ((meshData as any).edgePositions) {
      const geo = new THREE.BufferGeometry();
      geo.setAttribute(
        "position",
        new THREE.BufferAttribute((meshData as any).edgePositions, 3)
      );
      geometryRef.current = geo;
      return geo;
    }

    // Fallback: compute edges from the mesh geometry
    const geo = new THREE.BufferGeometry();
    geo.setAttribute(
      "position",
      new THREE.BufferAttribute(meshData.positions, 3)
    );
    geo.setIndex(new THREE.BufferAttribute(meshData.indices, 1));
    geo.computeVertexNormals();
    const edges = new THREE.EdgesGeometry(geo, 15);
    // Dispose the intermediate geometry (not the edges result)
    geo.dispose();
    geometryRef.current = edges;
    return edges;
  }, [meshData]);

  // Dispose geometry when meshData changes or component unmounts
  useEffect(() => {
    return () => {
      if (geometryRef.current) {
        geometryRef.current.dispose();
        geometryRef.current = null;
      }
    };
  }, [meshData]);

  return (
    <lineSegments geometry={edgesGeometry}>
      <lineBasicMaterial color="#1a1a2e" linewidth={1.5} />
    </lineSegments>
  );
});
