import { useMemo } from "react";
import * as THREE from "three";
import { Html } from "@react-three/drei";
import type { MateEntry, ComponentEntry } from "@/stores/assembly-store";

export interface MateVisualizationProps {
  mate: MateEntry;
  components: ComponentEntry[];
  isHighlighted: boolean;
}

/** Color mapping per mate kind */
const MATE_COLORS: Record<string, string> = {
  Coincident: "#44aaff",
  Concentric: "#ff8844",
  Distance: "#44ff88",
  Angle: "#ff44aa",
  Parallel: "#88ff44",
  Perpendicular: "#aa44ff",
  Tangent: "#ffdd44",
  Lock: "#ff4444",
  Gear: "#ffaa00",
  "Rack-Pinion": "#ffaa00",
  RackPinion: "#ffaa00",
};

function getMateColor(kind: string): string {
  return MATE_COLORS[kind] ?? "#44aaff";
}

/**
 * Resolve approximate component positions from their index in the component list.
 * Since we don't have full transform data in the store entries, we use index-based
 * spacing as a heuristic (matching the x-offset pattern from setupAssemblyWithBoxes).
 */
function getComponentPosition(
  compId: string,
  components: ComponentEntry[],
): THREE.Vector3 {
  const idx = components.findIndex((c) => c.id === compId);
  if (idx < 0) return new THREE.Vector3(0, 0, 0);
  return new THREE.Vector3(idx * 15, 2.5, 3.5);
}

/** Semi-transparent plane indicator for Coincident mates */
function CoincidentIndicator({ posA, posB, color }: { posA: THREE.Vector3; posB: THREE.Vector3; color: string }) {
  return (
    <group>
      <mesh position={posA} data-testid="mate-vis-coincident-a">
        <planeGeometry args={[6, 6]} />
        <meshBasicMaterial color={color} transparent opacity={0.25} side={THREE.DoubleSide} depthWrite={false} />
      </mesh>
      <mesh position={posB} data-testid="mate-vis-coincident-b">
        <planeGeometry args={[6, 6]} />
        <meshBasicMaterial color={color} transparent opacity={0.25} side={THREE.DoubleSide} depthWrite={false} />
      </mesh>
    </group>
  );
}

/** Circle rings for Concentric mates */
function ConcentricIndicator({ posA, posB, color }: { posA: THREE.Vector3; posB: THREE.Vector3; color: string }) {
  const ringGeo = useMemo(() => new THREE.RingGeometry(2, 2.3, 32), []);
  return (
    <group>
      <mesh position={posA} geometry={ringGeo}>
        <meshBasicMaterial color={color} transparent opacity={0.5} side={THREE.DoubleSide} depthWrite={false} />
      </mesh>
      <mesh position={posB} geometry={ringGeo}>
        <meshBasicMaterial color={color} transparent opacity={0.5} side={THREE.DoubleSide} depthWrite={false} />
      </mesh>
    </group>
  );
}

/** Dimension line between two faces with distance label */
function DistanceIndicator({ posA, posB, color }: { posA: THREE.Vector3; posB: THREE.Vector3; color: string }) {
  const midpoint = useMemo(() => posA.clone().add(posB).multiplyScalar(0.5), [posA, posB]);
  const dist = useMemo(() => posA.distanceTo(posB).toFixed(1), [posA, posB]);
  const linePoints = useMemo(
    () => new Float32Array([posA.x, posA.y, posA.z, posB.x, posB.y, posB.z]),
    [posA, posB],
  );

  return (
    <group>
      <line>
        <bufferGeometry>
          <bufferAttribute attach="attributes-position" args={[linePoints, 3]} />
        </bufferGeometry>
        <lineBasicMaterial color={color} linewidth={2} />
      </line>
      <Html position={midpoint} center style={{ pointerEvents: "none" }}>
        <div className="rounded bg-black/70 px-1.5 py-0.5 text-[10px] font-mono text-white whitespace-nowrap">
          {dist}
        </div>
      </Html>
    </group>
  );
}

/** Arc showing the angle between faces */
function AngleIndicator({ posA, posB, color }: { posA: THREE.Vector3; posB: THREE.Vector3; color: string }) {
  const midpoint = useMemo(() => posA.clone().add(posB).multiplyScalar(0.5), [posA, posB]);
  const arcPoints = useMemo(() => {
    const pts: number[] = [];
    const segments = 16;
    const radius = 3;
    for (let i = 0; i <= segments; i++) {
      const t = (i / segments) * Math.PI * 0.25;
      pts.push(
        midpoint.x + Math.cos(t) * radius,
        midpoint.y + Math.sin(t) * radius,
        midpoint.z,
      );
    }
    return new Float32Array(pts);
  }, [midpoint]);

  return (
    <group>
      <line>
        <bufferGeometry>
          <bufferAttribute attach="attributes-position" args={[arcPoints, 3]} />
        </bufferGeometry>
        <lineBasicMaterial color={color} linewidth={2} />
      </line>
    </group>
  );
}

