/* tslint:disable */
/* eslint-disable */

/**
 * WASM entry point for assembly operations.
 */
export class AssemblyHandle {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Activate a configuration by index.
     */
    activate_configuration(index: number): boolean;
    /**
     * Add a component instance. `transform_json` is a JSON array of 16 f64 values (column-major 4x4).
     */
    add_component(part_id: string, name: string, transform_json: string): string;
    /**
     * Add a configuration. Returns its index.
     */
    add_configuration(name: string): number;
    /**
     * Add a feature to a part. Returns the feature ID.
     */
    add_feature_to_part(part_id: string, kind: string, params_json: string): string;
    /**
     * Add a mate constraint between two components.
     */
    add_mate(mate_json: string): string;
    /**
     * Add a new part to the assembly. Returns the part ID.
     */
    add_part(name: string): string;
    /**
     * Add reference geometry from JSON. Returns the ID.
     */
    add_reference_geometry(json: string): string;
    /**
     * Clear the section cutting plane.
     */
    clear_section_plane(): void;
    component_count(): number;
    /**
     * Copy selected components to a JSON snapshot.
     */
    copy_components(ids_json: string): string;
    /**
     * Load from assembly JSON.
     */
    static deserialize(json: string): AssemblyHandle;
    /**
     * Export assembly as GLB with per-component node hierarchy.
     */
    export_glb(chord_tolerance: number, angle_tolerance: number, options_json: string): Uint8Array;
    /**
     * Export assembly as STEP text.
     */
    export_step(): string;
    /**
     * Generate a full assembly report as HTML.
     */
    generate_report_html(): string;
    /**
     * Generate a full assembly report as JSON.
     */
    generate_report_json(): string;
    /**
     * Get advanced BOM with properties as JSON.
     */
    get_advanced_bom_json(): string;
    /**
     * Get the assembly structure as JSON.
     */
    get_assembly_json(): string;
    /**
     * Get advanced BOM as CSV string.
     */
    get_bom_csv(): string;
    /**
     * Get Bill of Materials as JSON.
     */
    get_bom_json(): string;
    /**
     * Get per-component DOF analysis as JSON.
     */
    get_dof_analysis_json(): string;
    /**
     * Get mass properties as JSON.
     */
    get_mass_properties_json(): string;
    /**
     * Ground a component (fix in place).
     */
    ground_component(index: number): void;
    /**
     * Hide a component (still evaluates for mates, but not rendered).
     */
    hide_component(index: number): void;
    /**
     * List configurations as JSON array of names.
     */
    list_configurations_json(): string;
    /**
     * List reference geometry as JSON array.
     */
    list_reference_geometry_json(): string;
    /**
     * Mark a part as dirty (forces re-evaluation).
     */
    mark_part_dirty(part_id: string): void;
    /**
     * Measure distance between two geometry references.
     * JSON: { comp_a, geom_a: { face: N }, comp_b, geom_b: { face: N } }
     */
    measure_distance(json: string): string;
    constructor();
    part_count(): number;
    /**
     * Paste components from snapshot with offset. Returns JSON array of new IDs.
     */
    paste_components(snapshot: string, offset_json: string): string;
    /**
     * Remove a component by ID. Cascade-deletes referencing mates.
     */
    remove_component(comp_id: string): boolean;
    /**
     * Replace a component's part reference.
     */
    replace_component_part(comp_id: string, new_part_id: string): void;
    /**
     * Serialize to assembly JSON format.
     */
    serialize(): string;
    /**
     * Set per-instance color override (RGBA 0-1). Pass empty string to clear.
     */
    set_component_color(index: number, color_json: string): void;
    /**
     * Set explosion steps from JSON array.
     */
    set_explosion_steps(json: string): void;
    /**
     * Set a part property.
     */
    set_part_property(part_id: string, key: string, value: string): void;
    /**
     * Set a section cutting plane. JSON: { normal: [x,y,z], offset: f64 }
     */
    set_section_plane(json: string): void;
    /**
     * Show a hidden component.
     */
    show_component(index: number): void;
    /**
     * Suggest a mate type based on face geometry. Returns JSON MateKind.
     */
    suggest_mate(face_a: number, face_b: number): string;
    /**
     * Suppress a component by index.
     */
    suppress_component(index: number): void;
    /**
     * Evaluate the assembly and tessellate all active components.
     */
    tessellate(chord_tolerance: number, angle_tolerance: number): Uint8Array;
    /**
     * Tessellate with exploded view offsets applied.
     */
    tessellate_exploded(chord_tolerance: number, angle_tolerance: number): Uint8Array;
    /**
     * Unground a component (allow movement).
     */
    unground_component(index: number): void;
    /**
     * Unsuppress a component by index.
     */
    unsuppress_component(index: number): void;
    /**
     * Validate that a replacement part has compatible face topology.
     */
    validate_replacement(comp_id: string, new_part_id: string): string;
}

