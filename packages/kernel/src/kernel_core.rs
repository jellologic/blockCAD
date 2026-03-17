//! Core kernel API — not gated behind WASM feature.
//! Used by the WASM KernelHandle and testable natively.

use crate::error::{KernelError, KernelResult};
use crate::feature_tree::evaluator::evaluate;
use crate::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
use crate::serialization::{feature_tree_io, migrations, schema::KernelDocument};
use crate::tessellation::{tessellate_brep, TessellationParams};

/// The core kernel state — wraps a feature tree and provides operations.
#[derive(Debug)]
pub struct KernelCore {
    pub tree: FeatureTree,
    pub name: String,
    feature_counter: usize,
}

impl KernelCore {
    pub fn new() -> Self {
        Self {
            tree: FeatureTree::new(),
            name: "Untitled".into(),
            feature_counter: 0,
        }
    }

    pub fn feature_count(&self) -> usize {
        self.tree.len()
    }

    pub fn cursor(&self) -> Option<usize> {
        self.tree.cursor()
    }

    /// Add a feature from a kind string and JSON params.
    /// Returns the feature ID.
    pub fn add_feature(&mut self, kind_str: &str, params_json: &str) -> KernelResult<String> {
        let kind: FeatureKind = serde_json::from_str(&format!("\"{}\"", kind_str))
            .map_err(|e| KernelError::InvalidParameter {
                param: "kind".into(),
                value: format!("{}: {}", kind_str, e),
            })?;

        let params: FeatureParams =
            serde_json::from_str(params_json).map_err(|e| KernelError::Serialization(e.to_string()))?;

        self.feature_counter += 1;
        let id = format!("{}-{}", kind_str, self.feature_counter);
        let name = format!("{} {}", kind.display_name(), self.feature_counter);

        let feature = Feature::new(id.clone(), name, kind, params.clone());
        self.tree.push(feature);

        // For sketch features, populate the sketches HashMap from params
        if kind == FeatureKind::Sketch {
            if let FeatureParams::Sketch(ref sketch) = params {
                let idx = self.tree.len() - 1;
                self.tree.sketches.insert(idx, sketch.clone());
            }
        }

        // Evaluate to catch errors early (skip for sketch-only — sketches don't produce geometry alone)
        if kind != FeatureKind::Sketch {
            evaluate(&mut self.tree)?;
        }

        Ok(id)
    }

    /// Build a tessellated mesh from the current model state.
    pub fn build_mesh(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
    ) -> KernelResult<crate::tessellation::mesh::TriMesh> {
        let brep = evaluate(&mut self.tree)?;
        let params = TessellationParams {
            chord_tolerance,
            angle_tolerance,
            ..TessellationParams::default()
        };
        tessellate_brep(&brep, &params)
    }

    /// Tessellate the current model state.
    /// Returns the mesh as a byte buffer.
    pub fn tessellate(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
    ) -> KernelResult<Vec<u8>> {
        let mesh = self.build_mesh(chord_tolerance, angle_tolerance)?;
        Ok(mesh.to_bytes())
    }

    /// Export as binary STL bytes.
    pub fn export_stl_binary(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
    ) -> KernelResult<Vec<u8>> {
        let mesh = self.build_mesh(chord_tolerance, angle_tolerance)?;
        Ok(crate::export::stl::export_stl_binary(&mesh))
    }

    /// Export as ASCII STL string.
    pub fn export_stl_ascii(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        options_json: &str,
    ) -> KernelResult<String> {
        let options: crate::export::StlOptions = serde_json::from_str(options_json).unwrap_or_default();
        let mesh = self.build_mesh(chord_tolerance, angle_tolerance)?;
        Ok(crate::export::stl::export_stl_ascii(&mesh, &self.name, &options))
    }

    /// Export as Wavefront OBJ string.
    pub fn export_obj(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        options_json: &str,
    ) -> KernelResult<String> {
        let options: crate::export::ObjOptions = serde_json::from_str(options_json).unwrap_or_default();
        let mesh = self.build_mesh(chord_tolerance, angle_tolerance)?;
        Ok(crate::export::obj::export_obj(&mesh, &self.name, &options))
    }

