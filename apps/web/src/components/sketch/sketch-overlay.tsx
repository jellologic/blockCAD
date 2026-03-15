import { useMemo, useCallback, useState, useRef } from "react";
import * as THREE from "three";
import { extend, type ThreeEvent } from "@react-three/fiber";
import { Html } from "@react-three/drei";
import { useEditorStore } from "@/stores/editor-store";
import type { SketchEntity2D, SketchConstraint2D, SketchPoint2D, SketchPlane } from "@blockCAD/kernel";
import { handleLineClick, getSnapPreview } from "./tools/line-tool";
import { handleRectangleClick } from "./tools/rectangle-tool";
import { handleCircleClick } from "./tools/circle-tool";
import { handleArcClick, circumcenter } from "./tools/arc-tool";
import { handleDimensionClick } from "./tools/dimension-tool";
import { DimensionInputOverlay } from "./dimension-input";
import { RelationsDialog } from "./relations-dialog";

// Extend R3F catalogue so we can use <line_> for THREE.Line (avoids SVG conflict)
extend({ Line_: THREE.Line });

declare module "@react-three/fiber" {
  interface ThreeElements {
    line_: ThreeElements["mesh"] & {
      geometry?: THREE.BufferGeometry;
      material?: THREE.Material;
    };
  }
}

function sketchToWorld(pt: SketchPoint2D, plane: SketchPlane): THREE.Vector3 {
  return new THREE.Vector3(
    plane.origin[0] + pt.x * plane.uAxis[0] + pt.y * plane.vAxis[0],
    plane.origin[1] + pt.x * plane.uAxis[1] + pt.y * plane.vAxis[1],
    plane.origin[2] + pt.x * plane.uAxis[2] + pt.y * plane.vAxis[2]
  );
}

function worldToSketch(
  worldPt: THREE.Vector3,
  plane: SketchPlane
): SketchPoint2D {
  const dx = worldPt.x - plane.origin[0];
  const dy = worldPt.y - plane.origin[1];
  const dz = worldPt.z - plane.origin[2];
  const u =
    dx * plane.uAxis[0] + dy * plane.uAxis[1] + dz * plane.uAxis[2];
  const v =
    dx * plane.vAxis[0] + dy * plane.vAxis[1] + dz * plane.vAxis[2];
  return { x: u, y: v };
}

function getPointPosition(
  id: string,
  entities: SketchEntity2D[]
): SketchPoint2D | null {
  const pt = entities.find((e) => e.type === "point" && e.id === id);
  if (pt && pt.type === "point") return pt.position;
  return null;
}

/** Compute rotation for gridHelper (default up = Y) to align with sketch plane normal */
function planeRotation(plane: SketchPlane): THREE.Euler {
  const normal = new THREE.Vector3(...plane.normal);
  const defaultNormal = new THREE.Vector3(0, 1, 0);
  const quat = new THREE.Quaternion().setFromUnitVectors(defaultNormal, normal);
  return new THREE.Euler().setFromQuaternion(quat);
}

/** Compute rotation for hit plane (default normal = Z+) to align with sketch plane normal */
function hitPlaneRotation(plane: SketchPlane): THREE.Euler {
  const normal = new THREE.Vector3(...plane.normal);
  const defaultNormal = new THREE.Vector3(0, 0, 1);
  const quat = new THREE.Quaternion().setFromUnitVectors(defaultNormal, normal);
  return new THREE.Euler().setFromQuaternion(quat);
}

function getEntityColor(
  entityId: string,
  constraints: SketchConstraint2D[],
  selectedIds?: Set<string>
): string {
  if (selectedIds?.has(entityId)) return "#ff8800"; // orange for selected
  const isConstrained = constraints.some((c) => c.entityIds.includes(entityId));
  return isConstrained ? "#2266cc" : "#6699ff"; // darker blue if constrained, lighter if not
}

function makeLine(points: THREE.Vector3[], color: string): THREE.Line {
  const geo = new THREE.BufferGeometry().setFromPoints(points);
  const mat = new THREE.LineBasicMaterial({ color });
  return new THREE.Line(geo, mat);
}

function makeDashedLine(points: THREE.Vector3[], color: string): THREE.Line {
  const geo = new THREE.BufferGeometry().setFromPoints(points);
  const mat = new THREE.LineDashedMaterial({
    color,
    dashSize: 0.5,
    gapSize: 0.3,
  });
  const line = new THREE.Line(geo, mat);
  line.computeLineDistances();
  return line;
}

