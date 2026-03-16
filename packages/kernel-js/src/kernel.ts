import { KernelHandle } from "@blockCAD/kernel-wasm";
import { parseMeshBytes, type MeshData } from "./mesh";
import type { FeatureEntry, FeatureParams } from "./types";
import { KernelError } from "./errors";

/**
 * Convert frontend SketchPlane (camelCase) to Rust Plane (snake_case).
 */
function transformPlane(plane: any): any {
  if (!plane) return plane;
  return {
    origin: plane.origin,
    normal: plane.normal,
    u_axis: plane.uAxis ?? plane.u_axis,
    v_axis: plane.vAxis ?? plane.v_axis,
  };
}

/**
 * Convert frontend SketchEntity2D to Rust SketchEntity serde format.
 * Rust serializes as externally-tagged: {Point: {position: [x,y]}}
 */
function transformSketchEntity(entity: any): any {
  switch (entity.type) {
    case "point":
      return { Point: { position: [entity.position.x, entity.position.y] } };
    case "line":
      return {
        Line: {
          start: parseEntityId(entity.startId),
          end: parseEntityId(entity.endId),
        },
      };
    case "circle":
      return {
        Circle: {
          center: parseEntityId(entity.centerId),
          radius: entity.radius,
        },
      };
    case "arc":
      return {
        Arc: {
          center: parseEntityId(entity.centerId),
          start: parseEntityId(entity.startId),
          end: parseEntityId(entity.endId),
        },
      };
    default:
      return entity;
  }
}

/**
 * Convert frontend SketchConstraint2D to Rust Constraint serde format.
 */
function transformSketchConstraint(constraint: any): any {
  const entityIds = (constraint.entityIds ?? []).map((id: string) =>
    parseEntityId(id)
  );

  let kind: any;
  switch (constraint.kind) {
    case "distance":
      kind = { Distance: { value: constraint.value ?? 0 } };
      break;
    case "angle":
      kind = { Angle: { value: constraint.value ?? 0, supplementary: false } };
      break;
    case "radius":
      kind = { Radius: { value: constraint.value ?? 0 } };
      break;
    case "diameter":
      kind = { Diameter: { value: constraint.value ?? 0 } };
      break;
    default:
      // Simple constraints like Horizontal, Vertical, Coincident, etc.
      kind = constraint.kind.charAt(0).toUpperCase() + constraint.kind.slice(1);
      break;
  }

  return {
    kind,
    entities: entityIds,
    driven: false,
  };
}

/**
 * Parse entity ID string (e.g. "se-3") to Rust EntityId format.
 * Rust EntityId serializes as {index: N, generation: 0}.
 */
function parseEntityId(id: string): { index: number; generation: number } {
  const num = parseInt(id.replace(/\D+/g, ""), 10);
  return { index: isNaN(num) ? 0 : num, generation: 0 };
}

/**
 * Wrap an array of items into Rust EntityStore serde format.
 * EntityStore serializes as {entries: [{Occupied: {generation, value}}], free_list: [], len: N}
 */
function toEntityStore(items: any[]): any {
  return {
    entries: items.map((value) => ({
      Occupied: { generation: 0, value },
    })),
    free_list: [],
    len: items.length,
  };
}

/**
 * Transform frontend FeatureParams to Rust FeatureParams serde format.
 * Handles the schema differences between TS camelCase and Rust snake_case.
 */
/**
 * Normalize entities/constraints that may be either a flat SketchEntity2D[]
 * (from the frontend editor session) or an EntityStore object
 * (from a kernel roundtrip via featureList JSON).
 */
function normalizeEntities(raw: any): any[] {
  if (Array.isArray(raw)) return raw;
  // EntityStore format: { entries: [{ Occupied: { value, generation } } | "Free"], ... }
  if (raw && Array.isArray(raw.entries)) {
    return raw.entries
      .filter((e: any) => e && typeof e === "object" && e.Occupied)
      .map((e: any) => e.Occupied.value);
  }
  return [];
}

