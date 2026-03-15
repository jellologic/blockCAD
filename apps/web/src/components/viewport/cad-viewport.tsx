import { Canvas } from "@react-three/fiber";
import { OrbitControls, GizmoHelper, GizmoViewport } from "@react-three/drei";
import { useEditorStore } from "@/stores/editor-store";
import { ModelMesh } from "./model-mesh";
import { EdgesOverlay } from "./edges-overlay";

export function CadViewport() {
  const meshData = useEditorStore((s) => s.meshData);
  const wireframe = useEditorStore((s) => s.wireframe);
  const showEdges = useEditorStore((s) => s.showEdges);

  if (!meshData || meshData.vertexCount === 0) {
    return (
      <div className="flex h-full items-center justify-center bg-[#1a1a2e]">
        <p className="text-white/40">No geometry to display</p>
      </div>
    );
  }

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
