import { useRef } from "react";
import { Canvas, useThree, useFrame } from "@react-three/fiber";
import { OrbitControls, GizmoHelper, GizmoViewport, Html } from "@react-three/drei";
import * as THREE from "three";
import { useEditorStore } from "@/stores/editor-store";
import { ModelMesh } from "./model-mesh";
import { EdgesOverlay } from "./edges-overlay";
import { ExtrudePreview } from "./extrude-preview";
import { HeadsUpToolbar } from "@/components/editor/heads-up-toolbar";
import { SketchOverlay } from "@/components/sketch/sketch-overlay";
import { ConfirmationCorner } from "@/components/sketch/confirmation-corner";
import type { SketchPlaneId } from "@blockCAD/kernel";

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

/** Animates the camera to a target position for view orientation commands */
function ViewOrientationController() {
  const cameraTarget = useEditorStore((s) => s.cameraTarget);
  const setCameraTarget = useEditorStore((s) => s.setCameraTarget);
  const { camera } = useThree();
  const targetRef = useRef<THREE.Vector3 | null>(null);

  useFrame(() => {
    if (!cameraTarget) {
      targetRef.current = null;
      return;
    }

    const target = new THREE.Vector3(...cameraTarget);
    if (!targetRef.current) {
      targetRef.current = target;
    }

    camera.position.lerp(targetRef.current, 0.1);
    camera.lookAt(0, 0, 0);
    camera.updateProjectionMatrix();

    // Clear target when close enough (animation complete)
    if (camera.position.distanceTo(targetRef.current) < 0.1) {
      setCameraTarget(null);
      targetRef.current = null;
    }
  });

  return null;
}

const PLANE_CONFIGS: { id: SketchPlaneId; label: string; rotation: [number, number, number]; color: string; labelPos: [number, number, number] }[] = [
  { id: "front", label: "Front", rotation: [0, 0, 0], color: "#4060ff", labelPos: [6, 6, 0] },
  { id: "top", label: "Top", rotation: [-Math.PI / 2, 0, 0], color: "#40ff60", labelPos: [6, 0, -6] },
  { id: "right", label: "Right", rotation: [0, Math.PI / 2, 0], color: "#ff4060", labelPos: [0, 6, -6] },
];

/** Clickable reference planes shown in the viewport */
function ReferencePlanes() {
  const mode = useEditorStore((s) => s.mode);
  const hoveredPlaneId = useEditorStore((s) => s.hoveredPlaneId);
  const hoverPlane = useEditorStore((s) => s.hoverPlane);
  const enterSketchMode = useEditorStore((s) => s.enterSketchMode);

  // Only show in view or select-plane mode
  if (mode !== "view" && mode !== "select-plane") return null;

  return (
    <group>
      {PLANE_CONFIGS.map((p) => {
        const isHovered = hoveredPlaneId === p.id;
        const isSelecting = mode === "select-plane";
        const opacity = isHovered ? 0.3 : isSelecting ? 0.2 : 0.06;
        const planeSize = 12;

        return (
          <group key={p.id}>
            {/* Filled plane */}
            <mesh
              rotation={p.rotation}
              onPointerEnter={(e) => {
                e.stopPropagation();
                hoverPlane(p.id);
              }}
              onPointerLeave={() => hoverPlane(null)}
              onClick={(e) => {
                e.stopPropagation();
                enterSketchMode(p.id);
              }}
            >
              <planeGeometry args={[planeSize, planeSize]} />
              <meshBasicMaterial
                color={p.color}
                transparent
                opacity={opacity}
                side={THREE.DoubleSide}
                depthWrite={false}
              />
            </mesh>
            {/* Border wireframe */}
            <lineLoop rotation={p.rotation}>
              <bufferGeometry>
                <bufferAttribute
                  attach="attributes-position"
                  args={[new Float32Array([
                    -planeSize/2, -planeSize/2, 0,
                     planeSize/2, -planeSize/2, 0,
                     planeSize/2,  planeSize/2, 0,
                    -planeSize/2,  planeSize/2, 0,
                  ]), 3]}
                />
              </bufferGeometry>
              <lineBasicMaterial color={p.color} transparent opacity={isHovered ? 0.6 : 0.25} />
            </lineLoop>
            {/* Plane label — always visible */}
            <Html position={p.labelPos} center style={{ pointerEvents: "none" }}>
              <div
                className={`select-none rounded px-1.5 py-0.5 text-[10px] font-medium whitespace-nowrap transition-opacity ${
                  isHovered
                    ? "bg-white/25 text-white"
                    : "bg-transparent text-white/40"
                }`}
              >
                {p.label}
              </div>
            </Html>
          </group>
        );
      })}
    </group>
  );
}

export function CadViewport() {
  const meshData = useEditorStore((s) => s.meshData);
  const wireframe = useEditorStore((s) => s.wireframe);
  const showEdges = useEditorStore((s) => s.showEdges);
  const mode = useEditorStore((s) => s.mode);
  const isProcessing = useEditorStore((s) => s.isProcessing);
  const sketchEntityCount = useEditorStore((s) => s.sketchSession?.entities.length ?? 0);
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const hasMesh = meshData && meshData.vertexCount > 0;

  return (
    <div className="relative h-full w-full" data-testid="viewport">
      <HeadsUpToolbar />
      <ConfirmationCorner />
      {isProcessing && (
        <div
          className="absolute top-2 right-2 z-10 flex items-center gap-2 rounded bg-black/60 px-3 py-1.5 text-xs text-white/80"
          data-testid="processing-indicator"
        >
          <span className="inline-block h-3 w-3 animate-spin rounded-full border-2 border-white/30 border-t-white/80" />
          Processing...
        </div>
      )}
      <Canvas
        camera={{ position: [20, 15, 20], fov: 50, near: 0.1, far: 1000 }}
        style={{ background: "#3d3d40" }}
      >
        <ambientLight intensity={0.4} />
        <directionalLight position={[10, 10, 10]} intensity={0.8} />
        <directionalLight position={[-5, -5, -5]} intensity={0.3} />

        {hasMesh && <ModelMesh meshData={meshData} wireframe={wireframe} />}
        {hasMesh && showEdges && !wireframe && <EdgesOverlay meshData={meshData} />}

        {(activeOperation?.type === "extrude" || activeOperation?.type === "cut_extrude") && <ExtrudePreview />}

        {/* Instructional text overlays */}
        {!hasMesh && mode === "view" && (
          <Html center style={{ pointerEvents: "none" }}>
            <div className="text-center select-none">
              <p className="text-white/60 text-sm mb-1">No geometry to display</p>
              <p className="text-white/40 text-xs">Click <strong className="text-white/70">Sketch</strong> or press <strong className="text-white/70">S</strong> to begin</p>
            </div>
          </Html>
        )}
        {mode === "select-plane" && (
          <Html center style={{ pointerEvents: "none" }}>
            <div className="text-center select-none">
              <p className="text-white/80 text-base font-medium mb-1">Click a reference plane</p>
              <p className="text-white/50 text-xs">Front, Top, or Right plane to start sketching</p>
            </div>
          </Html>
        )}
        {mode === "sketch" && sketchEntityCount === 0 && (
          <Html center style={{ pointerEvents: "none" }}>
            <div className="text-center select-none">
              <p className="text-white/50 text-xs">
                Draw with <strong className="text-white/70">L</strong>ine, <strong className="text-white/70">R</strong>ect, <strong className="text-white/70">C</strong>ircle, or <strong className="text-white/70">A</strong>rc
              </p>
            </div>
          </Html>
        )}

        <ReferencePlanes />

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
        <ViewOrientationController />

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
