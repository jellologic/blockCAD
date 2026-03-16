import { useMemo } from "react";
import * as THREE from "three";
import { useEditorStore } from "@/stores/editor-store";
import type { SketchEntity2D, SketchPlane } from "@blockCAD/kernel";

/**
 * Extracts a closed polygon from sketch entities by walking the line chain.
 * Returns 2D points in sketch-plane coordinates.
 */
function extractProfilePoints(entities: SketchEntity2D[]): { x: number; y: number }[] | null {
  const points = entities.filter((e) => e.type === "point");
  const lines = entities.filter((e) => e.type === "line");

  if (lines.length < 3) return null;

  // Build adjacency: pointId → connected lineIds
  const adj = new Map<string, string[]>();
  for (const line of lines) {
    if (line.type !== "line") continue;
    adj.set(line.startId, [...(adj.get(line.startId) || []), line.id]);
    adj.set(line.endId, [...(adj.get(line.endId) || []), line.id]);
  }

  // Walk the line chain starting from the first line
  const firstLine = lines[0];
  if (firstLine.type !== "line") return null;

  const visited = new Set<string>();
  const orderedPointIds: string[] = [firstLine.startId];
  visited.add(firstLine.id);
  let currentPointId = firstLine.endId;

  while (currentPointId !== orderedPointIds[0]) {
    orderedPointIds.push(currentPointId);
    const connectedLines = adj.get(currentPointId) || [];
    const nextLine = connectedLines
      .map((lid) => lines.find((l) => l.id === lid)!)
      .find((l) => l && !visited.has(l.id));

    if (!nextLine || nextLine.type !== "line") break;
    visited.add(nextLine.id);
    currentPointId = nextLine.startId === currentPointId ? nextLine.endId : nextLine.startId;
  }

  if (orderedPointIds.length < 3) return null;

  // Map pointIds to positions
  return orderedPointIds.map((pid) => {
    const pt = points.find((p) => p.id === pid);
    return pt && pt.type === "point" ? pt.position : { x: 0, y: 0 };
  });
}

/**
 * Project 2D sketch point to 3D using the sketch plane basis vectors.
 */
function to3D(pt: { x: number; y: number }, plane: SketchPlane): THREE.Vector3 {
  return new THREE.Vector3(
    plane.origin[0] + pt.x * plane.uAxis[0] + pt.y * plane.vAxis[0],
    plane.origin[1] + pt.x * plane.uAxis[1] + pt.y * plane.vAxis[1],
    plane.origin[2] + pt.x * plane.uAxis[2] + pt.y * plane.vAxis[2],
  );
}

/**
 * Renders a semi-transparent preview mesh of the extrude operation.
 * Shown in the viewport while the extrude panel is active.
 */