function SketchPoints({
  entities,
  plane,
  constraints,
  selectedIds,
  onPointClick,
  onDragStart,
  onDragMove,
  onDragEnd,
}: {
  entities: SketchEntity2D[];
  plane: SketchPlane;
  constraints: SketchConstraint2D[];
  selectedIds: Set<string>;
  onPointClick?: (id: string, e: ThreeEvent<MouseEvent>) => void;
  onDragStart?: (id: string) => void;
  onDragMove?: (id: string, pos: SketchPoint2D) => void;
  onDragEnd?: (id: string) => void;
}) {
  const points = entities.filter(
    (e): e is Extract<SketchEntity2D, { type: "point" }> => e.type === "point"
  );
  const draggingRef = useRef<string | null>(null);

  return (
    <>
      {points.map((pt) => {
        const pos = sketchToWorld(pt.position, plane);
        const color = getEntityColor(pt.id, constraints, selectedIds);
        const isSelected = selectedIds.has(pt.id);
        return (
          <mesh
            key={pt.id}
            position={pos}
            onClick={(e) => {
              e.stopPropagation();
              onPointClick?.(pt.id, e);
            }}
            onPointerDown={(e) => {
              if (useEditorStore.getState().sketchSession?.activeTool === null) {
                e.stopPropagation();
                draggingRef.current = pt.id;
                onDragStart?.(pt.id);
                (e.target as any)?.setPointerCapture?.(e.pointerId);
              }
            }}
            onPointerMove={(e) => {
              if (draggingRef.current === pt.id) {
                e.stopPropagation();
                const sketchPt = worldToSketch(e.point, plane);
                onDragMove?.(pt.id, sketchPt);
              }
            }}
            onPointerUp={(e) => {
              if (draggingRef.current === pt.id) {
                e.stopPropagation();
                draggingRef.current = null;
                onDragEnd?.(pt.id);
              }
            }}
          >
            <sphereGeometry args={[isSelected ? 0.2 : 0.15, 12, 12]} />
            <meshBasicMaterial color={color} />
          </mesh>
        );
      })}
    </>
  );
}

function SketchLines({
  entities,
  plane,
  constraints,
}: {
  entities: SketchEntity2D[];
  plane: SketchPlane;
  constraints: SketchConstraint2D[];
}) {
  const lines = entities.filter(
    (e): e is Extract<SketchEntity2D, { type: "line" }> => e.type === "line"
  );

  const lineObjects = useMemo(() => {
    return lines.map((ln) => {
      const startPos = getPointPosition(ln.startId, entities);
      const endPos = getPointPosition(ln.endId, entities);
      if (!startPos || !endPos) return null;
      const start = sketchToWorld(startPos, plane);
      const end = sketchToWorld(endPos, plane);
      const color = getEntityColor(ln.id, constraints);
      return { id: ln.id, obj: makeLine([start, end], color) };
    }).filter(Boolean) as { id: string; obj: THREE.Line }[];
  }, [lines, entities, plane, constraints]);

  return (
    <>
      {lineObjects.map((l) => (
        <primitive key={l.id} object={l.obj} />
      ))}
    </>
  );
}

function SketchCircles({
  entities,
  plane,
  constraints,
}: {
  entities: SketchEntity2D[];
  plane: SketchPlane;
  constraints: SketchConstraint2D[];
}) {
  const circles = entities.filter(
    (e): e is Extract<SketchEntity2D, { type: "circle" }> => e.type === "circle"
  );

  const circleObjects = useMemo(() => {
    return circles.map((circle) => {
      const centerPos = getPointPosition(circle.centerId, entities);
      if (!centerPos) return null;
      const curve = new THREE.EllipseCurve(
        0, 0, circle.radius, circle.radius, 0, Math.PI * 2, false, 0
      );
      const points2d = curve.getPoints(64);
      const worldPoints = points2d.map((p) =>
        sketchToWorld({ x: centerPos.x + p.x, y: centerPos.y + p.y }, plane)
      );
      const color = getEntityColor(circle.id, constraints);
      return { id: circle.id, obj: makeLine(worldPoints, color) };
    }).filter(Boolean) as { id: string; obj: THREE.Line }[];
  }, [circles, entities, plane, constraints]);

  return (
    <>
      {circleObjects.map((c) => (
        <primitive key={c.id} object={c.obj} />
      ))}
    </>
  );
}

