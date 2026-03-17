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
import { handleMeasureClick } from "./tools/measure-tool";
import { handleSketchFilletClick } from "./tools/sketch-fillet-tool";
import { handleSketchChamferClick } from "./tools/sketch-chamfer-tool";
import { handleBlockClick } from "./tools/block-tool";
import { handleTrimClick } from "./tools/trim-tool";
import { handleExtendClick } from "./tools/extend-tool";
import { handleOffsetClick } from "./tools/offset-tool";
import { handleMirrorClick } from "./tools/mirror-tool";
import { handleEllipseClick } from "./tools/ellipse-tool";
import { handlePolygonClick } from "./tools/polygon-tool";
import { handleSlotClick } from "./tools/slot-tool";
import { handleSketchLinearPatternClick } from "./tools/sketch-linear-pattern-tool";
import { handleSketchCircularPatternClick } from "./tools/sketch-circular-pattern-tool";
import { handleConvertEntitiesClick } from "./tools/convert-entities-tool";
import { DimensionInputOverlay } from "./dimension-input";
import { RelationsDialog } from "./relations-dialog";
import { findNearestPoint, findSnapTarget } from "./tools/snap-utils";
import { usePreferencesStore } from "@/stores/preferences-store";

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
  selectedIds?: Set<string>,
  dofStatus?: string | null,
  hoveredId?: string | null
): string {
  if (selectedIds?.has(entityId)) return "#00ccff"; // cyan = selected (SolidWorks)
  if (hoveredId === entityId) return "#ff8844"; // orange = hovered (SolidWorks)
  if (dofStatus === "over_constrained") return "#cc2222"; // red = over-constrained
  const isConstrained = constraints.some((c) => c.entityIds.includes(entityId));
  if (dofStatus === "fully_constrained") {
    return isConstrained ? "#111111" : "#6699ff"; // black = fully defined
  }
  // under-constrained or unknown
  return isConstrained ? "#2266cc" : "#6699ff"; // blue shades
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
  dofStatus,
  hoveredId,
  onPointClick,
  onPointHover,
  onDragStart,
  onDragMove,
  onDragEnd,
}: {
  entities: SketchEntity2D[];
  plane: SketchPlane;
  constraints: SketchConstraint2D[];
  selectedIds: Set<string>;
  dofStatus?: string | null;
  hoveredId?: string | null;
  onPointClick?: (id: string, e: ThreeEvent<MouseEvent>) => void;
  onPointHover?: (id: string | null) => void;
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
        const color = getEntityColor(pt.id, constraints, selectedIds, dofStatus, hoveredId);
        const isSelected = selectedIds.has(pt.id);
        return (
          <mesh
            key={pt.id}
            position={pos}
            onClick={(e) => {
              e.stopPropagation();
              onPointClick?.(pt.id, e);
            }}
            onPointerEnter={(e) => {
              e.stopPropagation();
              onPointHover?.(pt.id);
            }}
            onPointerLeave={() => {
              onPointHover?.(null);
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
            <sphereGeometry args={[isSelected ? 0.2 : hoveredId === pt.id ? 0.18 : 0.15, 12, 12]} />
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

function DimensionGraphics({
  entities,
  constraints,
  plane,
}: {
  entities: SketchEntity2D[];
  constraints: SketchConstraint2D[];
  plane: SketchPlane;
}) {
  const editDimension = useEditorStore((s) => s.editDimension);

  const dims = useMemo(() => {
    return constraints
      .filter((c) => c.value !== undefined)
      .map((c) => {
        const positions: SketchPoint2D[] = [];
        for (const eid of c.entityIds) {
          const pt = entities.find((e) => e.type === "point" && e.id === eid);
          if (pt && pt.type === "point") positions.push(pt.position);
        }

        if (positions.length < 2) return null;

        const p1 = positions[0]!;
        const p2 = positions[1]!;

        // Dimension text position: offset perpendicular to the line between points
        const mx = (p1.x + p2.x) / 2;
        const my = (p1.y + p2.y) / 2;
        const dx = p2.x - p1.x;
        const dy = p2.y - p1.y;
        const len = Math.sqrt(dx * dx + dy * dy);
        // Perpendicular offset (1.5 units away from line)
        const offset = 1.5;
        const nx = len > 0 ? -dy / len : 0;
        const ny = len > 0 ? dx / len : 1;
        const textPos: SketchPoint2D = { x: mx + nx * offset, y: my + ny * offset };

        // Extension line endpoints (from entity points to dimension line)
        const ext1End: SketchPoint2D = { x: p1.x + nx * offset, y: p1.y + ny * offset };
        const ext2End: SketchPoint2D = { x: p2.x + nx * offset, y: p2.y + ny * offset };

        // Build THREE lines
        const extLine1 = makeLine(
          [sketchToWorld(p1, plane), sketchToWorld(ext1End, plane)],
          "#666"
        );
        const extLine2 = makeLine(
          [sketchToWorld(p2, plane), sketchToWorld(ext2End, plane)],
          "#666"
        );
        const dimLine = makeLine(
          [sketchToWorld(ext1End, plane), sketchToWorld(ext2End, plane)],
          "#4488ff"
        );

        // Import formatDimensionWithPrefix dynamically to avoid circular deps
        let displayText: string;
        if (c.kind === "angle") {
          const degrees = c.value! * (180 / Math.PI);
          displayText = `${degrees.toFixed(1)}°`;
        } else {
          displayText = `${c.value!.toFixed(2)} mm`;
        }

        return {
          id: c.id,
          extLine1,
          extLine2,
          dimLine,
          textWorldPos: sketchToWorld(textPos, plane),
          displayText,
        };
      })
      .filter(Boolean) as {
        id: string;
        extLine1: THREE.Line;
        extLine2: THREE.Line;
        dimLine: THREE.Line;
        textWorldPos: THREE.Vector3;
        displayText: string;
      }[];
  }, [constraints, entities, plane]);

  return (
    <>
      {dims.map((d) => (
        <group key={d.id}>
          <primitive object={d.extLine1} />
          <primitive object={d.extLine2} />
          <primitive object={d.dimLine} />
          <Html position={d.textWorldPos} center>
            <div
              className="rounded bg-[var(--cad-bg-panel)]/90 px-1.5 py-0.5 text-[10px] text-[#4488ff] border border-[var(--cad-border)] shadow cursor-pointer whitespace-nowrap hover:bg-[var(--cad-bg-hover)] select-none"
              onDoubleClick={(e) => {
                e.stopPropagation();
                editDimension(d.id);
              }}
              title="Double-click to edit"
            >
              {d.displayText}
            </div>
          </Html>
        </group>
      ))}
    </>
  );
}

function PreviewLine({ plane }: { plane: SketchPlane }) {
  const pendingPoints = useEditorStore((s) => s.sketchSession?.pendingPoints);
  const cursorPos = useEditorStore((s) => s.sketchSession?.cursorPos);
  const activeTool = useEditorStore((s) => s.sketchSession?.activeTool);
  const entities = useEditorStore((s) => s.sketchSession?.entities ?? []);

  const preview = useMemo(() => {
    if (activeTool !== "line" || !pendingPoints?.length || !cursorPos)
      return null;
    const fromPt = pendingPoints[pendingPoints.length - 1]!;
    const { snapped, snapType } = getSnapPreview(fromPt, cursorPos);
    const start = sketchToWorld(fromPt, plane);
    const end = sketchToWorld(snapped, plane);
    const lineObj = makeDashedLine([start, end], "#88bbff");

    // Compute length and angle for numeric feedback
    const dx = snapped.x - fromPt.x;
    const dy = snapped.y - fromPt.y;
    const length = Math.sqrt(dx * dx + dy * dy);
    const angle = Math.atan2(dy, dx) * (180 / Math.PI);

    // Check coincident snap
    const coincidentSnap = findNearestPoint(cursorPos, entities);

    // Build cursor snap symbols (SolidWorks-style yellow symbols at cursor)
    const symbols: string[] = [];
    if (snapType === "h") symbols.push("—");
    if (snapType === "v") symbols.push("|");
    if (coincidentSnap) symbols.push("⊙");

    return { lineObj, end, length, angle, symbols };
  }, [activeTool, pendingPoints, cursorPos, plane, entities]);

  if (!preview) return null;

  return (
    <>
      <primitive object={preview.lineObj} />
      {/* Cursor snap symbols — yellow badges at cursor */}
      {preview.symbols.length > 0 && (
        <Html position={preview.end} center>
          <div className="pointer-events-none select-none flex gap-0.5 ml-4 -mt-4">
            {preview.symbols.map((sym, i) => (
              <span
                key={i}
                className="inline-flex items-center justify-center w-4 h-4 rounded-sm bg-[#ccaa00] text-black text-[10px] font-bold leading-none"
              >
                {sym}
              </span>
            ))}
          </div>
        </Html>
      )}
      {/* Numeric feedback — length and angle near cursor */}
      {preview.length > 0.1 && (
        <Html position={preview.end} center>
          <div className="pointer-events-none select-none ml-5 mt-2 rounded bg-black/70 px-1.5 py-0.5 text-[10px] text-white whitespace-nowrap font-mono">
            {preview.length.toFixed(1)} mm &nbsp; {Math.abs(preview.angle).toFixed(1)}°
          </div>
        </Html>
      )}
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

const CONSTRAINT_SYMBOLS: Record<string, string> = {
  horizontal: "H",
  vertical: "V",
  coincident: "\u2299", // ⊙
  perpendicular: "\u22A5", // ⊥
  parallel: "\u2225", // ∥
  equal: "=",
  fixed: "\uD83D\uDD12", // 🔒
  tangent: "T",
  collinear: "\u2261", // ≡
  midpoint: "M",
};

function OriginMarker({ plane }: { plane: SketchPlane }) {
  const lines = useMemo(() => {
    const size = 1.5;
    const hStart = sketchToWorld({ x: -size, y: 0 }, plane);
    const hEnd = sketchToWorld({ x: size, y: 0 }, plane);
    const vStart = sketchToWorld({ x: 0, y: -size }, plane);
    const vEnd = sketchToWorld({ x: 0, y: size }, plane);
    return {
      h: makeLine([hStart, hEnd], "#ff4040"),
      v: makeLine([vStart, vEnd], "#ff4040"),
    };
  }, [plane]);

  const center = useMemo(() => sketchToWorld({ x: 0, y: 0 }, plane), [plane]);

  return (
    <group>
      <primitive object={lines.h} />
      <primitive object={lines.v} />
      <mesh position={center}>
        <sphereGeometry args={[0.08, 8, 8]} />
        <meshBasicMaterial color="#ff4040" />
      </mesh>
    </group>
  );
}

function ConstraintSymbols({
  constraints,
  entities,
  plane,
}: {
  constraints: SketchConstraint2D[];
  entities: SketchEntity2D[];
  plane: SketchPlane;
}) {
  const symbols = useMemo(() => {
    // Track positions per entity to stack multiple symbols
    const entitySymbolCounts: Record<string, number> = {};

    return constraints
      .filter((c) => !c.value) // Skip dimension constraints (they have their own labels)
      .map((c) => {
        const symbol = CONSTRAINT_SYMBOLS[c.kind];
        if (!symbol) return null;

        const entityId = c.entityIds[0];
        if (!entityId) return null;
        const entity = entities.find((e) => e.id === entityId);
        if (!entity) return null;

        // Stack offset for multiple symbols on same entity
        const stackIndex = entitySymbolCounts[entityId] ?? 0;
        entitySymbolCounts[entityId] = stackIndex + 1;
        const stackOffset = stackIndex * 0.6;

        let pos: SketchPoint2D;
        if (entity.type === "point") {
          pos = { x: entity.position.x + 0.3, y: entity.position.y + 0.3 + stackOffset };
        } else if (entity.type === "line") {
          // Position at line midpoint for line constraints
          const startPt = entities.find((e) => e.id === entity.startId);
          const endPt = entities.find((e) => e.id === entity.endId);
          if (startPt?.type === "point" && endPt?.type === "point") {
            pos = {
              x: (startPt.position.x + endPt.position.x) / 2 + 0.3,
              y: (startPt.position.y + endPt.position.y) / 2 + 0.3 + stackOffset,
            };
          } else {
            return null;
          }
        } else {
          return null;
        }

        return { id: c.id, symbol, worldPos: sketchToWorld(pos, plane) };
      })
      .filter(Boolean) as { id: string; symbol: string; worldPos: THREE.Vector3 }[];
  }, [constraints, entities, plane]);

  return (
    <>
      {symbols.map((s) => (
        <Html key={s.id} position={s.worldPos} center>
          <div className="pointer-events-none select-none inline-flex items-center justify-center min-w-[14px] h-[14px] rounded-sm bg-[#1a4a1a] border border-[#22cc22]/40 text-[8px] font-bold text-[#22cc22] px-0.5">
            {s.symbol}
          </div>
        </Html>
      ))}
    </>
  );
}

function MeasureOverlay({ plane }: { plane: SketchPlane }) {
  const measureResult = useEditorStore((s) => s.sketchSession?.measureResult);

  if (!measureResult) return null;

  const fromWorld = sketchToWorld(measureResult.from, plane);
  const toWorld = sketchToWorld(measureResult.to, plane);
  const midWorld = sketchToWorld(
    { x: (measureResult.from.x + measureResult.to.x) / 2, y: (measureResult.from.y + measureResult.to.y) / 2 },
    plane
  );
  const lineObj = useMemo(
    () => makeDashedLine([fromWorld, toWorld], "#ffcc00"),
    [fromWorld, toWorld]
  );

  // Format using preferences
  const { unitSystem, dimensionDecimals } = usePreferencesStore.getState();
  const displayValue = measureResult.distance / (unitSystem === "mm" ? 1 : unitSystem === "cm" ? 10 : unitSystem === "m" ? 1000 : unitSystem === "in" ? 25.4 : 304.8);

  return (
    <group>
      <primitive object={lineObj} />
      <Html position={midWorld} center>
        <div className="pointer-events-none select-none rounded bg-[#332200] border border-[#ffcc00]/40 px-2 py-1 text-[11px] text-[#ffcc00] font-mono whitespace-nowrap shadow-lg">
          {displayValue.toFixed(dimensionDecimals)} {unitSystem}
        </div>
      </Html>
    </group>
  );
}

function ToolLabel({ plane }: { plane: SketchPlane }) {
  const activeTool = useEditorStore((s) => s.sketchSession?.activeTool);
  const cursorPos = useEditorStore((s) => s.sketchSession?.cursorPos);

  if (!activeTool || !cursorPos) return null;

  const toolNames: Record<string, string> = {
    line: "Line",
    rectangle: "Rect",
    circle: "Circle",
    arc: "Arc",
    dimension: "Dim",
    measure: "Measure",
    ellipse: "Ellipse",
    polygon: "Polygon",
    slot: "Slot",
    trim: "Trim",
    extend: "Extend",
    offset: "Offset",
    mirror: "Mirror",
    "sketch-fillet": "Fillet",
    "sketch-chamfer": "Chamfer",
    "sketch-linear-pattern": "Lin Pattern",
    "sketch-circular-pattern": "Circ Pattern",
    "convert-entities": "Convert",
    block: "Block",
  };
  const name = toolNames[activeTool] ?? activeTool;
  const worldPos = sketchToWorld({ x: cursorPos.x + 1, y: cursorPos.y - 1 }, plane);

  return (
    <Html position={worldPos} center>
      <div className="pointer-events-none select-none text-[9px] text-white/50 font-medium ml-2 mt-2">
        {name}
      </div>
    </Html>
  );
}

function SnapIndicator({ plane }: { plane: SketchPlane }) {
  const cursorPos = useEditorStore((s) => s.sketchSession?.cursorPos);
  const entities = useEditorStore((s) => s.sketchSession?.entities ?? []);
  const activeTool = useEditorStore((s) => s.sketchSession?.activeTool);

  const snap = useMemo(() => {
    if (!cursorPos || !activeTool) return null;
    const target = findSnapTarget(cursorPos, entities);
    if (target) return { id: target.targetId ?? "", position: target.position, snapType: target.type };
    return findNearestPoint(cursorPos, entities) ? { ...findNearestPoint(cursorPos, entities)!, snapType: "coincident" as const } : null;
  }, [cursorPos, entities, activeTool]);

  if (!snap) return null;

  const worldPos = sketchToWorld(snap.position, plane);
  const snapSymbol = snap.snapType === "midpoint" ? "△"
    : snap.snapType === "center" ? "⊕"
    : snap.snapType === "intersection" ? "✕"
    : "⊙";
  return (
    <group>
      <mesh position={worldPos}>
        <ringGeometry args={[0.25, 0.35, 16]} />
        <meshBasicMaterial color="#ffcc00" side={THREE.DoubleSide} />
      </mesh>
      <Html position={worldPos} center>
        <div className="pointer-events-none select-none text-[9px] font-bold text-[#ffcc00] ml-4 -mt-2">
          {snapSymbol}
        </div>
      </Html>
    </group>
  );
}

export function SketchOverlay() {
  const sketchSession = useEditorStore((s) => s.sketchSession);
  const setSketchCursorPos = useEditorStore((s) => s.setSketchCursorPos);
  const dofStatus = useEditorStore((s) => s.sketchDofStatus);
  // Entity selection and hover state (local to sketch overlay)
  const [selectedEntityIds, setSelectedEntityIds] = useState<string[]>([]);
  const [hoveredEntityId, setHoveredEntityId] = useState<string | null>(null);
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
      } else if (tool === "measure") {
        handleMeasureClick(sketchPt);
      } else if (tool === "ellipse") {
        handleEllipseClick(sketchPt);
      } else if (tool === "polygon") {
        handlePolygonClick(sketchPt);
      } else if (tool === "slot") {
        handleSlotClick(sketchPt);
      } else if (tool === "trim") {
        handleTrimClick(sketchPt);
      } else if (tool === "extend") {
        handleExtendClick(sketchPt);
      } else if (tool === "offset") {
        handleOffsetClick(sketchPt);
      } else if (tool === "mirror") {
        handleMirrorClick(sketchPt);
      } else if (tool === "sketch-fillet") {
        handleSketchFilletClick(sketchPt);
      } else if (tool === "sketch-chamfer") {
        handleSketchChamferClick(sketchPt);
      } else if (tool === "sketch-linear-pattern") {
        handleSketchLinearPatternClick(sketchPt);
      } else if (tool === "sketch-circular-pattern") {
        handleSketchCircularPatternClick(sketchPt);
      } else if (tool === "convert-entities") {
        handleConvertEntitiesClick(sketchPt);
      } else if (tool === "block") {
        handleBlockClick(sketchPt);
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
      const store = useEditorStore.getState();
      const session = store.sketchSession;
      const solver = store.sketchSolver;
      if (!session) return;

      // Update the solver's point position
      if (solver) {
        try {
          const idx = parseInt(id.replace(/\D+/g, ""), 10);
          solver.updatePoint(idx, pos.x, pos.y);
        } catch { /* ignore */ }
      }

      // Update local entity position for visual feedback
      const updatedEntities = session.entities.map((e) => {
        if (e.type === "point" && e.id === id) {
          return { ...e, position: pos };
        }
        return e;
      });
      useEditorStore.setState({
        sketchSession: { ...session, entities: updatedEntities },
      });

      // Re-solve with updated position
      store.solveSketch();
    },
    []
  );

  const handleDragEnd = useCallback((_id: string) => {
    // Final solve for clean convergence
    useEditorStore.getState().solveSketch();
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

      {/* Origin marker (red crosshair at 0,0) */}
      <OriginMarker plane={plane} />

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
        dofStatus={dofStatus}
        hoveredId={hoveredEntityId}
        onPointClick={handlePointClick}
        onPointHover={setHoveredEntityId}
        onDragStart={handleDragStart}
        onDragMove={handleDragMove}
        onDragEnd={handleDragEnd}
      />
      <SketchLines entities={entities} plane={plane} constraints={constraints} />
      <SketchCircles entities={entities} plane={plane} constraints={constraints} />
      <SketchArcs entities={entities} plane={plane} constraints={constraints} />

      {/* Constraint symbols (H, V, ⊙, etc.) */}
      <ConstraintSymbols constraints={constraints} entities={entities} plane={plane} />

      {/* Snap indicator (yellow ring when near existing point) */}
      <SnapIndicator plane={plane} />
      <MeasureOverlay plane={plane} />
      <ToolLabel plane={plane} />

      {/* Dimension labels */}
      <DimensionGraphics entities={entities} constraints={constraints} plane={plane} />

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