export function ExtrudePreview() {
  const activeOperation = useEditorStore((s) => s.activeOperation);
  const features = useEditorStore((s) => s.features);

  const geometry = useMemo(() => {
    if (!activeOperation || (activeOperation.type !== "extrude" && activeOperation.type !== "cut_extrude")) return null;

    // Find the latest sketch feature
    const sketchFeature = [...features].reverse().find(
      (f) => f.type === "sketch" && f.params.type === "sketch"
    );
    if (!sketchFeature || sketchFeature.params.type !== "sketch") return null;

    const { plane, entities } = sketchFeature.params.params;
    const profile2D = extractProfilePoints(entities);
    if (!profile2D || profile2D.length < 3) return null;

    const {
      direction = [0, 0, 1],
      depth: rawDepth = 10,
      symmetric = false,
      draft_angle = 0,
      end_condition = "blind",
      direction2_enabled = false,
      depth2: rawDepth2 = 10,
      draft_angle2 = 0,
      end_condition2 = "blind",
      from_offset = 0,
      thin_feature = false,
      thin_wall_thickness = 1,
    } = activeOperation.params;

    const depth = end_condition === "through_all" ? 100
      : (end_condition === "up_to_next" || end_condition === "up_to_surface" || end_condition === "offset_from_surface" || end_condition === "up_to_vertex") ? 50
      : rawDepth;
    const depth2 = end_condition2 === "through_all" ? 100
      : (end_condition2 === "up_to_next" || end_condition2 === "up_to_surface" || end_condition2 === "offset_from_surface" || end_condition2 === "up_to_vertex") ? 50
      : rawDepth2;

    const dir = new THREE.Vector3(...(direction as [number, number, number])).normalize();
    const n = profile2D.length;

    // Compute from-offset shift
    const fromShift = dir.clone().multiplyScalar(from_offset);

    // Compute bottom and top offsets for direction 1
    let bottomOffset: THREE.Vector3;
    let topOffset: THREE.Vector3;
    if (symmetric) {
      const half = depth / 2;
      bottomOffset = dir.clone().multiplyScalar(-half).add(fromShift);
      topOffset = dir.clone().multiplyScalar(half).add(fromShift);
    } else {
      bottomOffset = fromShift.clone();
      topOffset = dir.clone().multiplyScalar(depth).add(fromShift);
    }

    // If direction2 is enabled, extend bottom in the opposite direction
    let bottom2Offset: THREE.Vector3 | null = null;
    if (direction2_enabled && !symmetric) {
      bottom2Offset = dir.clone().multiplyScalar(-depth2).add(fromShift);
    }

    // Project profile to 3D and compute bottom points
    const bottomPts = profile2D.map((pt) => to3D(pt, plane).add(bottomOffset));

    // Helper to compute top/draft points given base points, extrude vector, draft angle, and effective depth
    const computeDraftedPts = (
      basePts: THREE.Vector3[],
      extrudeVec: THREE.Vector3,
      draftAngle: number,
      effectiveDepth: number
    ): THREE.Vector3[] => {
      const draftRad = (Math.abs(draftAngle) * Math.PI) / 180;
      if (draftRad > 1e-6) {
        const cx = basePts.reduce((s, p) => s + p.x, 0) / n;
        const cy = basePts.reduce((s, p) => s + p.y, 0) / n;
        const cz = basePts.reduce((s, p) => s + p.z, 0) / n;
        const dOffset = effectiveDepth * Math.tan(draftRad);
        const sign = draftAngle < 0 ? -1 : 1;
        return basePts.map((bp) => {
          const toCentroid = new THREE.Vector3(cx - bp.x, cy - bp.y, cz - bp.z);
          const dist = toCentroid.length();
          if (dist < 1e-9) return bp.clone().add(extrudeVec);
          const inward = toCentroid.normalize().multiplyScalar(dOffset * sign);
          return bp.clone().add(extrudeVec).add(inward);
        });
      } else {
        return basePts.map((bp) => bp.clone().add(extrudeVec));
      }
    };

    // Direction 1 top points
    const extrudeVec1 = topOffset.clone().sub(bottomOffset);
    const topPts = computeDraftedPts(bottomPts, extrudeVec1, draft_angle, depth);

    // Direction 2 bottom points (extending in opposite direction)
    let bottom2Pts: THREE.Vector3[] | null = null;
    if (bottom2Offset) {
      const basePtsForDir2 = profile2D.map((pt) => to3D(pt, plane).add(fromShift));
      const extrudeVec2 = bottom2Offset.clone().sub(fromShift);
      bottom2Pts = computeDraftedPts(basePtsForDir2, extrudeVec2, draft_angle2, depth2);
    }

    // Build geometry: triangulate the sides + caps
    const positions: number[] = [];
    const normals: number[] = [];

    // Helper to add a triangle
    const addTri = (a: THREE.Vector3, b: THREE.Vector3, c: THREE.Vector3) => {
      const ab = b.clone().sub(a);
      const ac = c.clone().sub(a);
      const n = ab.cross(ac).normalize();
      positions.push(a.x, a.y, a.z, b.x, b.y, b.z, c.x, c.y, c.z);
      normals.push(n.x, n.y, n.z, n.x, n.y, n.z, n.x, n.y, n.z);
    };

    // Direction 1: Side faces (quads as 2 triangles)
    for (let i = 0; i < n; i++) {
      const j = (i + 1) % n;
      addTri(bottomPts[i], bottomPts[j], topPts[j]);
      addTri(bottomPts[i], topPts[j], topPts[i]);
    }

    // Direction 1: Bottom cap (fan triangulation)
    for (let i = 1; i < n - 1; i++) {
      addTri(bottomPts[0], bottomPts[i + 1], bottomPts[i]); // reversed winding
    }

    // Direction 1: Top cap (fan triangulation)
    for (let i = 1; i < n - 1; i++) {
      addTri(topPts[0], topPts[i], topPts[i + 1]);
    }

    // Thin feature: inner wall geometry
    if (thin_feature && thin_wall_thickness > 0) {
      // Compute inner profile by offsetting each vertex toward centroid
      const offsetPts = (pts: THREE.Vector3[], thickness: number) => {
        const cx = pts.reduce((s, p) => s + p.x, 0) / n;
        const cy = pts.reduce((s, p) => s + p.y, 0) / n;
        const cz = pts.reduce((s, p) => s + p.z, 0) / n;
        return pts.map(p => {
          const toCentroid = new THREE.Vector3(cx - p.x, cy - p.y, cz - p.z);
          const dist = toCentroid.length();
          if (dist < 1e-9) return p.clone();
          return p.clone().add(toCentroid.normalize().multiplyScalar(thickness));
        });
      };
      const innerBottomPts = offsetPts(bottomPts, thin_wall_thickness);
      const innerTopPts = offsetPts(topPts, thin_wall_thickness);

      // Inner side walls (reversed winding)
      for (let i = 0; i < n; i++) {
        const j = (i + 1) % n;
        addTri(innerBottomPts[j], innerBottomPts[i], innerTopPts[i]);
        addTri(innerBottomPts[j], innerTopPts[i], innerTopPts[j]);
      }
      // Note: caps are already rendered as the outer boundary - they'll look like annular rings
      // since both outer and inner geometry overlap visually in the transparent preview
    }

    // Direction 2: if enabled, add the opposite extrusion
    if (bottom2Pts) {
      const basePtsForDir2 = profile2D.map((pt) => to3D(pt, plane).add(fromShift));
      // Side faces
      for (let i = 0; i < n; i++) {
        const j = (i + 1) % n;
        addTri(basePtsForDir2[i], bottom2Pts[j], basePtsForDir2[j]);
        addTri(basePtsForDir2[i], bottom2Pts[i], bottom2Pts[j]);
      }
      // Cap at the far end of direction 2
      for (let i = 1; i < n - 1; i++) {
        addTri(bottom2Pts[0], bottom2Pts[i], bottom2Pts[i + 1]);
      }
    }

    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.BufferAttribute(new Float32Array(positions), 3));
    geo.setAttribute("normal", new THREE.BufferAttribute(new Float32Array(normals), 3));
    return geo;
  }, [activeOperation, features]);

  if (!geometry) return null;

  const isCut = activeOperation?.type === "cut_extrude";

  return (
    <mesh geometry={geometry} renderOrder={2}>
      <meshStandardMaterial
        color={isCut ? "#cc4444" : "#44cc88"}
        transparent
        opacity={0.35}
        depthWrite={false}
        side={THREE.DoubleSide}
      />
    </mesh>
  );
}