function SketchArcs({
  entities,
  plane,
  constraints,
}: {
  entities: SketchEntity2D[];
  plane: SketchPlane;
  constraints: SketchConstraint2D[];
}) {
  const arcs = entities.filter(
    (e): e is Extract<SketchEntity2D, { type: "arc" }> => e.type === "arc"
  );

  const arcObjects = useMemo(() => {
    return arcs.map((arc) => {
      const centerPos = getPointPosition(arc.centerId, entities);
      const startPos = getPointPosition(arc.startId, entities);
      const endPos = getPointPosition(arc.endId, entities);
      if (!centerPos || !startPos || !endPos) return null;

      const startAngle = Math.atan2(
        startPos.y - centerPos.y,
        startPos.x - centerPos.x
      );
      const endAngle = Math.atan2(
        endPos.y - centerPos.y,
        endPos.x - centerPos.x
      );

      const curve = new THREE.EllipseCurve(
        0, 0, arc.radius, arc.radius, startAngle, endAngle, false, 0
      );
      const points2d = curve.getPoints(64);
      const worldPoints = points2d.map((p) =>
        sketchToWorld({ x: centerPos.x + p.x, y: centerPos.y + p.y }, plane)
      );
      const color = getEntityColor(arc.id, constraints);
      return { id: arc.id, obj: makeLine(worldPoints, color) };
    }).filter(Boolean) as { id: string; obj: THREE.Line }[];
  }, [arcs, entities, plane, constraints]);

  return (
    <>
      {arcObjects.map((a) => (
        <primitive key={a.id} object={a.obj} />
      ))}
    </>
  );
}

function DimensionLabels({
  entities,
  constraints,
  plane,
}: {
  entities: SketchEntity2D[];
  constraints: SketchConstraint2D[];
  plane: SketchPlane;
}) {
  const labels = useMemo(() => {
    return constraints
      .filter((c) => c.value !== undefined)
      .map((c) => {
        // Find midpoint between the constraint's entity points
        const positions: SketchPoint2D[] = [];
        for (const eid of c.entityIds) {
          const pt = entities.find((e) => e.type === "point" && e.id === eid);
          if (pt && pt.type === "point") positions.push(pt.position);
        }
        if (positions.length < 2) return null;
        const mid: SketchPoint2D = {
          x: (positions[0]!.x + positions[1]!.x) / 2,
          y: (positions[0]!.y + positions[1]!.y) / 2 + 1, // offset slightly above
        };
        const worldPos = sketchToWorld(mid, plane);
        return { id: c.id, value: c.value!, worldPos };
      })
      .filter(Boolean) as { id: string; value: number; worldPos: THREE.Vector3 }[];
  }, [constraints, entities, plane]);

  return (
    <>
      {labels.map((label) => (
        <Html key={label.id} position={label.worldPos} center>
          <div className="rounded bg-[var(--cad-bg-panel)] px-1.5 py-0.5 text-[10px] text-[var(--cad-text-primary)] border border-[var(--cad-border)] shadow pointer-events-none whitespace-nowrap">
            {label.value.toFixed(2)} mm
          </div>
        </Html>
      ))}
    </>
  );
}

function PreviewLine({ plane }: { plane: SketchPlane }) {
  const pendingPoints = useEditorStore((s) => s.sketchSession?.pendingPoints);
  const cursorPos = useEditorStore((s) => s.sketchSession?.cursorPos);
  const activeTool = useEditorStore((s) => s.sketchSession?.activeTool);

  const { lineObj, snapObj } = useMemo(() => {
    if (activeTool !== "line" || !pendingPoints?.length || !cursorPos)
      return { lineObj: null, snapObj: null };
    const fromPt = pendingPoints[pendingPoints.length - 1]!;
    const { snapped, snapType } = getSnapPreview(fromPt, cursorPos);
    const start = sketchToWorld(fromPt, plane);
    const end = sketchToWorld(snapped, plane);
    const line = makeDashedLine([start, end], "#88bbff");

    // If snapping, show a thin dashed indicator along the snap axis
    let snap: THREE.Line | null = null;
    if (snapType) {
      const indicatorLen = 3;
      if (snapType === "h") {
        // Horizontal snap indicator
        const left = sketchToWorld({ x: snapped.x - indicatorLen, y: snapped.y }, plane);
        const right = sketchToWorld({ x: snapped.x + indicatorLen, y: snapped.y }, plane);
        snap = makeDashedLine([left, right], "#44ff88");
      } else {
        // Vertical snap indicator
        const top = sketchToWorld({ x: snapped.x, y: snapped.y + indicatorLen }, plane);
        const bottom = sketchToWorld({ x: snapped.x, y: snapped.y - indicatorLen }, plane);
        snap = makeDashedLine([top, bottom], "#44ff88");
      }
    }
    return { lineObj: line, snapObj: snap };
  }, [activeTool, pendingPoints, cursorPos, plane]);

  return (
    <>
      {lineObj && <primitive object={lineObj} />}
      {snapObj && <primitive object={snapObj} />}
    </>
  );
}

