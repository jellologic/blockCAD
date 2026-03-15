/** TypeScript mirrors of Rust public types — matches .blockcad JSON format */

export type Vec3 = [number, number, number];
export type Point3 = [number, number, number];

// Client feature kinds (always available)
export type ClientFeatureKind =
  | "sketch"
  | "extrude"
  | "revolve"
  | "fillet"
  | "chamfer";

// Server-only feature kinds
export type ServerFeatureKind =
  | "boolean_union"
  | "boolean_subtract"
  | "boolean_intersect"
  | "sweep"
  | "loft"
  | "shell"
  | "draft"
  | "linear_pattern"
  | "circular_pattern"
  | "mirror";

export type FeatureKind = ClientFeatureKind | ServerFeatureKind;

export interface ExtrudeParams {
  direction: Vec3;
  depth: number;
  symmetric: boolean;
  draft_angle: number;
}

export interface RevolveParams {
  axis_origin: Point3;
  axis_direction: Vec3;
  angle: number;
}

export interface FilletParams {
  edge_indices: number[];
  radius: number;
}

export interface ChamferParams {
  edge_indices: number[];
  distance: number;
  distance2?: number;
}

// --- Sketch 2D types ---

export interface SketchPoint2D {
  x: number;
  y: number;
}

export type SketchPlaneId = "front" | "top" | "right";

export interface SketchPlane {
  origin: Vec3;
  normal: Vec3;
  uAxis: Vec3;
  vAxis: Vec3;
}

export type SketchEntity2D =
  | { type: "point"; id: string; position: SketchPoint2D }
  | { type: "line"; id: string; startId: string; endId: string }
  | { type: "circle"; id: string; centerId: string; radius: number }
  | { type: "arc"; id: string; centerId: string; startId: string; endId: string; radius: number };

export interface SketchConstraint2D {
  id: string;
  kind: string;
  entityIds: string[];
  value?: number;
}

export interface SketchFeatureData {
  plane: SketchPlane;
  entities: SketchEntity2D[];
  constraints: SketchConstraint2D[];
}

/** Standard reference planes */
export const FRONT_PLANE: SketchPlane = {
  origin: [0, 0, 0],
  normal: [0, 0, 1],
  uAxis: [1, 0, 0],
  vAxis: [0, 1, 0],
};

export const TOP_PLANE: SketchPlane = {
  origin: [0, 0, 0],
  normal: [0, 1, 0],
  uAxis: [1, 0, 0],
  vAxis: [0, 0, 1],
};

export const RIGHT_PLANE: SketchPlane = {
  origin: [0, 0, 0],
  normal: [1, 0, 0],
  uAxis: [0, 1, 0],
  vAxis: [0, 0, 1],
};

export type FeatureParams =
  | { type: "placeholder" }
  | { type: "sketch"; params: SketchFeatureData }
  | { type: "extrude"; params: ExtrudeParams }
  | { type: "revolve"; params: RevolveParams }
  | { type: "fillet"; params: FilletParams }
  | { type: "chamfer"; params: ChamferParams }
  // Server-only params stored as opaque JSON
  | { type: ServerFeatureKind; params: Record<string, unknown> };

export interface DocumentMetadata {
  name: string;
  description?: string;
  created_at?: string;
  modified_at?: string;
}

export interface FeatureEntry {
  id: string;
  name: string;
  type: FeatureKind;
  suppressed: boolean;
  params: FeatureParams;
}

export interface KernelDocument {
  $schema?: string;
  version: number;
  metadata: DocumentMetadata;
  features: FeatureEntry[];
}
