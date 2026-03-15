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

export type FeatureParams =
  | { type: "placeholder" }
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
