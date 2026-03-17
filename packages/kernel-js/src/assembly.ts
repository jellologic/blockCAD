import { AssemblyHandle } from "@blockCAD/kernel-wasm";
import { parseMeshBytes, type MeshData } from "./mesh";
import type { FeatureParams } from "./types";
import { KernelError } from "./errors";
import { transformFeatureParams } from "./kernel";

/** A lightweight sub-assembly grouping (frontend-only). */
export interface SubAssembly {
  name: string;
  /** Component IDs that belong to this sub-assembly. */
  componentIds: string[];
}

/** Per-component mesh data with face IDs for picking. */
export interface PerComponentMesh {
  componentId: string;
  componentIndex: number;
  positions: Float64Array;
  normals: Float64Array;
  indices: Uint32Array;
  faceIds: Uint32Array;
}

/** A single frame of a motion study. */
export interface MotionFrame {
  step: number;
  driverValue: number;
  mesh: MeshData;
}

/**
 * High-level TypeScript client for assembly operations.
 * Wraps the WASM AssemblyHandle.
 */
export class AssemblyClient {
  private handle: AssemblyHandle;
  /** Frontend-only sub-assembly groupings. */
  private _subAssemblies: SubAssembly[] = [];

  constructor() {
    this.handle = new AssemblyHandle();
  }

  /** Create a named sub-assembly group. Returns its index. */
  addSubAssembly(name: string): number {
    const idx = this._subAssemblies.length;
    this._subAssemblies.push({ name, componentIds: [] });
    return idx;
  }

  /** Insert a component and associate it with a sub-assembly. */
  insertComponentInSubAssembly(subIdx: number, partId: string, name: string, transform?: number[]): string {
    const sub = this._subAssemblies[subIdx];
    if (!sub) {
      throw new KernelError("not_found", `Sub-assembly index ${subIdx} out of bounds`);
    }
    const compId = this.addComponent(partId, name, transform);
    sub.componentIds.push(compId);
    return compId;
  }

  /** Get the list of sub-assemblies (read-only snapshot). */
  get subAssemblies(): ReadonlyArray<Readonly<SubAssembly>> {
    return this._subAssemblies;
  }

  /** Add a new part to the assembly. Returns the part ID. */
  addPart(name: string): string {
    return this.handle.add_part(name);
  }