function transformParams(kind: string, params: FeatureParams): any {
  if (kind === "sketch" && params.type === "sketch") {
    const p = params.params;
    const entities = normalizeEntities(p.entities);
    const constraints = normalizeEntities(p.constraints);

    // If entities are already in Rust format (from kernel roundtrip), pass through
    const isRustFormat = entities.length > 0 && entities[0] && typeof entities[0] === "object"
      && ("Point" in entities[0] || "Line" in entities[0] || "Circle" in entities[0] || "Arc" in entities[0]);

    return {
      type: "sketch",
      params: {
        plane: transformPlane(p.plane),
        entities: isRustFormat
          ? toEntityStore(entities)
          : toEntityStore(entities.map(transformSketchEntity)),
        constraints: isRustFormat
          ? toEntityStore(constraints)
          : toEntityStore(constraints.map(transformSketchConstraint)),
      },
    };
  }
  // For non-sketch params, pass through as-is (extrude, revolve, etc. use snake_case already)
  return params;
}

/**
 * High-level TypeScript client wrapping the WASM KernelHandle.
 * Provides ergonomic methods for feature tree manipulation.
 */
export class KernelClient {
  private handle: KernelHandle;

  constructor() {
    this.handle = new KernelHandle();
  }

  get featureCount(): number {
    return this.handle.feature_count();
  }

  get cursor(): number {
    return this.handle.cursor();
  }

  addFeature(kind: string, _name: string, params: FeatureParams): string {
    try {
      const transformed = transformParams(kind, params);
      return this.handle.add_feature(kind, JSON.stringify(transformed));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  tessellate(chordTolerance: number = 0.01, angleTolerance: number = 0.5): MeshData {
    try {
      const bytes = this.handle.tessellate(chordTolerance, angleTolerance);
      // Copy to a fresh ArrayBuffer to ensure compatibility
      const buffer = new ArrayBuffer(bytes.byteLength);
      new Uint8Array(buffer).set(bytes);
      return parseMeshBytes(buffer);
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  get featureList(): FeatureEntry[] {
    try {
      return JSON.parse(this.handle.get_features_json());
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  suppressFeature(index: number): void {
    try {
      this.handle.suppress(index);
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  unsuppressFeature(index: number): void {
    try {
      this.handle.unsuppress(index);
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  serialize(): string {
    try {
      return this.handle.serialize();
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  exportSTLBinary(chordTolerance: number = 0.01, angleTolerance: number = 0.5): Uint8Array {
    try {
      const bytes = this.handle.export_stl_binary(chordTolerance, angleTolerance);
      const buffer = new Uint8Array(bytes.byteLength);
      buffer.set(bytes);
      return buffer;
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  exportSTLAscii(options: { precision?: number } = {}, chordTolerance: number = 0.01, angleTolerance: number = 0.5): string {
    try {
      return this.handle.export_stl_ascii(chordTolerance, angleTolerance, JSON.stringify(options));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  exportOBJ(options: { precision?: number } = {}, chordTolerance: number = 0.01, angleTolerance: number = 0.5): string {
    try {
      return this.handle.export_obj(chordTolerance, angleTolerance, JSON.stringify(options));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  export3MF(options: { unit?: string; vertex_colors?: boolean } = {}, chordTolerance: number = 0.01, angleTolerance: number = 0.5): Uint8Array {
    try {
      const bytes = this.handle.export_3mf(chordTolerance, angleTolerance, JSON.stringify(options));
      const buffer = new Uint8Array(bytes.byteLength);
      buffer.set(bytes);
      return buffer;
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  exportGLB(options: { quantize?: boolean } = {}, chordTolerance: number = 0.01, angleTolerance: number = 0.5): Uint8Array {
    try {
      const bytes = this.handle.export_glb(chordTolerance, angleTolerance, JSON.stringify(options));
      const buffer = new Uint8Array(bytes.byteLength);
      buffer.set(bytes);
      return buffer;
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  static deserialize(json: string): KernelClient {
    const client = new KernelClient();
    // Free the default handle, replace with deserialized one
    client.handle.free();
    client.handle = KernelHandle.deserialize(json);
    return client;
  }
}
