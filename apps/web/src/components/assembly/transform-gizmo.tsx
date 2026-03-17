import { useRef, useEffect, useMemo } from "react";
import { useThree, useFrame, type ThreeEvent } from "@react-three/fiber";
import * as THREE from "three";
import { useAssemblyStore } from "@/stores/assembly-store";

/**
 * Length of each gizmo axis arrow / ring radius.
 */
const AXIS_LENGTH = 3;
const RING_RADIUS = 2.5;
const RING_TUBE = 0.04;
const ARROW_CONE_HEIGHT = 0.6;
const ARROW_CONE_RADIUS = 0.15;
const SHAFT_RADIUS = 0.05;

const AXIS_COLORS = {
  x: "#ff4060",
  y: "#40ff60",
  z: "#4060ff",
} as const;

const AXIS_DIRS: Record<string, THREE.Vector3> = {
  x: new THREE.Vector3(1, 0, 0),
  y: new THREE.Vector3(0, 1, 0),
  z: new THREE.Vector3(0, 0, 1),
};

/**
 * Extracts the translation from a column-major 4x4 matrix (16-element array).
 */
function getTranslation(m: number[]): [number, number, number] {
  return [m[12], m[13], m[14]];
}

/**
 * Sets the translation of a column-major 4x4 matrix. Returns a new array.
 */
function setTranslation(m: number[], x: number, y: number, z: number): number[] {
  const result = [...m];
  result[12] = x;
  result[13] = y;
  result[14] = z;
  return result;
}

/**
 * Applies a rotation delta (Euler XYZ, radians) to a column-major 4x4 matrix.
 */
function applyRotationDelta(m: number[], axis: string, angle: number): number[] {
  const mat = new THREE.Matrix4();
  mat.fromArray(m);

  const rot = new THREE.Matrix4();
  if (axis === "x") rot.makeRotationX(angle);
  else if (axis === "y") rot.makeRotationY(angle);
  else rot.makeRotationZ(angle);

  // Rotate around the component origin: T * R_delta * R_existing
  // Extract translation, apply rotation to the rotation part, then reapply translation
  const pos = new THREE.Vector3(m[12], m[13], m[14]);
  mat.setPosition(0, 0, 0);
  mat.premultiply(rot);
  mat.setPosition(pos);

  const result: number[] = [];
  mat.toArray(result);
  return result;
}

interface DragState {
  axis: string;
  startPoint: THREE.Vector3;
  startTransform: number[];
  mode: "translate" | "rotate";
}

/**
 * TransformGizmo renders translate arrows or rotate rings for the currently
 * selected assembly component.  Drag interactions update the component
 * transform in real-time and persist on drag-end.
 *
 * Must be rendered inside a R3F <Canvas>.
 */