  /** Add a feature to a specific part. Returns the feature ID. */
  addFeatureToPart(partId: string, kind: string, params: FeatureParams): string {
    try {
      const transformed = transformFeatureParams(kind, params);
      return this.handle.add_feature_to_part(partId, kind, JSON.stringify(transformed));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Add a component instance. Returns the component ID. */
  addComponent(partId: string, name: string, transform?: number[]): string {
    try {
      const transformJson = transform ? JSON.stringify(transform) : "";
      return this.handle.add_component(partId, name, transformJson);
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Add a mate constraint between two components. */
  addMate(mate: {
    id: string;
    kind: any;
    component_a: string;
    component_b: string;
    geometry_ref_a: any;
    geometry_ref_b: any;
    suppressed?: boolean;
  }): string {
    try {
      return this.handle.add_mate(JSON.stringify({
        ...mate,
        suppressed: mate.suppressed ?? false,
      }));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Update an existing mate. Only provided fields are changed. */
  updateMate(mateId: string, update: Record<string, unknown>): void {
    try {
      this.handle.update_mate(mateId, JSON.stringify(update));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Remove a mate by ID. */
  removeMate(mateId: string): void {
    try {
      this.handle.remove_mate(mateId);
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Get a mate by ID. Returns null if not found. */
  getMate(mateId: string): unknown | null {
    try {
      const result = this.handle.get_mate(mateId);
      if (result === null || result === undefined) return null;
      return JSON.parse(result as string);
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Hide a component (still evaluates for mates). */
  hideComponent(index: number): void {
    try { this.handle.hide_component(index); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Show a hidden component. */
  showComponent(index: number): void {
    try { this.handle.show_component(index); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Ground a component (fix in place). */
  groundComponent(index: number): void {
    try { this.handle.ground_component(index); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Unground a component. */
  ungroundComponent(index: number): void {
    try { this.handle.unground_component(index); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Replace a component's part reference. */
  replaceComponentPart(compId: string, newPartId: string): void {
    try { this.handle.replace_component_part(compId, newPartId); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Set per-instance color override. Pass null to clear. */
  setComponentColor(index: number, rgba: [number, number, number, number] | null): void {
    try { this.handle.set_component_color(index, rgba ? JSON.stringify(rgba) : ""); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Get mass properties (volume, center of gravity, bounding box). */
  getMassProperties(): { total_volume: number; bbox_min: number[]; bbox_max: number[]; center_of_gravity: number[]; component_count: number } {
    try { return JSON.parse(this.handle.get_mass_properties_json()); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Suppress a component by index. */
  suppressComponent(index: number): void {
    try {
      this.handle.suppress_component(index);
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Unsuppress a component by index. */
  unsuppressComponent(index: number): void {
    try {
      this.handle.unsuppress_component(index);
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Tessellate all active components into a single merged mesh. */
  tessellate(chordTolerance: number = 0.01, angleTolerance: number = 0.5): MeshData {
    try {
      const bytes = this.handle.tessellate(chordTolerance, angleTolerance);
      const buffer = new ArrayBuffer(bytes.byteLength);
      new Uint8Array(buffer).set(bytes);
      return parseMeshBytes(buffer);
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Serialize to assembly JSON format. */
  serialize(): string {
    try {
      return this.handle.serialize();
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Load from assembly JSON. */
  static deserialize(json: string): AssemblyClient {
    const client = new AssemblyClient();
    client.handle.free();
    client.handle = AssemblyHandle.deserialize(json);
    return client;
  }

  /** Get Bill of Materials. */
  getBom(): Array<{ part_id: string; part_name: string; quantity: number }> {
    try {
      return JSON.parse(this.handle.get_bom_json());
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Set explosion steps for exploded view. */
  setExplosionSteps(steps: Array<{ component_id: string; direction: [number, number, number]; distance: number }>): void {
    try {
      this.handle.set_explosion_steps(JSON.stringify(steps));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Tessellate with exploded view offsets applied. */
  tessellateExploded(chordTolerance: number = 0.01, angleTolerance: number = 0.5): MeshData {
    try {
      const bytes = this.handle.tessellate_exploded(chordTolerance, angleTolerance);
      const buffer = new ArrayBuffer(bytes.byteLength);
      new Uint8Array(buffer).set(bytes);
      return parseMeshBytes(buffer);
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Export assembly as GLB with per-component node hierarchy. */
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

  /** Tessellate each component separately, returning per-component mesh data with face IDs for picking. */
  tessellatePerComponent(chordTolerance: number = 0.01, angleTolerance: number = 0.5): PerComponentMesh[] {
    try {
      const jsonStr = this.handle.tessellate_per_component(chordTolerance, angleTolerance) as string;
      const raw: Array<{
        component_id: string;
        component_index: number;
        positions: number[];
        normals: number[];
        indices: number[];
        face_ids: number[];
      }> = JSON.parse(jsonStr);

      return raw.map((entry) => ({
        componentId: entry.component_id,
        componentIndex: entry.component_index,
        positions: new Float64Array(entry.positions),
        normals: new Float64Array(entry.normals),
        indices: new Uint32Array(entry.indices),
        faceIds: new Uint32Array(entry.face_ids),
      }));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Set the transform of a component by index. Transform is a 16-element column-major 4x4 matrix. */
  setComponentTransform(index: number, transform: number[]): void {
    try {
      this.handle.set_component_transform(index, JSON.stringify(transform));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Get the transform of a component by index. Returns a 16-element column-major 4x4 matrix. */
  getComponentTransform(index: number): number[] {
    try {
      return JSON.parse(this.handle.get_component_transform(index));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Run a motion study on the assembly. Returns frames with tessellated meshes. */
  runMotionStudy(params: {
    driverMateId: string;
    startValue: number;
    endValue: number;
    numSteps: number;
  }): MotionFrame[] {
    try {
      const jsonStr = this.handle.run_motion_study(JSON.stringify({
        driver_mate_id: params.driverMateId,
        start_value: params.startValue,
        end_value: params.endValue,
        num_steps: params.numSteps,
      })) as string;
      const raw: Array<{
        step: number;
        driver_value: number;
        component_transforms: Array<{ component_id: string; transform: number[] }>;
      }> = JSON.parse(jsonStr);

      // For each frame, tessellate the assembly to get the mesh
      return raw.map((frame) => ({
        step: frame.step,
        driverValue: frame.driver_value,
        mesh: this.tessellate(),
      }));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Run detailed interference detection. Returns pairs of interfering components with contact points. */
  checkInterference(): {
    pairs: Array<{
      componentA: string;
      componentB: string;
      overlapVolumeEstimate: number;
      contactPoints: Array<[number, number, number]>;
    }>;
  } {
    try {
      const jsonStr = this.handle.check_interference_json() as string;
      const raw = JSON.parse(jsonStr);
      return {
        pairs: raw.pairs.map((p: any) => ({
          componentA: p.component_a,
          componentB: p.component_b,
          overlapVolumeEstimate: p.overlap_volume_estimate,
          contactPoints: p.contact_points,
        })),
      };
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Add an assembly-level feature (cut or hole across components). Returns the feature ID. */
  addAssemblyFeature(feature: {
    id: string;
    kind: any;
    affected_components: string[];
  }): string {
    try {
      return this.handle.add_assembly_feature(JSON.stringify(feature));
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /** Remove an assembly-level feature by ID. */
  removeAssemblyFeature(featureId: string): boolean {
    return this.handle.remove_assembly_feature(featureId);
  }

  /**
   * Create a linear pattern of components.
   * For each source, duplicates it `count-1` times along `direction` with given `spacing`.
   * `sources` maps component names to their partId so we can create proper components.
   * Returns the IDs of all newly created components.
   */
  addLinearPattern(
    sources: Array<{ partId: string; name: string }>,
    direction: [number, number, number],
    spacing: number,
    count: number,
  ): string[] {
    try {
      const len = Math.sqrt(direction[0] ** 2 + direction[1] ** 2 + direction[2] ** 2);
      if (len === 0) throw new Error("Direction vector must be non-zero");
      const dir: [number, number, number] = [direction[0] / len, direction[1] / len, direction[2] / len];

      const newIds: string[] = [];
      for (const src of sources) {
        for (let i = 1; i < count; i++) {
          const offset = spacing * i;
          const transform = [
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            dir[0] * offset, dir[1] * offset, dir[2] * offset, 1,
          ];
          const id = this.handle.add_component(
            src.partId,
            `${src.name} (Linear ${i + 1})`,
            JSON.stringify(transform),
          );
          newIds.push(id);
        }
      }
      return newIds;
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /**
   * Create a circular pattern of components.
   * For each source, duplicates it `count-1` times around an axis defined by
   * `axisOrigin` and `axisDirection`, separated by `angleSpacing` degrees.
   * Returns the IDs of all newly created components.
   */
  addCircularPattern(
    sources: Array<{ partId: string; name: string }>,
    axisOrigin: [number, number, number],
    axisDirection: [number, number, number],
    angleSpacing: number,
    count: number,
  ): string[] {
    try {
      const len = Math.sqrt(axisDirection[0] ** 2 + axisDirection[1] ** 2 + axisDirection[2] ** 2);
      if (len === 0) throw new Error("Axis direction must be non-zero");
      const ax = axisDirection[0] / len;
      const ay = axisDirection[1] / len;
      const az = axisDirection[2] / len;

      const newIds: string[] = [];
      for (const src of sources) {
        for (let i = 1; i < count; i++) {
          const angle = (angleSpacing * i * Math.PI) / 180;
          const c = Math.cos(angle);
          const s = Math.sin(angle);
          const t = 1 - c;

          // Rodrigues' rotation matrix (column-major)
          const r00 = t * ax * ax + c;
          const r01 = t * ax * ay + s * az;
          const r02 = t * ax * az - s * ay;
          const r10 = t * ax * ay - s * az;
          const r11 = t * ay * ay + c;
          const r12 = t * ay * az + s * ax;
          const r20 = t * ax * az + s * ay;
          const r21 = t * ay * az - s * ax;
          const r22 = t * az * az + c;

          // Translation: rotate(-origin) then translate(+origin)
          const ox = axisOrigin[0], oy = axisOrigin[1], oz = axisOrigin[2];
          const tx = ox - (r00 * ox + r10 * oy + r20 * oz);
          const ty = oy - (r01 * ox + r11 * oy + r21 * oz);
          const tz = oz - (r02 * ox + r12 * oy + r22 * oz);

          // Column-major 4x4
          const transform = [
            r00, r01, r02, 0,
            r10, r11, r12, 0,
            r20, r21, r22, 0,
            tx,  ty,  tz,  1,
          ];

          const id = this.handle.add_component(
            src.partId,
            `${src.name} (Circular ${i + 1})`,
            JSON.stringify(transform),
          );
          newIds.push(id);
        }
      }
      return newIds;
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  /**
   * Remove a pattern by suppressing all components that belong to it.
   * The store layer tracks which component indices belong to each pattern.
   */
  removePattern(componentIndices: number[]): void {
    try {
      for (const index of componentIndices) {
        this.handle.suppress_component(index);
      }
    } catch (err) {
      throw KernelError.fromWasm(String(err));
    }
  }

  get partCount(): number {
    return this.handle.part_count();
  }

  get componentCount(): number {
    return this.handle.component_count();
  }
}