    /// Export as 3MF bytes (ZIP archive).
    pub fn export_3mf(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        options_json: &str,
    ) -> KernelResult<Vec<u8>> {
        let options: crate::export::ThreeMfOptions = serde_json::from_str(options_json).unwrap_or_default();
        let mesh = self.build_mesh(chord_tolerance, angle_tolerance)?;
        crate::export::threemf::export_3mf(&mesh, &self.name, &options)
    }

    /// Export as GLB (binary glTF 2.0) bytes.
    pub fn export_glb(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        options_json: &str,
    ) -> KernelResult<Vec<u8>> {
        let options: crate::export::GlbOptions = serde_json::from_str(options_json).unwrap_or_default();
        let mesh = self.build_mesh(chord_tolerance, angle_tolerance)?;
        crate::export::gltf::export_glb(&mesh, &self.name, &options)
    }

    /// Export as STEP (ISO 10303-21) string.
    pub fn export_step(
        &mut self,
        _chord_tolerance: f64,
        _angle_tolerance: f64,
        options_json: &str,
    ) -> KernelResult<String> {
        let options: crate::export::StepExportOptions = serde_json::from_str(options_json).unwrap_or_default();
        let brep = evaluate(&mut self.tree)?;
        crate::export::step::export_step(&brep, &options)
    }

    /// Compute mass properties from the current model state.
    /// If density is provided, inertia values are scaled accordingly.
    pub fn compute_mass_properties(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        density: Option<f64>,
    ) -> KernelResult<crate::tessellation::MassProperties> {
        let mesh = self.build_mesh(chord_tolerance, angle_tolerance)?;
        let props = match density {
            Some(d) => crate::tessellation::compute_mass_properties_with_density(&mesh, d),
            None => crate::tessellation::compute_mass_properties(&mesh),
        };
        Ok(props)
    }

    /// Get the feature list as JSON.
    pub fn get_features_json(&self) -> KernelResult<String> {
        serde_json::to_string(self.tree.features())
            .map_err(|e| KernelError::Serialization(e.to_string()))
    }

    /// Serialize to .blockcad JSON format.
    pub fn serialize(&self) -> KernelResult<String> {
        let doc = feature_tree_io::serialize_tree(&self.tree, &self.name)?;
        doc.to_json_pretty()
            .map_err(|e| KernelError::Serialization(e.to_string()))
    }

    /// Suppress a feature by index.
    pub fn suppress(&mut self, index: usize) -> KernelResult<()> {
        self.tree.suppress(index)
    }

    /// Unsuppress a feature by index.
    pub fn unsuppress(&mut self, index: usize) -> KernelResult<()> {
        self.tree.unsuppress(index)
    }

    /// Deserialize from .blockcad JSON format.
    pub fn deserialize(json: &str) -> KernelResult<Self> {
        let doc = KernelDocument::from_json(json)
            .map_err(|e| KernelError::Serialization(e.to_string()))?;
        let doc = migrations::migrate(doc)?;
        let tree = feature_tree_io::deserialize_tree(&doc)?;
        Ok(Self {
            tree,
            name: doc.metadata.name,
            feature_counter: 0,
        })
    }
}

