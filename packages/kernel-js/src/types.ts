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
  | "shell"
  // Batch 2
  | "variable_fillet"
  | "face_fillet"
  | "move_body"
  | "scale_body"
  // Batch 3
  | "hole_wizard"
  | "dome"
  | "rib"
  | "split_body"
  | "combine_bodies"
  | "curve_pattern"
  // Reference geometry
  | "datum_plane"
  | "reference_axis"
  | "reference_point"
  | "coordinate_system";

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

// --- Batch 2 param types ---

/** A control point specifying fillet radius at a position along an edge. */
export interface RadiusPoint {
  /** Parameter along edge, 0.0 = start, 1.0 = end. */
  parameter: number;
  /** Fillet radius at this parameter. */
  radius: number;
}

export interface VariableFilletParams {
  edge_indices: number[];
  /** At least 2 radius control points (start and end), sorted by parameter. */
  radius_points: RadiusPoint[];
  /** If true, use cubic interpolation; otherwise linear. */
  smooth_transition: boolean;
}

export interface FaceFilletParams {
  face_indices: number[];
  radius: number;
}

/** Spatial transformation kind (internally tagged via "kind" field). */
export type TransformKind =
  | { kind: "translate"; delta: Vec3 }
  | { kind: "rotate"; axis: Vec3; angle: number; center: Point3 }
  | { kind: "translate_rotate"; delta: Vec3; axis: Vec3; angle: number; center: Point3 };

export interface MoveBodyParams {
  transform: TransformKind;
  /** If true, create a copy (union of original + transformed). */
  copy: boolean;
}

export interface ScaleBodyParams {
  /** Uniform scale factor (must be > 0). */
  scale_factor: number;
  /** Center point for scaling. Defaults to origin if omitted. */
  center?: Point3;
  /** Non-uniform scale factors (x, y, z). Overrides scale_factor when present. */
  non_uniform?: Vec3;
  /** If true, keep original body and union the scaled copy. */
  copy: boolean;
}

// --- Batch 3 param types ---

/** Hole type for hole wizard. */
export type HoleType =
  | "Simple"
  | { Counterbore: { cbore_diameter: number; cbore_depth: number } }
  | { Countersink: { csink_diameter: number; csink_angle: number } };

export interface HoleParams {
  hole_type: HoleType;
  diameter: number;
  depth: number;
  position: Point3;
  direction: Vec3;
  through_all: boolean;
}

export interface DomeParams {
  face_index: number;
  height: number;
  elliptical: boolean;
  direction?: Vec3;
}

export interface RibParams {
  thickness: number;
  direction: Vec3;
  flip: boolean;
  both_sides: boolean;
}

export type SplitKeep = "Above" | "Below" | "Both";

export interface SplitParams {
  plane_origin: Point3;
  plane_normal: Vec3;
  keep: SplitKeep;
}

export type CombineOperation = "Add" | "Subtract" | "Common";

export interface CombineParams {
  operation: CombineOperation;
}

export interface CurvePatternParams {
  curve_points: Point3[];
  count: number;
  equal_spacing: boolean;
  align_to_curve: boolean;
}

// --- Reference geometry param types ---

/** How a datum plane is defined (internally tagged via serde rename_all snake_case). */
export type DatumPlaneKind =
  | { kind: "offset"; distance: number }
  | { kind: "angle"; axis: [number, number, number]; angle: number }
  | { kind: "three_point"; p1: [number, number, number]; p2: [number, number, number]; p3: [number, number, number] }
  | { kind: "face_plane"; face_index: number };

export interface DatumPlaneParams {
  kind: DatumPlaneKind;
  /** Base plane index (for offset/angle). None = standard XY plane. */
  base_plane_index?: number;
}

export interface ReferenceAxisParams {
  origin: Point3;
  direction: Vec3;
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

// --- Server-only operation param types ---

export interface DraftParams {
  face_indices: number[];
  pull_direction: Vec3;
  /** Draft angle in radians. */
  angle: number;
}

export interface SweepParams {
  segments?: number;
  /** Twist angle along the sweep (radians). */
  twist: number;
  guide_curves?: Array<{ points: Point3[] }>;
  orientation?: SweepOrientation;
}

export type SweepOrientation =
  | { mode: "follow_path" }
  | { mode: "keep_normal" }
  | { mode: "follow_path_and_guide" }
  | { mode: "twist_along_path"; total_twist: number };

export interface LoftParams {
  slices_per_span?: number;
  closed?: boolean;
  guide_curves?: Array<{ points: Point3[] }>;
  start_tangency?: TangencyCondition;
  end_tangency?: TangencyCondition;
}

export type TangencyCondition =
  | "None"
  | "Normal"
  | { Direction: Vec3 }
  | { Weight: { direction: Vec3; weight: number } };

// --- Extended result types ---

export interface StepExportOptions {
  schema: "AP203" | "AP214";
  author: string;
  organization: string;
}

export interface MassProperties {
  volume: number;
  surface_area: number;
  center_of_mass: Point3;
  inertia_tensor: number[][];
  principal_moments: Vec3;
  principal_axes: number[][];
  bbox_min: Point3;
  bbox_max: Point3;
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
  // Batch 2
  | { type: "variable_fillet"; params: VariableFilletParams }
  | { type: "face_fillet"; params: FaceFilletParams }
  | { type: "move_body"; params: MoveBodyParams }
  | { type: "scale_body"; params: ScaleBodyParams }
  // Batch 3
  | { type: "hole_wizard"; params: HoleParams }
  | { type: "dome"; params: DomeParams }
  | { type: "rib"; params: RibParams }
  | { type: "split_body"; params: SplitParams }
  | { type: "combine_bodies"; params: CombineParams }
  | { type: "curve_pattern"; params: CurvePatternParams }
  // Reference geometry
  | { type: "datum_plane"; params: DatumPlaneParams }
  | { type: "reference_axis"; params: ReferenceAxisParams }
  | { type: "reference_point"; params: ReferencePointParams }
  | { type: "coordinate_system"; params: CoordinateSystemParams }
  // Server-only operations
  | { type: "draft"; params: DraftParams }
  | { type: "sweep"; params: SweepParams }
  | { type: "loft"; params: LoftParams }
  // Fallback for unknown server-only params
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
  | "coincident"
  | "concentric"
  | { distance: { value: number } }
  | { angle: { value: number } };

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
