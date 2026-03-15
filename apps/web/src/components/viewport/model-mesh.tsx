import { useMemo } from "react";
import * as THREE from "three";
import type { MeshData } from "@blockCAD/kernel";

interface ModelMeshProps {
  meshData: MeshData;
  wireframe?: boolean;
}

export function ModelMesh({ meshData, wireframe = false }: ModelMeshProps) {
  const geometry = useMemo(() => {
    const geo = new THREE.BufferGeometry();
    geo.setAttribute(
      "position",
      new THREE.BufferAttribute(meshData.positions, 3)
    );
    geo.setAttribute("normal", new THREE.BufferAttribute(meshData.normals, 3));
    geo.setAttribute("uv", new THREE.BufferAttribute(meshData.uvs, 2));
    geo.setIndex(new THREE.BufferAttribute(meshData.indices, 1));
    return geo;
  }, [meshData]);

  return (
    <mesh geometry={geometry}>
      <meshStandardMaterial
        color="#6b8cff"
        metalness={0.1}
        roughness={0.6}
        wireframe={wireframe}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}