function PreviewArc({ plane }: { plane: SketchPlane }) {
  const pendingPoints = useEditorStore((s) => s.sketchSession?.pendingPoints);
  const cursorPos = useEditorStore((s) => s.sketchSession?.cursorPos);
  const activeTool = useEditorStore((s) => s.sketchSession?.activeTool);

  const previewObj = useMemo(() => {
    if (activeTool !== "arc" || !pendingPoints?.length || !cursorPos) return null;

    if (pendingPoints.length === 1) {
      // Show line from start to cursor
      const start = sketchToWorld(pendingPoints[0]!, plane);
      const end = sketchToWorld(cursorPos, plane);
      return makeDashedLine([start, end], "#88bbff");
    }

    if (pendingPoints.length === 2) {
      // Show arc preview through start, end, and cursor as mid-point
      const start = pendingPoints[0]!;
      const end = pendingPoints[1]!;
      const mid = cursorPos;
      const result = circumcenter(start, end, mid);
      if (!result) {
        // Collinear - just show a line
        return makeDashedLine(
          [sketchToWorld(start, plane), sketchToWorld(end, plane)],
          "#88bbff"
        );
      }
      const { center, radius } = result;
      const startAngle = Math.atan2(start.y - center.y, start.x - center.x);
      const endAngle = Math.atan2(end.y - center.y, end.x - center.x);
      const curve = new THREE.EllipseCurve(
        0, 0, radius, radius, startAngle, endAngle, false, 0
      );
      const points2d = curve.getPoints(64);
      const worldPoints = points2d.map((p) =>
        sketchToWorld({ x: center.x + p.x, y: center.y + p.y }, plane)
      );
      return makeDashedLine(worldPoints, "#88bbff");
    }

    return null;
  }, [activeTool, pendingPoints, cursorPos, plane]);

  if (!previewObj) return null;
  return <primitive object={previewObj} />;
}

function PreviewRectangle({ plane }: { plane: SketchPlane }) {
  const pendingPoints = useEditorStore((s) => s.sketchSession?.pendingPoints);
  const cursorPos = useEditorStore((s) => s.sketchSession?.cursorPos);
  const activeTool = useEditorStore((s) => s.sketchSession?.activeTool);

  const lineObj = useMemo(() => {
    if (activeTool !== "rectangle" || !pendingPoints?.length || !cursorPos)
      return null;
    const c1 = pendingPoints[0]!;
    const c2 = cursorPos;
    const corners = [
      sketchToWorld({ x: c1.x, y: c1.y }, plane),
      sketchToWorld({ x: c2.x, y: c1.y }, plane),
      sketchToWorld({ x: c2.x, y: c2.y }, plane),
      sketchToWorld({ x: c1.x, y: c2.y }, plane),
      sketchToWorld({ x: c1.x, y: c1.y }, plane),
    ];
    return makeDashedLine(corners, "#88bbff");
  }, [activeTool, pendingPoints, cursorPos, plane]);

  if (!lineObj) return null;
  return <primitive object={lineObj} />;
}

function PreviewCircle({ plane }: { plane: SketchPlane }) {
  const pendingPoints = useEditorStore((s) => s.sketchSession?.pendingPoints);
  const cursorPos = useEditorStore((s) => s.sketchSession?.cursorPos);
  const activeTool = useEditorStore((s) => s.sketchSession?.activeTool);

  const lineObj = useMemo(() => {
    if (activeTool !== "circle" || !pendingPoints?.length || !cursorPos)
      return null;
    const center = pendingPoints[0]!;
    const dx = cursorPos.x - center.x;
    const dy = cursorPos.y - center.y;
    const radius = Math.sqrt(dx * dx + dy * dy);
    if (radius < 0.01) return null;
    const curve = new THREE.EllipseCurve(
      0, 0, radius, radius, 0, Math.PI * 2, false, 0
    );
    const points2d = curve.getPoints(64);
    const worldPoints = points2d.map((p) =>
      sketchToWorld({ x: center.x + p.x, y: center.y + p.y }, plane)
    );
    return makeDashedLine(worldPoints, "#88bbff");
  }, [activeTool, pendingPoints, cursorPos, plane]);

  if (!lineObj) return null;
  return <primitive object={lineObj} />;
}