/** Directional arrows for Parallel and Perpendicular mates */
function DirectionArrowIndicator({ posA, posB, color, perpendicular }: { posA: THREE.Vector3; posB: THREE.Vector3; color: string; perpendicular?: boolean }) {
  const arrowLen = 3;
  const arrowPointsA = useMemo(() => {
    const dir = perpendicular ? new THREE.Vector3(0, 1, 0) : new THREE.Vector3(0, 0, 1);
    const end = posA.clone().add(dir.multiplyScalar(arrowLen));
    return new Float32Array([posA.x, posA.y, posA.z, end.x, end.y, end.z]);
  }, [posA, perpendicular]);
  const arrowPointsB = useMemo(() => {
    const dir = perpendicular ? new THREE.Vector3(1, 0, 0) : new THREE.Vector3(0, 0, 1);
    const end = posB.clone().add(dir.multiplyScalar(arrowLen));
    return new Float32Array([posB.x, posB.y, posB.z, end.x, end.y, end.z]);
  }, [posB, perpendicular]);

  return (
    <group>
      <line>
        <bufferGeometry>
          <bufferAttribute attach="attributes-position" args={[arrowPointsA, 3]} />
        </bufferGeometry>
        <lineBasicMaterial color={color} linewidth={2} />
      </line>
      <line>
        <bufferGeometry>
          <bufferAttribute attach="attributes-position" args={[arrowPointsB, 3]} />
        </bufferGeometry>
        <lineBasicMaterial color={color} linewidth={2} />
      </line>
    </group>
  );
}

/** Gear mesh indicator with interlocking circles */
function GearIndicator({ posA, posB, color }: { posA: THREE.Vector3; posB: THREE.Vector3; color: string }) {
  const ringGeoSmall = useMemo(() => new THREE.RingGeometry(1.5, 2, 12), []);
  const ringGeoLarge = useMemo(() => new THREE.RingGeometry(2.5, 3, 12), []);
  return (
    <group>
      <mesh position={posA} geometry={ringGeoSmall}>
        <meshBasicMaterial color={color} transparent opacity={0.5} side={THREE.DoubleSide} depthWrite={false} />
      </mesh>
      <mesh position={posB} geometry={ringGeoLarge}>
        <meshBasicMaterial color={color} transparent opacity={0.5} side={THREE.DoubleSide} depthWrite={false} />
      </mesh>
    </group>
  );
}

/** Connector line between two components with a subtle glow color */
function ConnectorLine({ posA, posB, color }: { posA: THREE.Vector3; posB: THREE.Vector3; color: string }) {
  const linePoints = useMemo(
    () => new Float32Array([posA.x, posA.y, posA.z, posB.x, posB.y, posB.z]),
    [posA, posB],
  );
  return (
    <line>
      <bufferGeometry>
        <bufferAttribute attach="attributes-position" args={[linePoints, 3]} />
      </bufferGeometry>
      <lineBasicMaterial color={color} transparent opacity={0.4} linewidth={1} />
    </line>
  );
}

/** Component glow sphere to highlight connected components */
function ComponentGlow({ position, color }: { position: THREE.Vector3; color: string }) {
  return (
    <mesh position={position}>
      <sphereGeometry args={[5, 16, 16]} />
      <meshBasicMaterial color={color} transparent opacity={0.08} depthWrite={false} />
    </mesh>
  );
}

/**
 * Renders a Three.js overlay visualizing an assembly mate constraint.
 *
 * Visualization varies by mate kind:
 * - Coincident: semi-transparent plane indicators on both faces
 * - Concentric: circle rings at the concentric axis
 * - Distance: dimension line between faces with distance value
 * - Angle: arc showing the angle between faces
 * - Parallel/Perpendicular: directional arrows on faces
 * - Gear/Rack-Pinion: gear mesh indicators
 *
 * All types show a connector line and subtle glow on connected components.
 */
export function MateVisualization({ mate, components, isHighlighted }: MateVisualizationProps) {
  const posA = useMemo(() => getComponentPosition(mate.compA, components), [mate.compA, components]);
  const posB = useMemo(() => getComponentPosition(mate.compB, components), [mate.compB, components]);
  const color = getMateColor(mate.kind);

  const kindLower = mate.kind.toLowerCase();

  return (
    <group data-testid="mate-visualization">
      {/* Connector line between the two components */}
      <ConnectorLine posA={posA} posB={posB} color={color} />

      {/* Component glow highlights */}
      {isHighlighted && <ComponentGlow position={posA} color={color} />}
      {isHighlighted && <ComponentGlow position={posB} color={color} />}

      {/* Kind-specific visualization */}
      {kindLower === "coincident" && (
        <CoincidentIndicator posA={posA} posB={posB} color={color} />
      )}
      {kindLower === "concentric" && (
        <ConcentricIndicator posA={posA} posB={posB} color={color} />
      )}
      {(kindLower === "distance") && (
        <DistanceIndicator posA={posA} posB={posB} color={color} />
      )}
      {kindLower === "angle" && (
        <AngleIndicator posA={posA} posB={posB} color={color} />
      )}
      {kindLower === "parallel" && (
        <DirectionArrowIndicator posA={posA} posB={posB} color={color} />
      )}
      {kindLower === "perpendicular" && (
        <DirectionArrowIndicator posA={posA} posB={posB} color={color} perpendicular />
      )}
      {(kindLower === "gear" || kindLower === "rack-pinion" || kindLower === "rackpinion") && (
        <GearIndicator posA={posA} posB={posB} color={color} />
      )}

      {/* Mate kind label */}
      {isHighlighted && (
        <Html
          position={posA.clone().add(posB).multiplyScalar(0.5).add(new THREE.Vector3(0, 4, 0))}
          center
          style={{ pointerEvents: "none" }}
        >
          <div
            data-testid="mate-visualization-label"
            className="rounded bg-black/80 px-2 py-0.5 text-[10px] font-medium text-white whitespace-nowrap"
          >
            {mate.kind}: {mate.compA} ↔ {mate.compB}
          </div>
        </Html>
      )}
    </group>
  );
}