/**
 * The main WASM entry point for the kernel.
 * Delegates to KernelCore for all operations.
 */
export class KernelHandle {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add a feature. Returns the feature ID on success.
     */
    add_feature(kind: string, params_json: string): string;
    /**
     * Compute mass properties (volume, surface area, center of mass, inertia tensor).
     * Returns JSON-serialized MassProperties. If density > 0, inertia is scaled.
     */
    compute_mass_properties(chord_tolerance: number, angle_tolerance: number, density: number): string;
    cursor(): number;
    /**
     * Load from a .blockcad JSON document.
     */
    static deserialize(json: string): KernelHandle;
    /**
     * Evaluate the feature tree and return cache metrics as JSON.
     * Returns `{"features_evaluated": N, "features_skipped_param_hash": N, "features_skipped_fingerprint": N}`
     */
    evaluate_with_metrics(): string;
    /**
     * Export as 3MF bytes (ZIP archive).
     */
    export_3mf(chord_tolerance: number, angle_tolerance: number, options_json: string): Uint8Array;
    /**
     * Export as GLB (binary glTF 2.0) bytes.
     */
    export_glb(chord_tolerance: number, angle_tolerance: number, options_json: string): Uint8Array;
    /**
     * Export as Wavefront OBJ string.
     */
    export_obj(chord_tolerance: number, angle_tolerance: number, options_json: string): string;
    /**
     * Export as STEP (ISO 10303-21) string.
     */
    export_step(chord_tolerance: number, angle_tolerance: number, options_json: string): string;
    /**
     * Export as ASCII STL string.
     */
    export_stl_ascii(chord_tolerance: number, angle_tolerance: number, options_json: string): string;
    /**
     * Export as binary STL bytes.
     */
    export_stl_binary(chord_tolerance: number, angle_tolerance: number): Uint8Array;
    feature_count(): number;
    /**
     * Get the feature list as JSON.
     */
    get_features_json(): string;
    /**
     * Check if a feature kind is available in this build.
     */
    is_feature_available(kind: string): boolean;
    /**
     * Move a feature from one index to another.
     */
    move_feature(from: number, to: number): void;
    constructor();
    /**
     * Remove a feature by index.
     */
    remove_feature(index: number): void;
    /**
     * Rename a feature by index.
     */
    rename_feature(index: number, name: string): void;
    /**
     * Roll forward to include all features.
     */
    roll_forward(): void;
    /**
     * Roll back to just before the feature at `index`.
     */
    rollback_to(index: number): void;
    /**
     * Serialize to pretty-printed JSON (.blockcad format).
     */
    serialize(): string;
    /**
     * Suppress a feature by index.
     */
    suppress(index: number): void;
    /**
     * Tessellate the current model state into a byte buffer.
     */
    tessellate(chord_tolerance: number, angle_tolerance: number): Uint8Array;
    /**
     * Unsuppress a feature by index.
     */
    unsuppress(index: number): void;
    /**
     * Update feature params by index.
     */
    update_feature_params(index: number, params_json: string): void;
}

/**
 * WASM handle for mesh data, providing typed array access to vertex/index buffers.
 */
export class MeshHandle {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get the mesh as a flat byte buffer for zero-copy JS typed array access.
     */
    to_bytes(): Uint8Array;
    triangle_count(): number;
    vertex_count(): number;
}

