import type { KernelClient } from "@blockCAD/kernel";

export interface SampleModel {
  id: string;
  name: string;
  description: string;
  icon: string;
  build: (kernel: KernelClient) => void;
}

// ── Helpers ──────────────────────────────────────────────────────

const FRONT_PLANE_DATA = {
  origin: [0, 0, 0] as [number, number, number],
  normal: [0, 0, 1] as [number, number, number],
  uAxis: [1, 0, 0] as [number, number, number],
  vAxis: [0, 1, 0] as [number, number, number],
};

const TOP_PLANE_DATA = {
  origin: [0, 0, 0] as [number, number, number],
  normal: [0, 1, 0] as [number, number, number],
  uAxis: [1, 0, 0] as [number, number, number],
  vAxis: [0, 0, 1] as [number, number, number],
};

/** Build a rectangular sketch on the given plane */
function rectSketch(
  plane: typeof FRONT_PLANE_DATA,
  w: number,
  h: number,
) {
  return {
    type: "sketch" as const,
    params: {
      plane,
      entities: [
        { type: "point", id: "se-0", position: { x: 0, y: 0 } },
        { type: "point", id: "se-1", position: { x: w, y: 0 } },
        { type: "point", id: "se-2", position: { x: w, y: h } },
        { type: "point", id: "se-3", position: { x: 0, y: h } },
        { type: "line", id: "se-4", startId: "se-0", endId: "se-1" },
        { type: "line", id: "se-5", startId: "se-1", endId: "se-2" },
        { type: "line", id: "se-6", startId: "se-2", endId: "se-3" },
        { type: "line", id: "se-7", startId: "se-3", endId: "se-0" },
      ],
      constraints: [
        { id: "sc-0", kind: "fixed", entityIds: ["se-0"] },
        { id: "sc-1", kind: "horizontal", entityIds: ["se-4"] },
        { id: "sc-2", kind: "horizontal", entityIds: ["se-6"] },
        { id: "sc-3", kind: "vertical", entityIds: ["se-5"] },
        { id: "sc-4", kind: "vertical", entityIds: ["se-7"] },
        { id: "sc-5", kind: "distance", entityIds: ["se-0", "se-1"], value: w },
        { id: "sc-6", kind: "distance", entityIds: ["se-1", "se-2"], value: h },
      ],
    },
  };
}

/** Build a circle sketch on the given plane */
function circleSketch(
  plane: typeof FRONT_PLANE_DATA,
  cx: number,
  cy: number,
  radius: number,
) {
  return {
    type: "sketch" as const,
    params: {
      plane,
      entities: [
        { type: "point", id: "se-0", position: { x: cx, y: cy } },
        { type: "circle", id: "se-1", centerId: "se-0", radius },
      ],
      constraints: [
        { id: "sc-0", kind: "fixed", entityIds: ["se-0"] },
      ],
    },
  };
}

/** Standard blind-extrude params */
function blindExtrude(depth: number, direction: [number, number, number] = [0, 0, 1]) {
  return {
    type: "extrude" as const,
    params: {
      direction,
      depth,
      symmetric: false,
      draft_angle: 0,
      end_condition: "blind",
      direction2_enabled: false,
      depth2: 0,
      draft_angle2: 0,
      end_condition2: "blind",
      from_offset: 0,
      thin_feature: false,
      thin_wall_thickness: 0,
      flip_side_to_cut: false,
      cap_ends: false,
      from_condition: "sketch_plane",
    },
  };
}

/** Standard blind-cut-extrude params */
function blindCutExtrude(depth: number, direction: [number, number, number] = [0, 0, -1]) {
  return {
    type: "cut_extrude" as const,
    params: {
      direction,
      depth,
      symmetric: false,
      draft_angle: 0,
      end_condition: "blind",
      direction2_enabled: false,
      depth2: 0,
      draft_angle2: 0,
      end_condition2: "blind",
      from_offset: 0,
      thin_feature: false,
      thin_wall_thickness: 0,
      flip_side_to_cut: false,
      cap_ends: false,
      from_condition: "sketch_plane",
    },
  };
}

// ── Sample Models ────────────────────────────────────────────────

