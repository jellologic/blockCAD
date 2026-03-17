import { useMemo } from "react";
import { Html } from "@react-three/drei";
import * as THREE from "three";
import type { MeshData } from "@blockCAD/kernel";
import { useEditorStore } from "@/stores/editor-store";
import { usePreferencesStore } from "@/stores/preferences-store";

/**
 * Shows dimension annotations on the selected feature's faces.
 * Displays bounding box dimensions when a feature is selected.
 */
export function DimensionOverlay({ meshData }: { meshData: MeshData }) {
  const selectedFeatureId = useEditorStore((s) => s.selectedFeatureId);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  const bbox = useMemo(() => {
    if (!meshData || meshData.vertexCount === 0) return null;
    const box = new THREE.Box3();
    for (let i = 0; i < meshData.vertexCount; i++) {
      box.expandByPoint(
        new THREE.Vector3(
          meshData.positions[i * 3],
          meshData.positions[i * 3 + 1],
          meshData.positions[i * 3 + 2]
        )
      );
    }
    return box;
  }, [meshData]);

  if (!selectedFeatureId || !bbox) return null;

  const size = new THREE.Vector3();
  bbox.getSize(size);
  const center = new THREE.Vector3();
  bbox.getCenter(center);

  const format = (v: number) => `${v.toFixed(1)} ${unitSystem}`;

  return (
    <group>
      {/* Width (X) dimension line */}
      {size.x > 0.01 && (
        <Html
          position={[center.x, bbox.min.y - 1.5, center.z]}
          center
          style={{ pointerEvents: "none" }}
        >
          <div className="select-none whitespace-nowrap rounded bg-black/70 px-1.5 py-0.5 text-[10px] font-mono text-blue-300 border border-blue-500/30">
            ↔ {format(size.x)}
          </div>
        </Html>
      )}

      {/* Height (Y) dimension line */}
      {size.y > 0.01 && (
        <Html
          position={[bbox.max.x + 1.5, center.y, center.z]}
          center
          style={{ pointerEvents: "none" }}
        >
          <div className="select-none whitespace-nowrap rounded bg-black/70 px-1.5 py-0.5 text-[10px] font-mono text-green-300 border border-green-500/30">
            ↕ {format(size.y)}
          </div>
        </Html>
      )}

      {/* Depth (Z) dimension line */}
      {size.z > 0.01 && (
        <Html
          position={[center.x, bbox.min.y - 1.5, bbox.max.z + 1.5]}
          center
          style={{ pointerEvents: "none" }}
        >
          <div className="select-none whitespace-nowrap rounded bg-black/70 px-1.5 py-0.5 text-[10px] font-mono text-red-300 border border-red-500/30">
            ↔ {format(size.z)}
          </div>
        </Html>
      )}
    </group>
  );
}