export function TransformGizmo() {
  const gizmoMode = useAssemblyStore((s) => s.gizmoMode);
  const selectedComponentId = useAssemblyStore((s) => s.selectedComponentId);
  const components = useAssemblyStore((s) => s.components);
  const moveComponent = useAssemblyStore((s) => s.moveComponent);
  const getComponentTransform = useAssemblyStore((s) => s.getComponentTransform);

  const groupRef = useRef<THREE.Group>(null);
  const dragRef = useRef<DragState | null>(null);
  const { camera, gl, raycaster } = useThree();

  // Find the selected component index
  const selectedIndex = useMemo(() => {
    if (!selectedComponentId) return -1;
    return components.findIndex((c) => c.id === selectedComponentId);
  }, [selectedComponentId, components]);

  // Current transform of the selected component
  const currentTransform = useMemo(() => {
    if (selectedIndex < 0) return null;
    return getComponentTransform(selectedIndex);
  }, [selectedIndex, getComponentTransform]);

  // Position the gizmo group at the component's position
  useFrame(() => {
    if (!groupRef.current || !currentTransform) return;
    const [x, y, z] = getTranslation(currentTransform);
    groupRef.current.position.set(x, y, z);
  });

  // Plane for raycasting drag events
  const planeRef = useRef(new THREE.Plane());
  const intersectionPoint = useRef(new THREE.Vector3());

  const projectOntoAxis = (point: THREE.Vector3, origin: THREE.Vector3, dir: THREE.Vector3): number => {
    const delta = point.clone().sub(origin);
    return delta.dot(dir);
  };

  const handlePointerDown = (axis: string) => (e: ThreeEvent<PointerEvent>) => {
    e.stopPropagation();
    if (selectedIndex < 0 || !gizmoMode || !currentTransform) return;

    const transform = getComponentTransform(selectedIndex);
    if (!transform) return;

    const origin = new THREE.Vector3(...getTranslation(transform));
    const axisDir = AXIS_DIRS[axis].clone();

    if (gizmoMode === "translate") {
      // Build a plane that contains the axis and faces the camera
      const camDir = camera.position.clone().sub(origin).normalize();
      const planeNormal = camDir.clone().sub(axisDir.clone().multiplyScalar(camDir.dot(axisDir))).normalize();
      if (planeNormal.length() < 0.001) {
        // Camera is looking along the axis, use a perpendicular plane
        planeNormal.set(0, 1, 0);
        if (Math.abs(axisDir.dot(planeNormal)) > 0.9) planeNormal.set(1, 0, 0);
      }
      planeRef.current.setFromNormalAndCoplanarPoint(planeNormal, origin);
    } else {
      // For rotation, use the axis as the plane normal
      planeRef.current.setFromNormalAndCoplanarPoint(axisDir, origin);
    }

    // Get initial intersection
    const pointer = new THREE.Vector2(
      (e.clientX / gl.domElement.clientWidth) * 2 - 1,
      -(e.clientY / gl.domElement.clientHeight) * 2 + 1,
    );
    raycaster.setFromCamera(pointer, camera);
    if (raycaster.ray.intersectPlane(planeRef.current, intersectionPoint.current)) {
      dragRef.current = {
        axis,
        startPoint: intersectionPoint.current.clone(),
        startTransform: [...transform],
        mode: gizmoMode,
      };

      gl.domElement.style.cursor = "grabbing";
      gl.domElement.setPointerCapture(e.pointerId);
    }
  };

  useEffect(() => {
    const domElement = gl.domElement;

    const handlePointerMove = (e: PointerEvent) => {
      if (!dragRef.current || selectedIndex < 0) return;

      const pointer = new THREE.Vector2(
        (e.clientX / domElement.clientWidth) * 2 - 1,
        -(e.clientY / domElement.clientHeight) * 2 + 1,
      );
      raycaster.setFromCamera(pointer, camera);

      if (!raycaster.ray.intersectPlane(planeRef.current, intersectionPoint.current)) return;

      const { axis, startPoint, startTransform, mode } = dragRef.current;

      if (mode === "translate") {
        const axisDir = AXIS_DIRS[axis];
        const startProjection = projectOntoAxis(startPoint, new THREE.Vector3(...getTranslation(startTransform)), axisDir);
        const currentProjection = projectOntoAxis(intersectionPoint.current, new THREE.Vector3(...getTranslation(startTransform)), axisDir);
        const delta = currentProjection - startProjection;

        const [sx, sy, sz] = getTranslation(startTransform);
        const newTransform = setTranslation(
          startTransform,
          sx + axisDir.x * delta,
          sy + axisDir.y * delta,
          sz + axisDir.z * delta,
        );
        moveComponent(selectedIndex, newTransform);
      } else {
        // Rotation: compute angle from start to current around the axis
        const origin = new THREE.Vector3(...getTranslation(startTransform));
        const v1 = startPoint.clone().sub(origin).normalize();
        const v2 = intersectionPoint.current.clone().sub(origin).normalize();
        let angle = Math.atan2(
          v1.clone().cross(v2).dot(AXIS_DIRS[axis]),
          v1.dot(v2),
        );
        // Snap to 5-degree increments if small movement
        if (Math.abs(angle) > 0.001) {
          const newTransform = applyRotationDelta(startTransform, axis, angle);
          moveComponent(selectedIndex, newTransform);
        }
      }
    };

    const handlePointerUp = () => {
      if (dragRef.current) {
        dragRef.current = null;
        domElement.style.cursor = "";
      }
    };

    domElement.addEventListener("pointermove", handlePointerMove);
    domElement.addEventListener("pointerup", handlePointerUp);

    return () => {
      domElement.removeEventListener("pointermove", handlePointerMove);
      domElement.removeEventListener("pointerup", handlePointerUp);
    };
  }, [camera, gl, raycaster, selectedIndex, moveComponent, getComponentTransform]);

  // Don't render if no mode, no selection, or no transform
  if (!gizmoMode || selectedIndex < 0 || !currentTransform) return null;

  if (gizmoMode === "translate") {
    return (
      <group ref={groupRef} data-testid="transform-gizmo">
        {(["x", "y", "z"] as const).map((axis) => {
          const dir = AXIS_DIRS[axis];
          const color = AXIS_COLORS[axis];
          // Quaternion to rotate from Y-up to the axis direction
          const quat = new THREE.Quaternion().setFromUnitVectors(
            new THREE.Vector3(0, 1, 0),
            dir,
          );
          const euler = new THREE.Euler().setFromQuaternion(quat);

          return (
            <group key={axis} rotation={euler} onPointerDown={handlePointerDown(axis)}>
              {/* Shaft */}
              <mesh position={[0, AXIS_LENGTH / 2, 0]}>
                <cylinderGeometry args={[SHAFT_RADIUS, SHAFT_RADIUS, AXIS_LENGTH, 8]} />
                <meshBasicMaterial color={color} />
              </mesh>
              {/* Arrow cone */}
              <mesh position={[0, AXIS_LENGTH + ARROW_CONE_HEIGHT / 2, 0]}>
                <coneGeometry args={[ARROW_CONE_RADIUS, ARROW_CONE_HEIGHT, 12]} />
                <meshBasicMaterial color={color} />
              </mesh>
              {/* Invisible wider hitbox for easier clicking */}
              <mesh position={[0, AXIS_LENGTH / 2, 0]} visible={false}>
                <cylinderGeometry args={[0.3, 0.3, AXIS_LENGTH + ARROW_CONE_HEIGHT, 8]} />
                <meshBasicMaterial transparent opacity={0} />
              </mesh>
            </group>
          );
        })}
      </group>
    );
  }

  // Rotation rings
  return (
    <group ref={groupRef} data-testid="transform-gizmo">
      {(["x", "y", "z"] as const).map((axis) => {
        const color = AXIS_COLORS[axis];
        const dir = AXIS_DIRS[axis];
        const quat = new THREE.Quaternion().setFromUnitVectors(
          new THREE.Vector3(0, 0, 1),
          dir,
        );
        const euler = new THREE.Euler().setFromQuaternion(quat);

        return (
          <group key={axis} rotation={euler} onPointerDown={handlePointerDown(axis)}>
            <mesh>
              <torusGeometry args={[RING_RADIUS, RING_TUBE, 16, 64]} />
              <meshBasicMaterial color={color} />
            </mesh>
            {/* Wider invisible hitbox ring */}
            <mesh visible={false}>
              <torusGeometry args={[RING_RADIUS, 0.25, 8, 32]} />
              <meshBasicMaterial transparent opacity={0} />
            </mesh>
          </group>
        );
      })}
    </group>
  );
}