export const SAMPLE_MODELS: SampleModel[] = [
  {
    id: "simple-box",
    name: "Simple Box",
    description: "10x10x5 rectangle sketch + blind extrude",
    icon: "📦",
    build: (kernel) => {
      kernel.addFeature("sketch", "Base Sketch", rectSketch(FRONT_PLANE_DATA, 10, 10));
      kernel.addFeature("extrude", "Extrude", blindExtrude(5));
    },
  },
  {
    id: "filleted-box",
    name: "Filleted Box",
    description: "Box with rounded bottom edges (radius 1)",
    icon: "🔘",
    build: (kernel) => {
      kernel.addFeature("sketch", "Base Sketch", rectSketch(FRONT_PLANE_DATA, 10, 10));
      kernel.addFeature("extrude", "Extrude", blindExtrude(5));
      kernel.addFeature("fillet", "Fillet", {
        type: "fillet",
        params: { edge_indices: [0, 1, 2, 3], radius: 1 },
      });
    },
  },
  {
    id: "hollow-shell",
    name: "Hollow Shell",
    description: "Box with top face removed + 1mm shell",
    icon: "📭",
    build: (kernel) => {
      kernel.addFeature("sketch", "Base Sketch", rectSketch(FRONT_PLANE_DATA, 10, 10));
      kernel.addFeature("extrude", "Extrude", blindExtrude(8));
      kernel.addFeature("shell", "Shell", {
        type: "shell",
        params: { faces_to_remove: [5], thickness: 1 },
      });
    },
  },
  {
    id: "chamfered-plate",
    name: "Chamfered Plate",
    description: "Wide plate with chamfered bottom edges",
    icon: "📐",
    build: (kernel) => {
      kernel.addFeature("sketch", "Base Sketch", rectSketch(FRONT_PLANE_DATA, 20, 15));
      kernel.addFeature("extrude", "Extrude", blindExtrude(3));
      kernel.addFeature("chamfer", "Chamfer", {
        type: "chamfer",
        params: { edge_indices: [0, 1, 2, 3], distance: 1 },
      });
    },
  },
  {
    id: "l-bracket",
    name: "L-Bracket",
    description: "L-shaped sketch extruded with filleted corner",
    icon: "🔧",
    build: (kernel) => {
      // L-shape: 6 points, 6 lines
      kernel.addFeature("sketch", "L Sketch", {
        type: "sketch",
        params: {
          plane: FRONT_PLANE_DATA,
          entities: [
            { type: "point", id: "se-0", position: { x: 0, y: 0 } },
            { type: "point", id: "se-1", position: { x: 15, y: 0 } },
            { type: "point", id: "se-2", position: { x: 15, y: 5 } },
            { type: "point", id: "se-3", position: { x: 5, y: 5 } },
            { type: "point", id: "se-4", position: { x: 5, y: 12 } },
            { type: "point", id: "se-5", position: { x: 0, y: 12 } },
            { type: "line", id: "se-6", startId: "se-0", endId: "se-1" },
            { type: "line", id: "se-7", startId: "se-1", endId: "se-2" },
            { type: "line", id: "se-8", startId: "se-2", endId: "se-3" },
            { type: "line", id: "se-9", startId: "se-3", endId: "se-4" },
            { type: "line", id: "se-10", startId: "se-4", endId: "se-5" },
            { type: "line", id: "se-11", startId: "se-5", endId: "se-0" },
          ],
          constraints: [
            { id: "sc-0", kind: "fixed", entityIds: ["se-0"] },
          ],
        },
      });
      kernel.addFeature("extrude", "Extrude", blindExtrude(5));
      kernel.addFeature("fillet", "Fillet", {
        type: "fillet",
        params: { edge_indices: [0, 1], radius: 1 },
      });
    },
  },
  {
    id: "hole-plate",
    name: "Hole Plate",
    description: "Plate with a circular through-hole",
    icon: "🕳️",
    build: (kernel) => {
      // Base plate
      kernel.addFeature("sketch", "Plate Sketch", rectSketch(FRONT_PLANE_DATA, 20, 15));
      kernel.addFeature("extrude", "Plate Extrude", blindExtrude(5));
      // Hole sketch on top face
      kernel.addFeature("sketch", "Hole Sketch", circleSketch(
        { ...FRONT_PLANE_DATA, origin: [0, 0, 5] },
        10, 7.5, 3,
      ));
      kernel.addFeature("cut_extrude", "Through Hole", blindCutExtrude(5));
    },
  },
  {
    id: "tall-cylinder",
    name: "Tall Cylinder",
    description: "Circle sketch extruded into a tall cylinder",
    icon: "🥫",
    build: (kernel) => {
      kernel.addFeature("sketch", "Circle Sketch", circleSketch(FRONT_PLANE_DATA, 0, 0, 5));
      kernel.addFeature("extrude", "Extrude", blindExtrude(15));
    },
  },
  {
    id: "stepped-block",
    name: "Stepped Block",
    description: "Two extrusions stacked on different planes",
    icon: "🪜",
    build: (kernel) => {
      // Bottom step: 15x15x5
      kernel.addFeature("sketch", "Base Sketch", rectSketch(FRONT_PLANE_DATA, 15, 15));
      kernel.addFeature("extrude", "Base Extrude", blindExtrude(5));
      // Top step: 8x8x5, offset on top of the first
      kernel.addFeature("sketch", "Top Sketch", rectSketch(
        { ...FRONT_PLANE_DATA, origin: [0, 0, 5] },
        8, 8,
      ));
      kernel.addFeature("extrude", "Top Extrude", blindExtrude(5));
    },
  },
];