export function SketchOverlay() {
  const sketchSession = useEditorStore((s) => s.sketchSession);
  const setSketchCursorPos = useEditorStore((s) => s.setSketchCursorPos);
  // Entity selection state (local to sketch overlay)
  const [selectedEntityIds, setSelectedEntityIds] = useState<string[]>([]);
  const [showRelations, setShowRelations] = useState(false);
  const selectedIdsSet = useMemo(() => new Set(selectedEntityIds), [selectedEntityIds]);

  if (!sketchSession) return null;

  const { plane, entities, constraints } = sketchSession;
  const gridRotation = planeRotation(plane);
  const planeRot = hitPlaneRotation(plane);

  const handlePointerMove = useCallback(
    (event: ThreeEvent<PointerEvent>) => {
      event.stopPropagation();
      const sketchPt = worldToSketch(event.point, plane);
      setSketchCursorPos(sketchPt);
    },
    [plane, setSketchCursorPos]
  );

  const handleClick = useCallback(
    (event: ThreeEvent<MouseEvent>) => {
      event.stopPropagation();
      const sketchPt = worldToSketch(event.point, plane);
      const store = useEditorStore.getState();
      const tool = store.sketchSession?.activeTool;

      if (tool === "line") {
        handleLineClick(sketchPt);
      } else if (tool === "rectangle") {
        handleRectangleClick(sketchPt);
      } else if (tool === "circle") {
        handleCircleClick(sketchPt);
      } else if (tool === "arc") {
        handleArcClick(sketchPt);
      } else if (tool === "dimension") {
        handleDimensionClick(sketchPt);
      } else {
        // No tool active — clicking empty space clears selection
        setSelectedEntityIds([]);
        setShowRelations(false);
      }
    },
    [plane]
  );

  const handlePointClick = useCallback(
    (id: string, e: ThreeEvent<MouseEvent>) => {
      const store = useEditorStore.getState();
      if (store.sketchSession?.activeTool !== null) return; // Don't select while tool active

      // Shift-click for multi-select
      if (e.nativeEvent.shiftKey) {
        setSelectedEntityIds((prev) => {
          const next = prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id];
          if (next.length >= 2) setShowRelations(true);
          return next;
        });
      } else {
        setSelectedEntityIds([id]);
        setShowRelations(false);
      }
    },
    []
  );

  const handleDragStart = useCallback((_id: string) => {
    // Nothing special needed on drag start
  }, []);

  const handleDragMove = useCallback(
    (id: string, pos: SketchPoint2D) => {
      // Update the entity position in the store for visual feedback
      const store = useEditorStore.getState();
      const session = store.sketchSession;
      if (!session) return;
      const updatedEntities = session.entities.map((e) => {
        if (e.type === "point" && e.id === id) {
          return { ...e, position: pos };
        }
        return e;
      });
      useEditorStore.setState({
        sketchSession: { ...session, entities: updatedEntities },
      });
    },
    []
  );

  const handleDragEnd = useCallback((_id: string) => {
    // Drag complete — entity position is already updated in store
  }, []);

  return (
    <group>
      {/* Invisible hit plane for mouse projection */}
      <mesh
        visible={false}
        rotation={planeRot}
        onPointerMove={handlePointerMove}
        onClick={handleClick}
      >
        <planeGeometry args={[200, 200]} />
        <meshBasicMaterial
          side={THREE.DoubleSide}
          transparent
          opacity={0}
        />
      </mesh>

      {/* Sketch grid */}
      <gridHelper
        args={[40, 40, "#4488ff", "#334466"]}
        rotation={gridRotation}
      />

      {/* Rendered entities */}
      <SketchPoints
        entities={entities}
        plane={plane}
        constraints={constraints}
        selectedIds={selectedIdsSet}
        onPointClick={handlePointClick}
        onDragStart={handleDragStart}
        onDragMove={handleDragMove}
        onDragEnd={handleDragEnd}
      />
      <SketchLines entities={entities} plane={plane} constraints={constraints} />
      <SketchCircles entities={entities} plane={plane} constraints={constraints} />
      <SketchArcs entities={entities} plane={plane} constraints={constraints} />

      {/* Dimension labels */}
      <DimensionLabels entities={entities} constraints={constraints} plane={plane} />

      {/* Dimension input overlay */}
      <DimensionInputOverlay />

      {/* Relations dialog (shown when 2+ entities selected) */}
      {showRelations && (
        <Html center>
          <RelationsDialog
            open={showRelations}
            onClose={() => {
              setShowRelations(false);
              setSelectedEntityIds([]);
            }}
            selectedEntityIds={selectedEntityIds}
          />
        </Html>
      )}

      {/* Preview shapes */}
      <PreviewLine plane={plane} />
      <PreviewRectangle plane={plane} />
      <PreviewCircle plane={plane} />
      <PreviewArc plane={plane} />
    </group>
  );
}