impl Default for KernelCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;
    use crate::geometry::{Pt2, Vec3};
    use crate::sketch::constraint::{Constraint, ConstraintKind};
    use crate::sketch::entity::SketchEntity;
    use crate::sketch::Sketch;
    use crate::tessellation::mesh::TriMesh;

    fn make_sketch_json() -> String {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p0 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(10.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(10.0, 5.0),
        });
        let p3 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 5.0),
        });
        let bottom = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
        let right = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
        let top = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
        let left = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![bottom]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![top]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![right]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![left]));
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Distance { value: 10.0 },
            vec![p0, p1],
        ));
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Distance { value: 5.0 },
            vec![p1, p2],
        ));

        let params = FeatureParams::Sketch(sketch);
        serde_json::to_string(&params).unwrap()
    }

    fn make_extrude_json(depth: f64) -> String {
        let params = FeatureParams::Extrude(crate::operations::extrude::ExtrudeParams::blind(
            Vec3::new(0.0, 0.0, 1.0),
            depth,
        ));
        serde_json::to_string(&params).unwrap()
    }

    #[test]
    fn test_add_sketch_and_extrude() {
        let mut core = KernelCore::new();

        let sketch_id = core.add_feature("sketch", &make_sketch_json()).unwrap();
        assert!(sketch_id.starts_with("sketch-"));
        assert_eq!(core.feature_count(), 1);

        let extrude_id = core.add_feature("extrude", &make_extrude_json(7.0)).unwrap();
        assert!(extrude_id.starts_with("extrude-"));
        assert_eq!(core.feature_count(), 2);
    }

    #[test]
    fn test_tessellate_returns_valid_bytes() {
        let mut core = KernelCore::new();
        core.add_feature("sketch", &make_sketch_json()).unwrap();
        core.add_feature("extrude", &make_extrude_json(7.0)).unwrap();

        let bytes = core.tessellate(0.01, 0.5).unwrap();

        // Parse the byte buffer to verify it's valid
        let view = bytes.as_slice();
        let vc = u32::from_le_bytes([view[0], view[1], view[2], view[3]]) as usize;
        assert_eq!(vc, 24, "Should have 24 vertices");
    }

    #[test]
    fn test_get_features_json() {
        let mut core = KernelCore::new();
        core.add_feature("sketch", &make_sketch_json()).unwrap();
        core.add_feature("extrude", &make_extrude_json(7.0)).unwrap();

        let json = core.get_features_json().unwrap();
        let features: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(features.len(), 2);
        assert_eq!(features[0]["type"], "sketch");
        assert_eq!(features[1]["type"], "extrude");
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let mut core = KernelCore::new();
        core.add_feature("sketch", &make_sketch_json()).unwrap();
        core.add_feature("extrude", &make_extrude_json(7.0)).unwrap();

        let json = core.serialize().unwrap();
        let mut core2 = KernelCore::deserialize(&json).unwrap();
        assert_eq!(core2.feature_count(), 2);

        // Tessellate the deserialized model
        let bytes = core2.tessellate(0.01, 0.5).unwrap();
        let view = bytes.as_slice();
        let vc = u32::from_le_bytes([view[0], view[1], view[2], view[3]]) as usize;
        assert_eq!(vc, 24);
    }

    #[test]
    fn test_invalid_kind_returns_error() {
        let mut core = KernelCore::new();
        let result = core.add_feature("not_a_feature", "{}");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_stl_binary() {
        let mut core = KernelCore::new();
        core.add_feature("sketch", &make_sketch_json()).unwrap();
        core.add_feature("extrude", &make_extrude_json(7.0)).unwrap();
        let bytes = core.export_stl_binary(0.01, 0.5).unwrap();
        assert!(bytes.len() > 84);
        let tc = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]);
        assert_eq!(tc, 12); // Box: 6 faces × 2 tris = 12
        assert_eq!(bytes.len(), 84 + 50 * tc as usize);
    }

    #[test]
    fn test_export_stl_ascii() {
        let mut core = KernelCore::new();
        core.add_feature("sketch", &make_sketch_json()).unwrap();
        core.add_feature("extrude", &make_extrude_json(7.0)).unwrap();
        let text = core.export_stl_ascii(0.01, 0.5, "{}").unwrap();
        assert!(text.starts_with("solid"));
        assert!(text.contains("endsolid"));
        assert_eq!(text.matches("facet normal").count(), 12);
    }

    #[test]
    fn test_export_obj() {
        let mut core = KernelCore::new();
        core.add_feature("sketch", &make_sketch_json()).unwrap();
        core.add_feature("extrude", &make_extrude_json(7.0)).unwrap();
        let text = core.export_obj(0.01, 0.5, "{}").unwrap();
        assert!(text.contains("v "));
        assert!(text.contains("vn "));
        assert!(text.contains("f "));
    }

    #[test]
    fn test_export_3mf() {
        let mut core = KernelCore::new();
        core.add_feature("sketch", &make_sketch_json()).unwrap();
        core.add_feature("extrude", &make_extrude_json(7.0)).unwrap();
        let bytes = core.export_3mf(0.01, 0.5, "{}").unwrap();
        assert_eq!(bytes[0], b'P');
        assert_eq!(bytes[1], b'K');
    }

    #[test]
    fn test_export_glb() {
        let mut core = KernelCore::new();
        core.add_feature("sketch", &make_sketch_json()).unwrap();
        core.add_feature("extrude", &make_extrude_json(7.0)).unwrap();
        let bytes = core.export_glb(0.01, 0.5, "{}").unwrap();
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(magic, 0x46546C67);
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(version, 2);
    }
}
