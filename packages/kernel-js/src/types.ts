/** TypeScript mirrors of Rust public types — matches .blockcad JSON format */

export type Vec3 = [number, number, number];
export type Point3 = [number, number, number];

// Client feature kinds (always available)
export type ClientFeatureKind =
  | "sketch"
  | "extrude"
  | "cut_extrude"
  | "revolve"
  | "cut_revolve"
  | "fillet"
  | "chamfer"
  | "linear_pattern"
  | "circular_pattern"
  | "mirror"
  | "shell";

// Server-only feature kinds
export type ServerFeatureKind =
  | "boolean_union"
  | "boolean_subtract"
  | "boolean_intersect"
  | "sweep"
  | "loft"
  | "draft";

export type FeatureKind = ClientFeatureKind | ServerFeatureKind;

export interface ExtrudeParams {
  direction: Vec3;
  depth: number;
  symmetric: boolean;
  draft_angle: number;
  end_condition?: "blind" | "through_all" | "up_to_next" | "up_to_surface" | "offset_from_surface" | "up_to_vertex";
  direction2_enabled?: boolean;
  depth2?: number;
  draft_angle2?: number;
  end_condition2?: "blind" | "through_all" | "up_to_next" | "up_to_surface" | "offset_from_surface" | "up_to_vertex";
  target_face_index?: number;
  surface_offset?: number;
  target_vertex_position?: [number, number, number];
  flip_side_to_cut?: boolean;
  cap_ends?: boolean;
  from_offset?: number;
  thin_feature?: boolean;
  thin_wall_thickness?: number;
  from_condition?: "sketch_plane" | "offset" | "surface" | "vertex";
  from_face_index?: number;
  from_vertex_position?: [number, number, number];
  contour_index?: number;
}

export interface RevolveParams {
  axis_origin: Point3;
  axis_direction: Vec3;
  angle: number;
  direction2_enabled?: boolean;
  angle2?: number;
  symmetric?: boolean;
  thin_feature?: boolean;
  thin_wall_thickness?: number;
  flip_side_to_cut?: boolean;
}

export interface FilletParams {
  edge_indices: number[];
  radius: number;
}

export type ChamferMode =
  | { type: "equal_distance"; distance: number }
  | { type: "two_distance"; distance1: number; distance2: number }
  | { type: "angle_distance"; distance: number; angle: number };

export interface ChamferParams {
  edge_indices: number[];
  distance: number;
  distance2?: number;
  mode?: ChamferMode;
}

export interface LinearPatternParams {
  direction: Vec3;
  spacing: number;
  count: number;
  direction2?: Vec3;
  spacing2?: number;
  count2?: number;
}

export interface CircularPatternParams {
  axis_origin: Point3;
  axis_direction: Vec3;
  count: number;
  total_angle: number;
}

export interface MirrorParams {
  plane_origin: Point3;
  plane_normal: Vec3;
}

