import { useState, useCallback } from "react";
import { Html } from "@react-three/drei";
import * as THREE from "three";
import type { ThreeEvent } from "@react-three/fiber";
import type { MeshData } from "@blockCAD/kernel";
import { usePreferencesStore } from "@/stores/preferences-store";

interface MeasurePoint {
  position: THREE.Vector3;
}

export function MeasureTool({
  meshData,
  active,
}: {
  meshData: MeshData;
  active: boolean;
}) {
  const [points, setPoints] = useState<MeasurePoint[]>([]);
  const unitSystem = usePreferencesStore((s) => s.unitSystem);

  const handleClick = useCallback(
    (event: ThreeEvent<MouseEvent>) => {
      if (!active) return;
      event.stopPropagation();

      const pt = event.point.clone();

      setPoints((prev) => {
        if (prev.length >= 2) {
          // Reset and start new measurement
          return [{ position: pt }];
        }
        return [...prev, { position: pt }];
      });
    },
    [active]
  );

  if (!active) return null;

  const distance =
    points.length === 2
      ? points[0].position.distanceTo(points[1].position)
      : null;

  const midpoint =
    points.length === 2
      ? new THREE.Vector3()
          .addVectors(points[0].position, points[1].position)
          .multiplyScalar(0.5)
      : null;

  return (
    <group>
      {/* Invisible click surface over the mesh */}
      <mesh onClick={handleClick} visible={false}>
        <bufferGeometry>
          <bufferAttribute
            attach="attributes-position"
            args={[meshData.positions, 3]}
          />
          <bufferAttribute
            attach="index"
            args={[meshData.indices, 1]}
          />
        </bufferGeometry>
        <meshBasicMaterial />
      </mesh>

      {/* Measurement points */}
      {points.map((pt, i) => (
        <mesh key={i} position={pt.position}>
          <sphereGeometry args={[0.3, 16, 16]} />
          <meshBasicMaterial color="#ff6644" />
        </mesh>
      ))}

      {/* Measurement line */}
      {points.length === 2 && (
        <line>
          <bufferGeometry>
            <bufferAttribute
              attach="attributes-position"
              args={[
                new Float32Array([
                  points[0].position.x, points[0].position.y, points[0].position.z,
                  points[1].position.x, points[1].position.y, points[1].position.z,
                ]),
                3,
              ]}
            />
          </bufferGeometry>
          <lineBasicMaterial color="#ff6644" linewidth={2} />
        </line>
      )}

      {/* Distance label */}
      {distance !== null && midpoint && (
        <Html position={midpoint} center style={{ pointerEvents: "none" }}>
          <div className="select-none whitespace-nowrap rounded bg-[#ff6644] px-2 py-1 text-[11px] font-mono font-bold text-white shadow-lg">
            {distance.toFixed(2)} {unitSystem}
          </div>
        </Html>
      )}

      {/* Instructions */}
      {points.length < 2 && (
        <Html center position={[0, 15, 0]} style={{ pointerEvents: "none" }}>
          <div className="select-none rounded bg-black/60 px-3 py-1.5 text-xs text-white/80">
            {points.length === 0
              ? "Click first point to measure"
              : "Click second point"}
          </div>
        </Html>
      )}
    </group>
  );
}
