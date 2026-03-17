use wasm_bindgen::prelude::*;

use crate::assembly::{Assembly, Component, Part};
use crate::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
use crate::serialization::assembly_io;
use crate::tessellation::{tessellate_brep, TessellationParams};

/// WASM entry point for assembly operations.
#[wasm_bindgen]
pub struct AssemblyHandle {
    assembly: Assembly,
    name: String,
    feature_counter: usize,
}

#[wasm_bindgen]
impl AssemblyHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> AssemblyHandle {
        AssemblyHandle {
            assembly: Assembly::new(),
            name: "Untitled Assembly".into(),
            feature_counter: 0,
        }
    }

    /// Add a new part to the assembly. Returns the part ID.
    pub fn add_part(&mut self, name: &str) -> String {
        let id = format!("part-{}", self.assembly.parts.len());
        self.assembly.add_part(Part::new(id.clone(), name, FeatureTree::new()));
        id
    }

    /// Add a feature to a part. Returns the feature ID.
    pub fn add_feature_to_part(
        &mut self,
        part_id: &str,
        kind: &str,
        params_json: &str,
    ) -> Result<String, JsValue> {
        let part = self.assembly.find_part_mut(part_id).ok_or_else(|| {
            JsValue::from_str(&format!("Part '{}' not found", part_id))
        })?;

        let feature_kind: FeatureKind = serde_json::from_str(&format!("\"{}\"", kind))
            .map_err(|e| JsValue::from_str(&format!("Invalid kind: {}", e)))?;

        let params: FeatureParams = serde_json::from_str(params_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid params: {}", e)))?;

        self.feature_counter += 1;
        let id = format!("{}-{}", kind, self.feature_counter);
        let name = format!("{} {}", feature_kind.display_name(), self.feature_counter);

        let feature = Feature::new(id.clone(), name, feature_kind, params.clone());
        part.tree.push(feature);

        // For sketch features, populate the sketches HashMap
        if feature_kind == FeatureKind::Sketch {
            if let FeatureParams::Sketch(ref sketch) = params {
                let idx = part.tree.len() - 1;
                part.tree.sketches.insert(idx, sketch.clone());
            }
        }

        Ok(id)
    }

    /// Add a component instance. `transform_json` is a JSON array of 16 f64 values (column-major 4x4).
    pub fn add_component(
        &mut self,
        part_id: &str,
        name: &str,
        transform_json: &str,
    ) -> Result<String, JsValue> {
        if self.assembly.find_part(part_id).is_none() {
            return Err(JsValue::from_str(&format!("Part '{}' not found", part_id)));
        }

        let id = format!("comp-{}", self.assembly.components.len());
        let mut component = Component::new(id.clone(), part_id.into(), name.into());

        if !transform_json.is_empty() && transform_json != "{}" {
            let arr: [f64; 16] = serde_json::from_str(transform_json)
                .map_err(|e| JsValue::from_str(&format!("Invalid transform: {}", e)))?;
            component.transform = arr;
        }

        self.assembly.add_component(component);
        Ok(id)
    }

    /// Add a mate constraint between two components.
    pub fn add_mate(&mut self, mate_json: &str) -> Result<String, JsValue> {
        let mate: crate::assembly::Mate = serde_json::from_str(mate_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid mate JSON: {}", e)))?;
        let id = mate.id.clone();
        self.assembly.mates.push(mate);
        Ok(id)
    }

    /// Hide a component (still evaluates for mates, but not rendered).
    pub fn hide_component(&mut self, index: usize) -> Result<(), JsValue> {
        let comp = self.assembly.components.get_mut(index)
            .ok_or_else(|| JsValue::from_str("Component index out of bounds"))?;
        comp.hidden = true;
        Ok(())
    }

    /// Show a hidden component.
    pub fn show_component(&mut self, index: usize) -> Result<(), JsValue> {
        let comp = self.assembly.components.get_mut(index)
            .ok_or_else(|| JsValue::from_str("Component index out of bounds"))?;
        comp.hidden = false;
        Ok(())
    }

    /// Ground a component (fix in place).
    pub fn ground_component(&mut self, index: usize) -> Result<(), JsValue> {
        let comp = self.assembly.components.get_mut(index)
            .ok_or_else(|| JsValue::from_str("Component index out of bounds"))?;
        comp.grounded = true;
        Ok(())
    }

    /// Unground a component (allow movement).
    pub fn unground_component(&mut self, index: usize) -> Result<(), JsValue> {
        let comp = self.assembly.components.get_mut(index)
            .ok_or_else(|| JsValue::from_str("Component index out of bounds"))?;
        comp.grounded = false;
        Ok(())
    }

    /// Replace a component's part reference.
    pub fn replace_component_part(&mut self, comp_id: &str, new_part_id: &str) -> Result<(), JsValue> {
        if !self.assembly.replace_component_part(comp_id, new_part_id) {
            return Err(JsValue::from_str(&format!("Component '{}' not found", comp_id)));
        }
        Ok(())
    }

    /// Set per-instance color override (RGBA 0-1). Pass empty string to clear.
    pub fn set_component_color(&mut self, index: usize, color_json: &str) -> Result<(), JsValue> {
        let comp = self.assembly.components.get_mut(index)
            .ok_or_else(|| JsValue::from_str("Component index out of bounds"))?;
        if color_json.is_empty() || color_json == "null" {
            comp.color_override = None;
        } else {
            let color: [f32; 4] = serde_json::from_str(color_json)
                .map_err(|e| JsValue::from_str(&format!("Invalid color: {}", e)))?;
            comp.color_override = Some(color);
        }
        Ok(())
    }

    /// Get mass properties as JSON.
    pub fn get_mass_properties_json(&mut self) -> Result<String, JsValue> {
        let result = crate::assembly::evaluator::evaluate_assembly(&mut self.assembly)
            .map_err(|e| -> JsValue { e.into() })?;
        let props = crate::assembly::mass::compute_mass_properties(&result.components);
        serde_json::to_string(&props)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Suppress a component by index.
    pub fn suppress_component(&mut self, index: usize) -> Result<(), JsValue> {
        let comp = self.assembly.components.get_mut(index)
            .ok_or_else(|| JsValue::from_str("Component index out of bounds"))?;
        comp.suppressed = true;
        Ok(())
    }

    /// Unsuppress a component by index.
    pub fn unsuppress_component(&mut self, index: usize) -> Result<(), JsValue> {
        let comp = self.assembly.components.get_mut(index)
            .ok_or_else(|| JsValue::from_str("Component index out of bounds"))?;
        comp.suppressed = false;
        Ok(())
    }

    /// Evaluate the assembly and tessellate all active components.
    pub fn tessellate(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
    ) -> Result<Vec<u8>, JsValue> {
        let result = crate::assembly::evaluator::evaluate_assembly(&mut self.assembly)
            .map_err(|e| -> JsValue { e.into() })?;

        let params = TessellationParams {
            chord_tolerance,
            angle_tolerance,
            ..TessellationParams::default()
        };

        let mut merged = crate::tessellation::mesh::TriMesh::new();
        for (_, brep) in &result.components {
            let mesh = tessellate_brep(brep, &params)
                .map_err(|e| -> JsValue { e.into() })?;
            merged.merge(&mesh);
        }

        Ok(merged.to_bytes())
    }

    /// Get the assembly structure as JSON.
    pub fn get_assembly_json(&self) -> Result<String, JsValue> {
        let doc = assembly_io::serialize_assembly(&self.assembly, &self.name)
            .map_err(|e| -> JsValue { e.into() })?;
        doc.to_json_pretty()
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Serialize to assembly JSON format.
    pub fn serialize(&self) -> Result<String, JsValue> {
        self.get_assembly_json()
    }

    /// Load from assembly JSON.
    pub fn deserialize(json: &str) -> Result<AssemblyHandle, JsValue> {
        let doc = crate::serialization::AssemblyDocument::from_json(json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        let assembly = assembly_io::deserialize_assembly(&doc)
            .map_err(|e| -> JsValue { e.into() })?;
        Ok(AssemblyHandle {
            assembly,
            name: doc.metadata.name,
            feature_counter: 0,
        })
    }

    /// Get Bill of Materials as JSON.
    pub fn get_bom_json(&self) -> String {
        let bom = crate::assembly::bom::generate_bom(&self.assembly);
        serde_json::to_string(&bom).unwrap_or_else(|_| "[]".into())
    }

    /// Get advanced BOM with properties as JSON.
    pub fn get_advanced_bom_json(&self) -> String {
        let bom = crate::assembly::bom::generate_advanced_bom(&self.assembly);
        serde_json::to_string(&bom).unwrap_or_else(|_| "[]".into())
    }

    /// Get advanced BOM as CSV string.
    pub fn get_bom_csv(&self) -> String {
        let bom = crate::assembly::bom::generate_advanced_bom(&self.assembly);
        crate::assembly::bom::bom_to_csv(&bom)
    }

    /// Set explosion steps from JSON array.
    pub fn set_explosion_steps(&mut self, json: &str) -> Result<(), JsValue> {
        let steps: Vec<crate::assembly::ExplosionStep> = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Invalid explosion steps: {}", e)))?;
        self.assembly.explosion_steps = steps;
        Ok(())
    }

    /// Tessellate with exploded view offsets applied.
    pub fn tessellate_exploded(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
    ) -> Result<Vec<u8>, JsValue> {
        let result = crate::assembly::evaluator::evaluate_assembly_exploded(&mut self.assembly)
            .map_err(|e| -> JsValue { e.into() })?;

        let params = TessellationParams {
            chord_tolerance,
            angle_tolerance,
            ..TessellationParams::default()
        };

        let mut merged = crate::tessellation::mesh::TriMesh::new();
        for (_, brep) in &result.components {
            let mesh = tessellate_brep(brep, &params)
                .map_err(|e| -> JsValue { e.into() })?;
            merged.merge(&mesh);
        }

        Ok(merged.to_bytes())
    }

    /// Export assembly as GLB with per-component node hierarchy.
    pub fn export_glb(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        options_json: &str,
    ) -> Result<Vec<u8>, JsValue> {
        let options: crate::export::GlbOptions = serde_json::from_str(options_json).unwrap_or_default();

        let result = crate::assembly::evaluator::evaluate_assembly(&mut self.assembly)
            .map_err(|e| -> JsValue { e.into() })?;

        let params = TessellationParams {
            chord_tolerance,
            angle_tolerance,
            ..TessellationParams::default()
        };

        let mut components: Vec<(String, crate::tessellation::mesh::TriMesh, [f64; 16])> = Vec::new();
        for (comp_id, brep) in &result.components {
            let mesh = tessellate_brep(brep, &params)
                .map_err(|e| -> JsValue { e.into() })?;
            let transform = self.assembly.components.iter()
                .find(|c| c.id == *comp_id)
                .map(|c| c.transform)
                .unwrap_or_else(|| crate::geometry::transform::to_array(&crate::geometry::Mat4::identity()));
            components.push((comp_id.clone(), mesh, transform));
        }

        crate::export::gltf::export_glb_assembly(&components, &options)
            .map_err(|e| -> JsValue { e.into() })
    }

    // -- C1: Configurations --

    /// Add a configuration. Returns its index.
    pub fn add_configuration(&mut self, name: &str) -> usize {
        self.assembly.add_configuration(
            crate::assembly::configuration::AssemblyConfig::new(name)
        )
    }

    /// Activate a configuration by index.
    pub fn activate_configuration(&mut self, index: usize) -> bool {
        self.assembly.activate_configuration(index)
    }

    /// List configurations as JSON array of names.
    pub fn list_configurations_json(&self) -> String {
        let names = self.assembly.list_configurations();
        serde_json::to_string(&names).unwrap_or_else(|_| "[]".into())
    }

    // -- C3: Section views --

    /// Set a section cutting plane. JSON: { normal: [x,y,z], offset: f64 }
    pub fn set_section_plane(&mut self, json: &str) -> Result<(), JsValue> {
        let plane: crate::assembly::section::SectionPlane = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Invalid section plane: {}", e)))?;
        self.assembly.set_section_plane(plane);
        Ok(())
    }

    /// Clear the section cutting plane.
    pub fn clear_section_plane(&mut self) {
        self.assembly.clear_section_plane();
    }

    // -- C4: Reference geometry --

    /// Add reference geometry from JSON. Returns the ID.
    pub fn add_reference_geometry(&mut self, json: &str) -> Result<String, JsValue> {
        let geom: crate::assembly::reference_geometry::AssemblyRefGeometry = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Invalid ref geometry: {}", e)))?;
        Ok(self.assembly.add_reference_geometry(geom))
    }

    /// List reference geometry as JSON array.
    pub fn list_reference_geometry_json(&self) -> String {
        serde_json::to_string(self.assembly.list_reference_geometry())
            .unwrap_or_else(|_| "[]".into())
    }

    // -- C5: Smart mates --

    /// Suggest a mate type based on face geometry. Returns JSON MateKind.
    pub fn suggest_mate(&mut self, face_a: usize, face_b: usize) -> Result<String, JsValue> {
        let result = crate::assembly::evaluator::evaluate_assembly(&mut self.assembly)
            .map_err(|e| -> JsValue { e.into() })?;

        if result.components.len() < 2 {
            return Ok("\"coincident\"".into());
        }

        let brep_a = &result.components[0].1;
        let brep_b = &result.components[1].1;
        let suggestion = crate::assembly::smart_mate::suggest_mate(brep_a, face_a, brep_b, face_b);
        serde_json::to_string(&suggestion)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    // -- C6: Remove component --

    /// Remove a component by ID. Cascade-deletes referencing mates.
    pub fn remove_component(&mut self, comp_id: &str) -> bool {
        self.assembly.remove_component(comp_id)
    }

    // -- C7: DOF analysis --

    /// Get per-component DOF analysis as JSON.
    pub fn get_dof_analysis_json(&self) -> String {
        let analysis = crate::solver::dof::analyze_assembly_dof(&self.assembly);
        serde_json::to_string(&analysis).unwrap_or_else(|_| "[]".into())
    }

    // -- C8: Copy/Paste --

    /// Copy selected components to a JSON snapshot.
    pub fn copy_components(&self, ids_json: &str) -> Result<String, JsValue> {
        let ids: Vec<String> = serde_json::from_str(ids_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid IDs: {}", e)))?;
        Ok(self.assembly.copy_components(&ids))
    }

    /// Paste components from snapshot with offset. Returns JSON array of new IDs.
    pub fn paste_components(&mut self, snapshot: &str, offset_json: &str) -> Result<String, JsValue> {
        let offset: [f64; 3] = serde_json::from_str(offset_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid offset: {}", e)))?;
        let new_ids = self.assembly.paste_components(snapshot, offset);
        serde_json::to_string(&new_ids)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    // -- C10: Measure tool --

    /// Measure distance between two geometry references.
    /// JSON: { comp_a, geom_a: { face: N }, comp_b, geom_b: { face: N } }
    pub fn measure_distance(&mut self, json: &str) -> Result<String, JsValue> {
        #[derive(serde::Deserialize)]
        struct MeasureInput {
            comp_a: String,
            geom_a: crate::assembly::GeometryRef,
            comp_b: String,
            geom_b: crate::assembly::GeometryRef,
        }
        let input: MeasureInput = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Invalid measure input: {}", e)))?;

        let result = crate::assembly::evaluator::evaluate_assembly(&mut self.assembly)
            .map_err(|e| -> JsValue { e.into() })?;

        let brep_a = result.components.iter().find(|(id, _)| *id == input.comp_a)
            .map(|(_, b)| b)
            .ok_or_else(|| JsValue::from_str("Component A not found in results"))?;
        let brep_b = result.components.iter().find(|(id, _)| *id == input.comp_b)
            .map(|(_, b)| b)
            .ok_or_else(|| JsValue::from_str("Component B not found in results"))?;

        let measurement = crate::assembly::measure::measure_distance(
            &self.assembly, &input.comp_a, &input.geom_a, brep_a,
            &input.comp_b, &input.geom_b, brep_b,
        ).ok_or_else(|| JsValue::from_str("Measurement failed"))?;

        serde_json::to_string(&measurement)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    // -- D1: STEP export --

    /// Export assembly as STEP text.
    pub fn export_step(&self) -> Result<String, JsValue> {
        let components: Vec<crate::export::step::StepComponent> = self.assembly.components
            .iter()
            .filter(|c| !c.suppressed)
            .map(|c| {
                let part_name = self.assembly.find_part(&c.part_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| c.part_id.clone());
                crate::export::step::StepComponent {
                    id: c.id.clone(),
                    name: c.name.clone(),
                    part_name,
                    transform: c.transform,
                }
            })
            .collect();

        crate::export::step::export_step_assembly(&self.name, &components, &Default::default())
            .map_err(|e| -> JsValue { e.into() })
    }

    // -- D4: Assembly report --

    /// Generate a full assembly report as JSON.
    pub fn generate_report_json(&mut self) -> Result<String, JsValue> {
        let result = crate::assembly::evaluator::evaluate_assembly(&mut self.assembly)
            .map_err(|e| -> JsValue { e.into() })?;
        let report = crate::assembly::report::generate_report(
            &self.assembly, &self.name, &result.components,
        );
        serde_json::to_string(&report)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Generate a full assembly report as HTML.
    pub fn generate_report_html(&mut self) -> Result<String, JsValue> {
        let result = crate::assembly::evaluator::evaluate_assembly(&mut self.assembly)
            .map_err(|e| -> JsValue { e.into() })?;
        let report = crate::assembly::report::generate_report(
            &self.assembly, &self.name, &result.components,
        );
        Ok(crate::assembly::report::report_to_html(&report))
    }

    // -- D6: Validate replacement --

    /// Validate that a replacement part has compatible face topology.
    pub fn validate_replacement(&self, comp_id: &str, new_part_id: &str) -> String {
        let conflicts = self.assembly.validate_replacement(comp_id, new_part_id);
        serde_json::to_string(&conflicts).unwrap_or_else(|_| "[]".into())
    }

    // -- D7: Performance (dirty flags) --

    /// Mark a part as dirty (forces re-evaluation).
    pub fn mark_part_dirty(&mut self, part_id: &str) {
        self.assembly.mark_part_dirty(part_id);
    }

    /// Set a part property.
    pub fn set_part_property(&mut self, part_id: &str, key: &str, value: &str) -> Result<(), JsValue> {
        let part = self.assembly.find_part_mut(part_id).ok_or_else(|| {
            JsValue::from_str(&format!("Part '{}' not found", part_id))
        })?;
        part.properties.insert(key.into(), value.into());
        Ok(())
    }

    pub fn part_count(&self) -> usize {
        self.assembly.parts.len()
    }

    pub fn component_count(&self) -> usize {
        self.assembly.components.len()
    }
}