export interface ShellParams {
  faces_to_remove: number[];
  thickness: number;
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

/** Rust Plane format (snake_case, returned by kernel roundtrip) */
export interface RustPlane {
  origin: Vec3;
  normal: Vec3;
  u_axis: Vec3;
  v_axis: Vec3;
}

/** Rust EntityStore serialization format */
export interface RustEntityStore {
  entries: Array<{ Occupied: { generation: number; value: unknown } } | string>;
  free_list: number[];
  len: number;
}

/** Sketch data as it comes back from kernel featureList (Rust serialization format) */
export interface RustSketchFeatureData {
  plane: RustPlane;
  entities: RustEntityStore;
  constraints: RustEntityStore;
  block_definitions?: unknown[];
  block_instances?: unknown[];
  construction_entities?: number[];
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
  | { type: "cut_extrude"; params: ExtrudeParams }
  | { type: "revolve"; params: RevolveParams }
  | { type: "cut_revolve"; params: RevolveParams }
  | { type: "fillet"; params: FilletParams }
  | { type: "chamfer"; params: ChamferParams }
  | { type: "linear_pattern"; params: LinearPatternParams }
  | { type: "circular_pattern"; params: CircularPatternParams }
  | { type: "mirror"; params: MirrorParams }
  | { type: "shell"; params: ShellParams }
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

// --- Batch 2 extended types ---

export interface RadiusPoint {
  parameter: number;
  radius: number;
}

export interface VariableFilletParams {
  edge_indices: number[];
  control_points: RadiusPoint[];
  smooth_transition?: boolean;
}

export interface FaceFilletParams {
  face_indices: number[];
  radius: number;
}

export type TransformKind = "translate" | "rotate" | "scale";

export interface MoveBodyParams {
  transform_type: TransformKind;
  translate_x?: number;
  translate_y?: number;
  translate_z?: number;
  rotate_axis_direction?: Vec3;
  rotate_angle?: number;
  rotate_center?: Point3;
  copy?: boolean;
}

export interface ScaleBodyParams {
  uniform?: boolean;
  scale_factor?: number;
  scale_x?: number;
  scale_y?: number;
  scale_z?: number;
  center?: Point3;
  copy?: boolean;
}

// --- Batch 3 extended types ---

export type HoleType = "simple" | "counterbore" | "countersink";

export interface HoleParams {
  hole_type: HoleType;
  diameter: number;
  depth: number;
  through_all?: boolean;
  position?: Point3;
  direction?: Vec3;
  cbore_diameter?: number;
  cbore_depth?: number;
  csink_diameter?: number;
  csink_angle?: number;
}

export interface DomeParams {
  face_index: number | null;
  height: number;
  elliptical?: boolean;
  direction?: Vec3 | null;
}

export interface RibParams {
  thickness: number;
  direction: Vec3;
  flip?: boolean;
  both_sides?: boolean;
}

export type SplitKeep = "both" | "top" | "bottom";

export interface SplitParams {
  plane_origin: Point3;
  plane_normal: Vec3;
  keep: SplitKeep;
}

export type CombineOperation = "add" | "subtract" | "intersect";

export interface CombineParams {
  operation: CombineOperation;
  body_indices: number[];
}

export interface CurvePatternParams {
  curve_entity_ids: string[];
  count: number;
  equal_spacing?: boolean;
}

// --- Reference geometry ---

export type DatumPlaneKind =
  | { type: "offset"; face_index: number; distance: number }
  | { type: "through_three_points"; points: [Point3, Point3, Point3] }
  | { type: "angle"; face_index: number; edge_index: number; angle: number };

export interface DatumPlaneParams {
  kind: DatumPlaneKind;
}

export interface ReferenceAxisParams {
  point1: Point3;
  point2: Point3;
}

export interface ReferencePointParams {
  position: Point3;
}

export interface CoordinateSystemParams {
  origin: Point3;
  x_axis: Vec3;
  y_axis: Vec3;
  z_axis: Vec3;
}

// --- Server-only operation types ---

export interface DraftParams {
  face_indices: number[];
  angle: number;
  pull_direction: Vec3;
}

export type SweepOrientation = "FollowPath" | "KeepNormal" | "FollowFirstSegment";

export interface SweepParams {
  guide_curves: unknown[];
  orientation: SweepOrientation;
  total_twist?: number;
}

export interface TangencyCondition {
  type: "None" | "Direction" | "NormalToProfile";
  direction?: Vec3;
}

export interface LoftParams {
  guide_curves: unknown[];
  start_tangency?: TangencyCondition;
  end_tangency?: TangencyCondition;
}

// --- Export / analysis types ---

export interface StepExportOptions {
  schema?: string;
  author?: string;
  organization?: string;
}

export interface MassProperties {
  volume: number;
  surface_area: number;
  center_of_mass: Point3;
  inertia_tensor: number[][];
  density?: number;
  mass?: number;
}

// --- Assembly types ---

export interface ComponentEntry {
  id: string;
  part_id: string;
  name: string;
  /** 4×4 column-major transform matrix */
  transform: number[];
  suppressed: boolean;
}

export type MateKind =
  // Standard mates
  | "coincident"
  | "concentric"
  | "parallel"
  | "perpendicular"
  | "tangent"
  | "lock"
  | { distance: { value: number } }
  | { angle: { value: number } }
  // Mechanical mates
  | "hinge"
  | { gear: { ratio: number } }
  | { screw: { pitch: number } }
  | { limit: { min: number; max: number } }
  // Advanced mates
  | "width"
  | "symmetric"
  | { rack_pinion: { pitch_radius: number } }
  | { cam: { eccentricity: number; base_radius: number } }
  | { slot: { axis: [number, number, number] } }
  | "universal_joint";

export interface MateEntry {
  id: string;
  kind: MateKind;
  component_a: string;
  component_b: string;
  suppressed: boolean;
}

export interface AssemblyDocument {
  $schema?: string;
  version: number;
  metadata: DocumentMetadata;
  parts: KernelDocument[];
  components: ComponentEntry[];
  mates: MateEntry[];
}
