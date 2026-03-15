import { useMemo, useCallback } from "react";
import * as THREE from "three";
import type { MeshData } from "@blockCAD/kernel";
import type { ThreeEvent } from "@react-three/fiber";
import { useEditorStore } from "@/stores/editor-store";

interface ModelMeshProps {
  meshData: MeshData;
  wireframe?: boolean;
}

/**
 * Renders a highlight overlay for the selected/hovered face.
 * Extracts the triangles belonging to the given faceId and renders them
 * slightly offset from the surface with a translucent highlight material.
 */
function FaceHighlight({
  meshData,
  faceIndex,
  color,
  opacity,
}: {
  meshData: MeshData;
  faceIndex: number;
  color: string;
  opacity: number;
}) {
  const geometry = useMemo(() => {
    const geo = new THREE.BufferGeometry();

    // Collect triangles that belong to this face
    const triIndices: number[] = [];
    for (let i = 0; i < meshData.faceIds.length; i++) {
      if (meshData.faceIds[i] === faceIndex) {
        triIndices.push(i);
      }
    }

    if (triIndices.length === 0) return null;

    // Build position and normal arrays for the highlight triangles
    const posArr: number[] = [];
    const normArr: number[] = [];

    for (const ti of triIndices) {
      for (let v = 0; v < 3; v++) {
        const idx = meshData.indices[ti * 3 + v];
        posArr.push(
          meshData.positions[idx * 3],
          meshData.positions[idx * 3 + 1],
          meshData.positions[idx * 3 + 2]
        );
        normArr.push(
          meshData.normals[idx * 3],
          meshData.normals[idx * 3 + 1],
          meshData.normals[idx * 3 + 2]
        );
      }
    }

    geo.setAttribute(
      "position",
      new THREE.BufferAttribute(new Float32Array(posArr), 3)
    );
    geo.setAttribute(
      "normal",
      new THREE.BufferAttribute(new Float32Array(normArr), 3)
    );

    return geo;
  }, [meshData, faceIndex]);

  if (!geometry) return null;

  return (
    <mesh geometry={geometry} renderOrder={1}>
      <meshStandardMaterial
        color={color}
        transparent
        opacity={opacity}
        depthWrite={false}
        side={THREE.DoubleSide}
        polygonOffset
        polygonOffsetFactor={-1}
        polygonOffsetUnits={-1}
      />
    </mesh>
  );
}

export function ModelMesh({ meshData, wireframe = false }: ModelMeshProps) {
  const mode = useEditorStore((s) => s.mode);
  const selectedFaceIndex = useEditorStore((s) => s.selectedFaceIndex);
  const hoveredFaceIndex = useEditorStore((s) => s.hoveredFaceIndex);
  const selectFace = useEditorStore((s) => s.selectFace);
  const hoverFace = useEditorStore((s) => s.hoverFace);

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

  const handlePointerMove = useCallback(
    (event: ThreeEvent<PointerEvent>) => {
      if (mode !== "select-face") return;
      event.stopPropagation();

      const fi = event.faceIndex;
      if (fi != null && meshData.faceIds) {
        const faceId = meshData.faceIds[fi];
        hoverFace(faceId !== undefined ? faceId : null);
      }
    },
    [mode, meshData, hoverFace]
  );

  const handlePointerOut = useCallback(() => {
    if (mode !== "select-face") return;
    hoverFace(null);
  }, [mode, hoverFace]);

  const handleClick = useCallback(
    (event: ThreeEvent<MouseEvent>) => {
      if (mode !== "select-face") return;
      event.stopPropagation();

      const fi = event.faceIndex;
      if (fi != null && meshData.faceIds) {
        const faceId = meshData.faceIds[fi];
        // Toggle selection: clicking same face deselects
        selectFace(faceId === selectedFaceIndex ? null : faceId);
      }
    },
    [mode, meshData, selectFace, selectedFaceIndex]
  );

  return (
    <group>
      <mesh
        geometry={geometry}
        onPointerMove={handlePointerMove}
        onPointerOut={handlePointerOut}
        onClick={handleClick}
      >
        <meshStandardMaterial
          color="#6b8cff"
          metalness={0.1}
          roughness={0.6}
          wireframe={wireframe}
          side={THREE.DoubleSide}
        />
      </mesh>

      {mode === "select-face" &&
        hoveredFaceIndex !== null &&
        hoveredFaceIndex !== selectedFaceIndex && (
          <FaceHighlight
            meshData={meshData}
            faceIndex={hoveredFaceIndex}
            color="#88bbff"
            opacity={0.3}
          />
        )}

      {selectedFaceIndex !== null && (
        <FaceHighlight
          meshData={meshData}
          faceIndex={selectedFaceIndex}
          color="#44aaff"
          opacity={0.5}
        />
      )}
    </group>
  );
}
