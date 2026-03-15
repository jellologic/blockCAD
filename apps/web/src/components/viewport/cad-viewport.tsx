import { useRef } from "react";
import { Canvas, useThree, useFrame } from "@react-three/fiber";
import { OrbitControls, GizmoHelper, GizmoViewport } from "@react-three/drei";
import * as THREE from "three";
import { useEditorStore } from "@/stores/editor-store";
import { ModelMesh } from "./model-mesh";
import { EdgesOverlay } from "./edges-overlay";
import { HeadsUpToolbar } from "@/components/editor/heads-up-toolbar";
import { SketchOverlay } from "@/components/sketch/sketch-overlay";

/** Animates the camera to face the sketch plane when entering sketch mode */
function SketchCameraController() {
  const mode = useEditorStore((s) => s.mode);
  const planeId = useEditorStore((s) => s.sketchSession?.planeId);
  const { camera } = useThree();
  const targetRef = useRef<THREE.Vector3 | null>(null);

  useFrame(() => {
    if (mode !== "sketch" || !planeId) {
      targetRef.current = null;
      return;
    }

    let target: THREE.Vector3;
    switch (planeId) {
      case "front":
        target = new THREE.Vector3(0, 0, 30);
        break;
      case "top":
        target = new THREE.Vector3(0, 30, 0);
        break;
      case "right":
        target = new THREE.Vector3(30, 0, 0);
        break;
      default:
        return;
    }

    if (!targetRef.current) {
      targetRef.current = target;
    }

    camera.position.lerp(targetRef.current, 0.08);
    camera.lookAt(0, 0, 0);
    camera.updateProjectionMatrix();
  });

  return null;
}

export function CadViewport() {
  const meshData = useEditorStore((s) => s.meshData);
  const wireframe = useEditorStore((s) => s.wireframe);
  const showEdges = useEditorStore((s) => s.showEdges);
  const mode = useEditorStore((s) => s.mode);

  if (!meshData || meshData.vertexCount === 0) {
    return (
      <div className="flex h-full items-center justify-center bg-[#3d3d40]">
        <p className="text-white/40">No geometry to display</p>
      </div>
    );
  }

  return (
    <div className="relative h-full w-full" data-testid="viewport">
    <HeadsUpToolbar />
    <Canvas
      camera={{ position: [20, 15, 20], fov: 50, near: 0.1, far: 1000 }}
      style={{ background: "#3d3d40" }}
    >
      <ambientLight intensity={0.4} />
      <directionalLight position={[10, 10, 10]} intensity={0.8} />
      <directionalLight position={[-5, -5, -5]} intensity={0.3} />

      <ModelMesh meshData={meshData} wireframe={wireframe} />
      {showEdges && !wireframe && <EdgesOverlay meshData={meshData} />}

      {mode === "sketch" && <SketchOverlay />}

      <gridHelper
        args={[40, 40, "#555558", "#4a4a4e"]}
        rotation={[0, 0, 0]}
      />

      <OrbitControls
        makeDefault
        enableDamping
        dampingFactor={0.1}
        minDistance={5}
        maxDistance={200}
        enableRotate={mode !== "sketch"}
      />

      <SketchCameraController />

      <GizmoHelper alignment="bottom-right" margin={[80, 80]}>
        <GizmoViewport
          axisColors={["#ff4060", "#40ff60", "#4060ff"]}
          labelColor="white"
        />
      </GizmoHelper>
    </Canvas>
    </div>
  );
}
