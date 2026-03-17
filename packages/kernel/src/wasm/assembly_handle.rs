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
        self.assembly.add_part(Part {
            id: id.clone(),
            name: name.into(),
            tree: FeatureTree::new(),
            density: 1.0,
        });
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

    /// Add a component instance. `transform_json` is a JSON array of 16 f64 values (column-major 4×4).
    pub fn add_component(
        &mut self,
        part_id: &str,
        name: &str,
        transform_json: &str,
    ) -> Result<String, JsValue> {
        // Verify part exists
        if self.assembly.find_part(part_id).is_none() {
            return Err(JsValue::from_str(&format!("Part '{}' not found", part_id)));
        }

        let id = format!("comp-{}", self.assembly.components.len());
        let mut component = Component::new(id.clone(), part_id.into(), name.into());

        // Parse transform if provided
        if !transform_json.is_empty() && transform_json != "{}" {
            let arr: [f64; 16] = serde_json::from_str(transform_json)
                .map_err(|e| JsValue::from_str(&format!("Invalid transform: {}", e)))?;
            component.transform = arr;
        }

        self.assembly.add_component(component);
        Ok(id)
    }

    /// Add a mate constraint between two components.
    /// `mate_json` is a JSON object: { kind, component_a, component_b, geometry_ref_a, geometry_ref_b }
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
    /// Returns a JSON array of `{id, meshBytes}` objects.
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

        // Tessellate each component's BRep and merge into a single mesh
        // (For Phase 0, we merge all components into one mesh for simplicity.
        //  Phase 1+ will return per-component meshes for selection.)
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

        // Build per-component mesh data
        let mut components: Vec<(String, crate::tessellation::mesh::TriMesh, [f64; 16])> = Vec::new();
        for (comp_id, brep) in &result.components {
            let mesh = tessellate_brep(brep, &params)
                .map_err(|e| -> JsValue { e.into() })?;
            // Find the component's transform
            let transform = self.assembly.components.iter()
                .find(|c| c.id == *comp_id)
                .map(|c| c.transform)
                .unwrap_or_else(|| crate::geometry::transform::to_array(&crate::geometry::Mat4::identity()));
            components.push((comp_id.clone(), mesh, transform));
        }

        crate::export::gltf::export_glb_assembly(&components, &options)
            .map_err(|e| -> JsValue { e.into() })
    }

    /// Update an existing mate. `mate_json` is a JSON object with optional fields:
    /// `{ kind?, geometry_ref_a?, geometry_ref_b? }`. Only provided fields are updated.
    pub fn update_mate(&mut self, mate_id: &str, mate_json: &str) -> Result<(), JsValue> {
        #[derive(serde::Deserialize)]
        struct MateUpdate {
            kind: Option<crate::assembly::MateKind>,
            geometry_ref_a: Option<crate::assembly::GeometryRef>,
            geometry_ref_b: Option<crate::assembly::GeometryRef>,
        }
        let update: MateUpdate = serde_json::from_str(mate_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid mate update JSON: {}", e)))?;
        self.assembly
            .update_mate(mate_id, update.kind, update.geometry_ref_a, update.geometry_ref_b)
            .map_err(|e| -> JsValue { e.into() })
    }

    /// Remove a mate by ID.
    pub fn remove_mate(&mut self, mate_id: &str) -> Result<(), JsValue> {
        self.assembly
            .remove_mate(mate_id)
            .map_err(|e| -> JsValue { e.into() })
    }

    /// Get a mate by ID as JSON. Returns null if not found.
    pub fn get_mate(&self, mate_id: &str) -> Result<JsValue, JsValue> {
        match self.assembly.get_mate(mate_id) {
            Some(mate) => {
                let json = serde_json::to_string(mate)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
                Ok(JsValue::from_str(&json))
            }
            None => Ok(JsValue::NULL),
        }
    }

    /// Add an assembly pattern (linear or circular component array).
    /// `pattern_json` is a JSON object matching `AssemblyPattern`.
    /// Returns JSON array of newly created component IDs.
    pub fn add_assembly_pattern(&mut self, pattern_json: &str) -> Result<JsValue, JsValue> {
        let pattern: crate::assembly::pattern::AssemblyPattern =
            serde_json::from_str(pattern_json)
                .map_err(|e| JsValue::from_str(&format!("Invalid pattern JSON: {}", e)))?;
        let new_ids = crate::assembly::pattern::apply_assembly_pattern(
            &mut self.assembly,
            &pattern,
        )
        .map_err(|e| -> JsValue { e.into() })?;
        let json = serde_json::to_string(&new_ids)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
        Ok(JsValue::from_str(&json))
    }

    /// Remove an assembly pattern and all its generated components/mates.
    pub fn remove_assembly_pattern(&mut self, pattern_id: &str) -> Result<(), JsValue> {
        crate::assembly::pattern::remove_assembly_pattern(&mut self.assembly, pattern_id)
            .map_err(|e| -> JsValue { e.into() })
    }

    pub fn part_count(&self) -> usize {
        self.assembly.parts.len()
    }

    pub fn component_count(&self) -> usize {
        self.assembly.components.len()
    }
}
