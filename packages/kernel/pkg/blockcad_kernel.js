/* @ts-self-types="./blockcad_kernel.d.ts" */

/**
 * WASM entry point for assembly operations.
 */
export class AssemblyHandle {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AssemblyHandle.prototype);
        obj.__wbg_ptr = ptr;
        AssemblyHandleFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AssemblyHandleFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_assemblyhandle_free(ptr, 0);
    }
    /**
     * Activate a configuration by index.
     * @param {number} index
     * @returns {boolean}
     */
    activate_configuration(index) {
        const ret = wasm.assemblyhandle_activate_configuration(this.__wbg_ptr, index);
        return ret !== 0;
    }
    /**
     * Add a component instance. `transform_json` is a JSON array of 16 f64 values (column-major 4x4).
     * @param {string} part_id
     * @param {string} name
     * @param {string} transform_json
     * @returns {string}
     */
    add_component(part_id, name, transform_json) {
        let deferred5_0;
        let deferred5_1;
        try {
            const ptr0 = passStringToWasm0(part_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(transform_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ret = wasm.assemblyhandle_add_component(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
            var ptr4 = ret[0];
            var len4 = ret[1];
            if (ret[3]) {
                ptr4 = 0; len4 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred5_0 = ptr4;
            deferred5_1 = len4;
            return getStringFromWasm0(ptr4, len4);
        } finally {
            wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
        }
    }
    /**
     * Add a configuration. Returns its index.
     * @param {string} name
     * @returns {number}
     */
    add_configuration(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.assemblyhandle_add_configuration(this.__wbg_ptr, ptr0, len0);
        return ret >>> 0;
    }
    /**
     * Add a feature to a part. Returns the feature ID.
     * @param {string} part_id
     * @param {string} kind
     * @param {string} params_json
     * @returns {string}
     */
    add_feature_to_part(part_id, kind, params_json) {
        let deferred5_0;
        let deferred5_1;
        try {
            const ptr0 = passStringToWasm0(part_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(params_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ret = wasm.assemblyhandle_add_feature_to_part(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
            var ptr4 = ret[0];
            var len4 = ret[1];
            if (ret[3]) {
                ptr4 = 0; len4 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred5_0 = ptr4;
            deferred5_1 = len4;
            return getStringFromWasm0(ptr4, len4);
        } finally {
            wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
        }
    }
    /**
     * Add a mate constraint between two components.
     * @param {string} mate_json
     * @returns {string}
     */
    add_mate(mate_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(mate_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.assemblyhandle_add_mate(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Add a new part to the assembly. Returns the part ID.
     * @param {string} name
     * @returns {string}
     */
    add_part(name) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.assemblyhandle_add_part(this.__wbg_ptr, ptr0, len0);
            deferred2_0 = ret[0];
            deferred2_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Add reference geometry from JSON. Returns the ID.
     * @param {string} json
     * @returns {string}
     */
    add_reference_geometry(json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.assemblyhandle_add_reference_geometry(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Clear the section cutting plane.
     */
    clear_section_plane() {
        wasm.assemblyhandle_clear_section_plane(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    component_count() {
        const ret = wasm.assemblyhandle_component_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Copy selected components to a JSON snapshot.
     * @param {string} ids_json
     * @returns {string}
     */
    copy_components(ids_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(ids_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.assemblyhandle_copy_components(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Load from assembly JSON.
     * @param {string} json
     * @returns {AssemblyHandle}
     */
    static deserialize(json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.assemblyhandle_deserialize(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return AssemblyHandle.__wrap(ret[0]);
    }
    /**
     * Export assembly as GLB with per-component node hierarchy.
     * @param {number} chord_tolerance
     * @param {number} angle_tolerance
     * @param {string} options_json
     * @returns {Uint8Array}
     */
    export_glb(chord_tolerance, angle_tolerance, options_json) {
        const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.assemblyhandle_export_glb(this.__wbg_ptr, chord_tolerance, angle_tolerance, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * Export assembly as STEP text.
     * @returns {string}
     */
    export_step() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.assemblyhandle_export_step(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Generate a full assembly report as HTML.
     * @returns {string}
     */
    generate_report_html() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.assemblyhandle_generate_report_html(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Generate a full assembly report as JSON.
     * @returns {string}
     */
    generate_report_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.assemblyhandle_generate_report_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get advanced BOM with properties as JSON.
     * @returns {string}
     */
    get_advanced_bom_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.assemblyhandle_get_advanced_bom_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get the assembly structure as JSON.
     * @returns {string}
     */
    get_assembly_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.assemblyhandle_get_assembly_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Get advanced BOM as CSV string.
     * @returns {string}
     */
    get_bom_csv() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.assemblyhandle_get_bom_csv(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get Bill of Materials as JSON.
     * @returns {string}
     */
    get_bom_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.assemblyhandle_get_bom_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get per-component DOF analysis as JSON.
     * @returns {string}
     */
    get_dof_analysis_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.assemblyhandle_get_dof_analysis_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get mass properties as JSON.
     * @returns {string}
     */
    get_mass_properties_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.assemblyhandle_get_mass_properties_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Ground a component (fix in place).
     * @param {number} index
     */
    ground_component(index) {
        const ret = wasm.assemblyhandle_ground_component(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Hide a component (still evaluates for mates, but not rendered).
     * @param {number} index
     */
    hide_component(index) {
        const ret = wasm.assemblyhandle_hide_component(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * List configurations as JSON array of names.
     * @returns {string}
     */
    list_configurations_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.assemblyhandle_list_configurations_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * List reference geometry as JSON array.
     * @returns {string}
     */
    list_reference_geometry_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.assemblyhandle_list_reference_geometry_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Mark a part as dirty (forces re-evaluation).
     * @param {string} part_id
     */
    mark_part_dirty(part_id) {
        const ptr0 = passStringToWasm0(part_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.assemblyhandle_mark_part_dirty(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Measure distance between two geometry references.
     * JSON: { comp_a, geom_a: { face: N }, comp_b, geom_b: { face: N } }
     * @param {string} json
     * @returns {string}
     */
    measure_distance(json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.assemblyhandle_measure_distance(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    constructor() {
        const ret = wasm.assemblyhandle_new();
        this.__wbg_ptr = ret >>> 0;
        AssemblyHandleFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {number}
     */
    part_count() {
        const ret = wasm.assemblyhandle_part_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Paste components from snapshot with offset. Returns JSON array of new IDs.
     * @param {string} snapshot
     * @param {string} offset_json
     * @returns {string}
     */
    paste_components(snapshot, offset_json) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(snapshot, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(offset_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.assemblyhandle_paste_components(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Remove a component by ID. Cascade-deletes referencing mates.
     * @param {string} comp_id
     * @returns {boolean}
     */
    remove_component(comp_id) {
        const ptr0 = passStringToWasm0(comp_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.assemblyhandle_remove_component(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Replace a component's part reference.
     * @param {string} comp_id
     * @param {string} new_part_id
     */
    replace_component_part(comp_id, new_part_id) {
        const ptr0 = passStringToWasm0(comp_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(new_part_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.assemblyhandle_replace_component_part(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Serialize to assembly JSON format.
     * @returns {string}
     */
    serialize() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.assemblyhandle_serialize(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Set per-instance color override (RGBA 0-1). Pass empty string to clear.
     * @param {number} index
     * @param {string} color_json
     */
    set_component_color(index, color_json) {
        const ptr0 = passStringToWasm0(color_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.assemblyhandle_set_component_color(this.__wbg_ptr, index, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set explosion steps from JSON array.
     * @param {string} json
     */
    set_explosion_steps(json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.assemblyhandle_set_explosion_steps(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set a part property.
     * @param {string} part_id
     * @param {string} key
     * @param {string} value
     */
    set_part_property(part_id, key, value) {
        const ptr0 = passStringToWasm0(part_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.assemblyhandle_set_part_property(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Set a section cutting plane. JSON: { normal: [x,y,z], offset: f64 }
     * @param {string} json
     */
    set_section_plane(json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.assemblyhandle_set_section_plane(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Show a hidden component.
     * @param {number} index
     */
    show_component(index) {
        const ret = wasm.assemblyhandle_show_component(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Suggest a mate type based on face geometry. Returns JSON MateKind.
     * @param {number} face_a
     * @param {number} face_b
     * @returns {string}
     */
    suggest_mate(face_a, face_b) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.assemblyhandle_suggest_mate(this.__wbg_ptr, face_a, face_b);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Suppress a component by index.
     * @param {number} index
     */
    suppress_component(index) {
        const ret = wasm.assemblyhandle_suppress_component(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Evaluate the assembly and tessellate all active components.
     * @param {number} chord_tolerance
     * @param {number} angle_tolerance
     * @returns {Uint8Array}
     */
    tessellate(chord_tolerance, angle_tolerance) {
        const ret = wasm.assemblyhandle_tessellate(this.__wbg_ptr, chord_tolerance, angle_tolerance);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Tessellate with exploded view offsets applied.
     * @param {number} chord_tolerance
     * @param {number} angle_tolerance
     * @returns {Uint8Array}
     */
    tessellate_exploded(chord_tolerance, angle_tolerance) {
        const ret = wasm.assemblyhandle_tessellate_exploded(this.__wbg_ptr, chord_tolerance, angle_tolerance);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Unground a component (allow movement).
     * @param {number} index
     */
    unground_component(index) {
        const ret = wasm.assemblyhandle_unground_component(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Unsuppress a component by index.
     * @param {number} index
     */
    unsuppress_component(index) {
        const ret = wasm.assemblyhandle_unsuppress_component(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Validate that a replacement part has compatible face topology.
     * @param {string} comp_id
     * @param {string} new_part_id
     * @returns {string}
     */
    validate_replacement(comp_id, new_part_id) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(comp_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(new_part_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.assemblyhandle_validate_replacement(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            deferred3_0 = ret[0];
            deferred3_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
}
if (Symbol.dispose) AssemblyHandle.prototype[Symbol.dispose] = AssemblyHandle.prototype.free;

/**
 * The main WASM entry point for the kernel.
 * Delegates to KernelCore for all operations.
 */
export class KernelHandle {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(KernelHandle.prototype);
        obj.__wbg_ptr = ptr;
        KernelHandleFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        KernelHandleFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_kernelhandle_free(ptr, 0);
    }
    /**
     * Add a feature. Returns the feature ID on success.
     * @param {string} kind
     * @param {string} params_json
     * @returns {string}
     */
    add_feature(kind, params_json) {
        let deferred4_0;
        let deferred4_1;
        try {
            const ptr0 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(params_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.kernelhandle_add_feature(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Compute mass properties (volume, surface area, center of mass, inertia tensor).
     * Returns JSON-serialized MassProperties. If density > 0, inertia is scaled.
     * @param {number} chord_tolerance
     * @param {number} angle_tolerance
     * @param {number} density
     * @returns {string}
     */
    compute_mass_properties(chord_tolerance, angle_tolerance, density) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.kernelhandle_compute_mass_properties(this.__wbg_ptr, chord_tolerance, angle_tolerance, density);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    cursor() {
        const ret = wasm.kernelhandle_cursor(this.__wbg_ptr);
        return ret;
    }
    /**
     * Load from a .blockcad JSON document.
     * @param {string} json
     * @returns {KernelHandle}
     */
    static deserialize(json) {
        const ptr0 = passStringToWasm0(json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.kernelhandle_deserialize(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return KernelHandle.__wrap(ret[0]);
    }
    /**
     * Evaluate the feature tree and return cache metrics as JSON.
     * Returns `{"features_evaluated": N, "features_skipped_param_hash": N, "features_skipped_fingerprint": N}`
     * @returns {string}
     */
    evaluate_with_metrics() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.kernelhandle_evaluate_with_metrics(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Export as 3MF bytes (ZIP archive).
     * @param {number} chord_tolerance
     * @param {number} angle_tolerance
     * @param {string} options_json
     * @returns {Uint8Array}
     */
    export_3mf(chord_tolerance, angle_tolerance, options_json) {
        const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.kernelhandle_export_3mf(this.__wbg_ptr, chord_tolerance, angle_tolerance, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * Export as GLB (binary glTF 2.0) bytes.
     * @param {number} chord_tolerance
     * @param {number} angle_tolerance
     * @param {string} options_json
     * @returns {Uint8Array}
     */
    export_glb(chord_tolerance, angle_tolerance, options_json) {
        const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.kernelhandle_export_glb(this.__wbg_ptr, chord_tolerance, angle_tolerance, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * Export as Wavefront OBJ string.
     * @param {number} chord_tolerance
     * @param {number} angle_tolerance
     * @param {string} options_json
     * @returns {string}
     */
    export_obj(chord_tolerance, angle_tolerance, options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.kernelhandle_export_obj(this.__wbg_ptr, chord_tolerance, angle_tolerance, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Export as STEP (ISO 10303-21) string.
     * @param {number} chord_tolerance
     * @param {number} angle_tolerance
     * @param {string} options_json
     * @returns {string}
     */
    export_step(chord_tolerance, angle_tolerance, options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.kernelhandle_export_step(this.__wbg_ptr, chord_tolerance, angle_tolerance, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Export as ASCII STL string.
     * @param {number} chord_tolerance
     * @param {number} angle_tolerance
     * @param {string} options_json
     * @returns {string}
     */
    export_stl_ascii(chord_tolerance, angle_tolerance, options_json) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(options_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.kernelhandle_export_stl_ascii(this.__wbg_ptr, chord_tolerance, angle_tolerance, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Export as binary STL bytes.
     * @param {number} chord_tolerance
     * @param {number} angle_tolerance
     * @returns {Uint8Array}
     */
    export_stl_binary(chord_tolerance, angle_tolerance) {
        const ret = wasm.kernelhandle_export_stl_binary(this.__wbg_ptr, chord_tolerance, angle_tolerance);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {number}
     */
    feature_count() {
        const ret = wasm.kernelhandle_feature_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get the feature list as JSON.
     * @returns {string}
     */
    get_features_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.kernelhandle_get_features_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Check if a feature kind is available in this build.
     * @param {string} kind
     * @returns {boolean}
     */
    is_feature_available(kind) {
        const ptr0 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.kernelhandle_is_feature_available(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * Move a feature from one index to another.
     * @param {number} from
     * @param {number} to
     */
    move_feature(from, to) {
        const ret = wasm.kernelhandle_move_feature(this.__wbg_ptr, from, to);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    constructor() {
        const ret = wasm.kernelhandle_new();
        this.__wbg_ptr = ret >>> 0;
        KernelHandleFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Remove a feature by index.
     * @param {number} index
     */
    remove_feature(index) {
        const ret = wasm.kernelhandle_remove_feature(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Rename a feature by index.
     * @param {number} index
     * @param {string} name
     */
    rename_feature(index, name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.kernelhandle_rename_feature(this.__wbg_ptr, index, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Roll forward to include all features.
     */
    roll_forward() {
        wasm.kernelhandle_roll_forward(this.__wbg_ptr);
    }
    /**
     * Roll back to just before the feature at `index`.
     * @param {number} index
     */
    rollback_to(index) {
        const ret = wasm.kernelhandle_rollback_to(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Serialize to pretty-printed JSON (.blockcad format).
     * @returns {string}
     */
    serialize() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.kernelhandle_serialize(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Suppress a feature by index.
     * @param {number} index
     */
    suppress(index) {
        const ret = wasm.kernelhandle_suppress(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Tessellate the current model state into a byte buffer.
     * @param {number} chord_tolerance
     * @param {number} angle_tolerance
     * @returns {Uint8Array}
     */
    tessellate(chord_tolerance, angle_tolerance) {
        const ret = wasm.kernelhandle_tessellate(this.__wbg_ptr, chord_tolerance, angle_tolerance);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * Unsuppress a feature by index.
     * @param {number} index
     */
    unsuppress(index) {
        const ret = wasm.kernelhandle_unsuppress(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Update feature params by index.
     * @param {number} index
     * @param {string} params_json
     */
    update_feature_params(index, params_json) {
        const ptr0 = passStringToWasm0(params_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.kernelhandle_update_feature_params(this.__wbg_ptr, index, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}
if (Symbol.dispose) KernelHandle.prototype[Symbol.dispose] = KernelHandle.prototype.free;

/**
 * WASM handle for mesh data, providing typed array access to vertex/index buffers.
 */
export class MeshHandle {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MeshHandleFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_meshhandle_free(ptr, 0);
    }
    /**
     * Get the mesh as a flat byte buffer for zero-copy JS typed array access.
     * @returns {Uint8Array}
     */
    to_bytes() {
        const ret = wasm.meshhandle_to_bytes(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @returns {number}
     */
    triangle_count() {
        const ret = wasm.meshhandle_triangle_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    vertex_count() {
        const ret = wasm.meshhandle_vertex_count(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) MeshHandle.prototype[Symbol.dispose] = MeshHandle.prototype.free;

/**
 * WASM handle for sketch editing operations.
 * Provides real-time constraint solving via the Rust solver.
 */
export class SketchHandle {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SketchHandle.prototype);
        obj.__wbg_ptr = ptr;
        SketchHandleFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SketchHandleFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_sketchhandle_free(ptr, 0);
    }
    /**
     * Add a constraint.
     * JSON: {"kind":"horizontal","entityIndices":[4],"value":null}
     * @param {string} constraint_json
     * @returns {number}
     */
    add_constraint(constraint_json) {
        const ptr0 = passStringToWasm0(constraint_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.sketchhandle_add_constraint(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Add a sketch entity. Returns the entity index.
     * JSON format: {"type":"point","x":0,"y":0} or {"type":"line","startIndex":0,"endIndex":1}
     * @param {string} entity_json
     * @returns {number}
     */
    add_entity(entity_json) {
        const ptr0 = passStringToWasm0(entity_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.sketchhandle_add_entity(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] >>> 0;
    }
    /**
     * Number of constraints in the sketch
     * @returns {number}
     */
    constraint_count() {
        const ret = wasm.sketchhandle_constraint_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get the DOF status of the sketch as JSON.
     * Returns: {"status":"fully_constrained","dof":0} or similar.
     * @returns {string}
     */
    dof_status() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.sketchhandle_dof_status(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Number of entities in the sketch
     * @returns {number}
     */
    entity_count() {
        const ret = wasm.sketchhandle_entity_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Get all sketch entities as JSON array
     * @returns {string}
     */
    get_entities_json() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.sketchhandle_get_entities_json(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    constructor() {
        const ret = wasm.sketchhandle_new();
        this.__wbg_ptr = ret >>> 0;
        SketchHandleFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Create a sketch on a specified plane (JSON: {origin, normal, uAxis, vAxis})
     * @param {string} plane_json
     * @returns {SketchHandle}
     */
    static new_on_plane(plane_json) {
        const ptr0 = passStringToWasm0(plane_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.sketchhandle_new_on_plane(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return SketchHandle.__wrap(ret[0]);
    }
    /**
     * Solve constraints and return solved entity positions as JSON.
     * Returns: {"converged":true,"iterations":5,"entities":[{"type":"point","x":0,"y":0},...]}
     * @returns {string}
     */
    solve() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.sketchhandle_solve(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Update a point entity's position (for dragging). Takes entity index, new x, new y.
     * @param {number} entity_index
     * @param {number} x
     * @param {number} y
     */
    update_point(entity_index, x, y) {
        const ret = wasm.sketchhandle_update_point(this.__wbg_ptr, entity_index, x, y);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}
if (Symbol.dispose) SketchHandle.prototype[Symbol.dispose] = SketchHandle.prototype.free;

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_6ddd609b62940d55: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./blockcad_kernel_bg.js": import0,
    };
}

const AssemblyHandleFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_assemblyhandle_free(ptr >>> 0, 1));
const KernelHandleFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_kernelhandle_free(ptr >>> 0, 1));
const MeshHandleFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_meshhandle_free(ptr >>> 0, 1));
const SketchHandleFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_sketchhandle_free(ptr >>> 0, 1));

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('blockcad_kernel_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
