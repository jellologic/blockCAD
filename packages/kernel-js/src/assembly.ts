import { AssemblyHandle } from "@blockCAD/kernel-wasm";
import { parseMeshBytes, type MeshData } from "./mesh";
import type { FeatureParams } from "./types";
import { KernelError } from "./errors";
import { transformFeatureParams } from "./kernel";

/**
 * High-level TypeScript client for assembly operations.
 * Wraps the WASM AssemblyHandle.
 */
export class AssemblyClient {
  private handle: AssemblyHandle;

  constructor() {
    this.handle = new AssemblyHandle();
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
    try { this.handle.suppress_component(index); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Unsuppress a component by index. */
  unsuppressComponent(index: number): void {
    try { this.handle.unsuppress_component(index); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
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
    try { return this.handle.serialize(); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
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
    try { return JSON.parse(this.handle.get_bom_json()); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Get advanced BOM with custom properties as JSON. */
  getAdvancedBom(): any[] {
    try { return JSON.parse(this.handle.get_advanced_bom_json()); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Get BOM as CSV string. */
  getBomCsv(): string {
    try { return this.handle.get_bom_csv(); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Set explosion steps for exploded view. */
  setExplosionSteps(steps: Array<{ component_id: string; direction: [number, number, number]; distance: number }>): void {
    try { this.handle.set_explosion_steps(JSON.stringify(steps)); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
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

  // -- C1: Configurations --

  /** Add a named configuration. Returns its index. */
  addConfiguration(name: string): number {
    return this.handle.add_configuration(name);
  }

  /** Activate a configuration by index. */
  activateConfiguration(index: number): boolean {
    return this.handle.activate_configuration(index);
  }

  /** List configuration names. */
  listConfigurations(): string[] {
    try { return JSON.parse(this.handle.list_configurations_json()); }
    catch { return []; }
  }

  // -- C3: Section views --

  /** Set a section cutting plane. */
  setSectionPlane(json: string): void {
    try { this.handle.set_section_plane(json); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Clear the section cutting plane. */
  clearSectionPlane(): void {
    this.handle.clear_section_plane();
  }

  // -- C4: Reference geometry --

  /** Add assembly-level reference geometry. Returns its ID. */
  addReferenceGeometry(json: string): string {
    try { return this.handle.add_reference_geometry(json); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** List all reference geometry. */
  listReferenceGeometry(): any[] {
    try { return JSON.parse(this.handle.list_reference_geometry_json()); }
    catch { return []; }
  }

  // -- C5: Smart mates --

  /** Suggest a mate type based on face geometry. */
  suggestMate(faceA: number, faceB: number): any {
    try { return JSON.parse(this.handle.suggest_mate(faceA, faceB)); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  // -- C6: Remove component --

  /** Remove a component by ID. Cascade-deletes referencing mates. */
  removeComponent(compId: string): boolean {
    return this.handle.remove_component(compId);
  }

  // -- C7: DOF analysis --

  /** Get per-component DOF analysis as JSON string. */
  getDofAnalysisJson(): string {
    return this.handle.get_dof_analysis_json();
  }

  // -- C8: Copy/Paste --

  /** Copy selected components to a JSON snapshot. */
  copyComponents(idsJson: string): string {
    try { return this.handle.copy_components(idsJson); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Paste components from snapshot with offset. Returns JSON array of new IDs. */
  pasteComponents(snapshot: string, offsetJson: string): string {
    try { return this.handle.paste_components(snapshot, offsetJson); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  // -- C10: Measure tool --

  /** Measure distance between two geometry references. Returns JSON. */
  measureDistance(json: string): string {
    try { return this.handle.measure_distance(json); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  // -- D1: STEP export --

  /** Export assembly as STEP text. */
  exportStep(): string {
    try { return this.handle.export_step(); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  // -- D4: Assembly report --

  /** Generate assembly report as JSON. */
  generateReportJson(): string {
    try { return this.handle.generate_report_json(); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  /** Generate assembly report as HTML. */
  generateReportHtml(): string {
    try { return this.handle.generate_report_html(); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  // -- D6: Validate replacement --

  /** Validate that a replacement part has compatible face topology. */
  validateReplacement(compId: string, newPartId: string): string[] {
    try { return JSON.parse(this.handle.validate_replacement(compId, newPartId)); }
    catch { return []; }
  }

  // -- D7: Performance --

  /** Mark a part as dirty (forces re-evaluation). */
  markPartDirty(partId: string): void {
    this.handle.mark_part_dirty(partId);
  }

  /** Set a custom property on a part. */
  setPartProperty(partId: string, key: string, value: string): void {
    try { this.handle.set_part_property(partId, key, value); }
    catch (err) { throw KernelError.fromWasm(String(err)); }
  }

  get partCount(): number {
    return this.handle.part_count();
  }

  get componentCount(): number {
    return this.handle.component_count();
  }
}