/**
 * WASM handle for sketch editing operations.
 * Provides real-time constraint solving via the Rust solver.
 */
export class SketchHandle {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add a constraint.
     * JSON: {"kind":"horizontal","entityIndices":[4],"value":null}
     */
    add_constraint(constraint_json: string): number;
    /**
     * Add a sketch entity. Returns the entity index.
     * JSON format: {"type":"point","x":0,"y":0} or {"type":"line","startIndex":0,"endIndex":1}
     */
    add_entity(entity_json: string): number;
    /**
     * Number of constraints in the sketch
     */
    constraint_count(): number;
    /**
     * Get the DOF status of the sketch as JSON.
     * Returns: {"status":"fully_constrained","dof":0} or similar.
     */
    dof_status(): string;
    /**
     * Number of entities in the sketch
     */
    entity_count(): number;
    /**
     * Get all sketch entities as JSON array
     */
    get_entities_json(): string;
    constructor();
    /**
     * Create a sketch on a specified plane (JSON: {origin, normal, uAxis, vAxis})
     */
    static new_on_plane(plane_json: string): SketchHandle;
    /**
     * Solve constraints and return solved entity positions as JSON.
     * Returns: {"converged":true,"iterations":5,"entities":[{"type":"point","x":0,"y":0},...]}
     */
    solve(): string;
    /**
     * Update a point entity's position (for dragging). Takes entity index, new x, new y.
     */
    update_point(entity_index: number, x: number, y: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_assemblyhandle_free: (a: number, b: number) => void;
    readonly __wbg_kernelhandle_free: (a: number, b: number) => void;
    readonly __wbg_meshhandle_free: (a: number, b: number) => void;
    readonly __wbg_sketchhandle_free: (a: number, b: number) => void;
    readonly assemblyhandle_activate_configuration: (a: number, b: number) => number;
    readonly assemblyhandle_add_component: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly assemblyhandle_add_configuration: (a: number, b: number, c: number) => number;
    readonly assemblyhandle_add_feature_to_part: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number, number];
    readonly assemblyhandle_add_mate: (a: number, b: number, c: number) => [number, number, number, number];
    readonly assemblyhandle_add_part: (a: number, b: number, c: number) => [number, number];
    readonly assemblyhandle_add_reference_geometry: (a: number, b: number, c: number) => [number, number, number, number];
    readonly assemblyhandle_clear_section_plane: (a: number) => void;
    readonly assemblyhandle_component_count: (a: number) => number;
    readonly assemblyhandle_copy_components: (a: number, b: number, c: number) => [number, number, number, number];
    readonly assemblyhandle_deserialize: (a: number, b: number) => [number, number, number];
    readonly assemblyhandle_export_glb: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly assemblyhandle_export_step: (a: number) => [number, number, number, number];
    readonly assemblyhandle_generate_report_html: (a: number) => [number, number, number, number];
    readonly assemblyhandle_generate_report_json: (a: number) => [number, number, number, number];
    readonly assemblyhandle_get_advanced_bom_json: (a: number) => [number, number];
    readonly assemblyhandle_get_assembly_json: (a: number) => [number, number, number, number];
    readonly assemblyhandle_get_bom_csv: (a: number) => [number, number];
    readonly assemblyhandle_get_bom_json: (a: number) => [number, number];
    readonly assemblyhandle_get_dof_analysis_json: (a: number) => [number, number];
    readonly assemblyhandle_get_mass_properties_json: (a: number) => [number, number, number, number];
    readonly assemblyhandle_ground_component: (a: number, b: number) => [number, number];
    readonly assemblyhandle_hide_component: (a: number, b: number) => [number, number];
    readonly assemblyhandle_list_configurations_json: (a: number) => [number, number];
    readonly assemblyhandle_list_reference_geometry_json: (a: number) => [number, number];
    readonly assemblyhandle_mark_part_dirty: (a: number, b: number, c: number) => void;
    readonly assemblyhandle_measure_distance: (a: number, b: number, c: number) => [number, number, number, number];
    readonly assemblyhandle_new: () => number;
    readonly assemblyhandle_part_count: (a: number) => number;
    readonly assemblyhandle_paste_components: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly assemblyhandle_remove_component: (a: number, b: number, c: number) => number;
    readonly assemblyhandle_replace_component_part: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly assemblyhandle_set_component_color: (a: number, b: number, c: number, d: number) => [number, number];
    readonly assemblyhandle_set_explosion_steps: (a: number, b: number, c: number) => [number, number];
    readonly assemblyhandle_set_part_property: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
    readonly assemblyhandle_set_section_plane: (a: number, b: number, c: number) => [number, number];
    readonly assemblyhandle_show_component: (a: number, b: number) => [number, number];
    readonly assemblyhandle_suggest_mate: (a: number, b: number, c: number) => [number, number, number, number];
    readonly assemblyhandle_suppress_component: (a: number, b: number) => [number, number];
    readonly assemblyhandle_tessellate: (a: number, b: number, c: number) => [number, number, number, number];
    readonly assemblyhandle_tessellate_exploded: (a: number, b: number, c: number) => [number, number, number, number];
    readonly assemblyhandle_unground_component: (a: number, b: number) => [number, number];
    readonly assemblyhandle_unsuppress_component: (a: number, b: number) => [number, number];
    readonly assemblyhandle_validate_replacement: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly kernelhandle_add_feature: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly kernelhandle_compute_mass_properties: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly kernelhandle_cursor: (a: number) => number;
    readonly kernelhandle_deserialize: (a: number, b: number) => [number, number, number];
    readonly kernelhandle_evaluate_with_metrics: (a: number) => [number, number, number, number];
    readonly kernelhandle_export_3mf: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly kernelhandle_export_glb: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly kernelhandle_export_obj: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly kernelhandle_export_step: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly kernelhandle_export_stl_ascii: (a: number, b: number, c: number, d: number, e: number) => [number, number, number, number];
    readonly kernelhandle_export_stl_binary: (a: number, b: number, c: number) => [number, number, number, number];
    readonly kernelhandle_feature_count: (a: number) => number;
    readonly kernelhandle_get_features_json: (a: number) => [number, number, number, number];
    readonly kernelhandle_is_feature_available: (a: number, b: number, c: number) => number;
    readonly kernelhandle_move_feature: (a: number, b: number, c: number) => [number, number];
    readonly kernelhandle_new: () => number;
    readonly kernelhandle_remove_feature: (a: number, b: number) => [number, number];
    readonly kernelhandle_rename_feature: (a: number, b: number, c: number, d: number) => [number, number];
    readonly kernelhandle_roll_forward: (a: number) => void;
    readonly kernelhandle_rollback_to: (a: number, b: number) => [number, number];
    readonly kernelhandle_serialize: (a: number) => [number, number, number, number];
    readonly kernelhandle_suppress: (a: number, b: number) => [number, number];
    readonly kernelhandle_tessellate: (a: number, b: number, c: number) => [number, number, number, number];
    readonly kernelhandle_unsuppress: (a: number, b: number) => [number, number];
    readonly kernelhandle_update_feature_params: (a: number, b: number, c: number, d: number) => [number, number];
    readonly meshhandle_to_bytes: (a: number) => [number, number];
    readonly meshhandle_triangle_count: (a: number) => number;
    readonly meshhandle_vertex_count: (a: number) => number;
    readonly sketchhandle_add_constraint: (a: number, b: number, c: number) => [number, number, number];
    readonly sketchhandle_add_entity: (a: number, b: number, c: number) => [number, number, number];
    readonly sketchhandle_constraint_count: (a: number) => number;
    readonly sketchhandle_dof_status: (a: number) => [number, number, number, number];
    readonly sketchhandle_entity_count: (a: number) => number;
    readonly sketchhandle_get_entities_json: (a: number) => [number, number, number, number];
    readonly sketchhandle_new: () => number;
    readonly sketchhandle_new_on_plane: (a: number, b: number) => [number, number, number];
    readonly sketchhandle_solve: (a: number) => [number, number, number, number];
    readonly sketchhandle_update_point: (a: number, b: number, c: number, d: number) => [number, number];
    readonly assemblyhandle_serialize: (a: number) => [number, number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
