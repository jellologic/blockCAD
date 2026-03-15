import { Canvas } from "@react-three/fiber";
import { OrbitControls, GizmoHelper, GizmoViewport } from "@react-three/drei";
import type { MeshData } from "@blockCAD/kernel";
import { ModelMesh } from "./model-mesh";
import { EdgesOverlay } from "./edges-overlay";

interface CadViewportProps {
  meshData: MeshData;
  wireframe?: boolean;
  showEdges?: boolean;
}

export function CadViewport({
  meshData,
  wireframe = false,
  showEdges = true,
}: CadViewportProps) {
  return (
    <Canvas
      camera={{ position: [20, 15, 20], fov: 50, near: 0.1, far: 1000 }}
      style={{ background: "#1a1a2e" }}
    >
      <ambientLight intensity={0.4} />
      <directionalLight position={[10, 10, 10]} intensity={0.8} />
      <directionalLight position={[-5, -5, -5]} intensity={0.3} />

      <ModelMesh meshData={meshData} wireframe={wireframe} />
      {showEdges && !wireframe && <EdgesOverlay meshData={meshData} />}

      <gridHelper
        args={[40, 40, "#444466", "#333355"]}
        rotation={[0, 0, 0]}
      />

      <OrbitControls
        makeDefault
        enableDamping
        dampingFactor={0.1}
        minDistance={5}
        maxDistance={200}
      />

      <GizmoHelper alignment="bottom-right" margin={[80, 80]}>
        <GizmoViewport
          axisColors={["#ff4060", "#40ff60", "#4060ff"]}
          labelColor="white"
        />
      </GizmoHelper>
    </Canvas>
  );
}
