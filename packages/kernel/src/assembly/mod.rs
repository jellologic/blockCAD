//! Assembly module — multi-part CAD assemblies with component positioning.
//!
//! An Assembly contains Parts (each with its own FeatureTree) and
//! Components (instances of Parts placed at specific transforms).

pub mod assembly_feature;
pub mod evaluator;
pub mod interference;
pub mod bom;
pub mod mass;
pub mod motion;
pub mod pattern;

use crate::error::{KernelError, KernelResult};
use crate::feature_tree::FeatureTree;
use crate::geometry::Mat4;
use crate::geometry::transform;

pub use assembly_feature::{AssemblyFeature, AssemblyFeatureKind};

/// A part definition containing a parametric feature tree.
#[derive(Debug)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub tree: FeatureTree,
    /// Material density (kg/m³ or arbitrary units). Default 1.0.
    pub density: f64,
}

/// A component instance — a Part placed at a specific position/orientation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Component {
    pub id: String,
    /// ID of the Part this component references.
    pub part_id: String,
    pub name: String,
    /// 4×4 homogeneous transform matrix (column-major).
    pub transform: [f64; 16],
    pub suppressed: bool,
    /// Hidden components are still evaluated (for mates) but not rendered.
    #[serde(default)]
    pub hidden: bool,
    /// Grounded components are fixed in place (0 DOF). Any component can be grounded.
    #[serde(default)]
    pub grounded: bool,
    /// Per-instance RGBA color override (0.0-1.0). None = use part default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_override: Option<[f32; 4]>,
}

impl Component {
    pub fn new(id: String, part_id: String, name: String) -> Self {
        Self {
            id,
            part_id,
            name,
            transform: transform::to_array(&Mat4::identity()),
            suppressed: false,
            hidden: false,
            grounded: false,
            color_override: None,
        }
    }

    pub fn with_transform(mut self, matrix: Mat4) -> Self {
        self.transform = transform::to_array(&matrix);
        self
    }

    pub fn with_grounded(mut self, grounded: bool) -> Self {
        self.grounded = grounded;
        self
    }

    pub fn transform_matrix(&self) -> Mat4 {
        transform::from_array(&self.transform)
    }
}

/// A geometry reference on a component's BRep (face, edge, or vertex by index).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryRef {
    Face(usize),
    Edge(usize),
    Vertex(usize),
}

/// A mate constraint between two components.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Mate {
    pub id: String,
    pub kind: MateKind,
    pub component_a: String,
    pub component_b: String,
    pub geometry_ref_a: GeometryRef,
    pub geometry_ref_b: GeometryRef,
    pub suppressed: bool,
}

/// Mate constraint types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MateKind {
    // Standard mates
    Coincident,
    Concentric,
    Distance { value: f64 },
    Angle { value: f64 },
    Parallel,
    Perpendicular,
    Tangent,
    Lock,
    // Mechanical mates
    Hinge,
    Gear { ratio: f64 },
    Screw { pitch: f64 },
    Limit { min: f64, max: f64 },
    // Advanced mates
    Width,
    Symmetric,
    // Mechanical mates (continued)
    RackPinion { pitch_radius: f64 },
    Cam { lift: f64, base_radius: f64 },
    Slot { axis: [f64; 3] },
    UniversalJoint,
}

/// A step in an exploded view — translates a component outward.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExplosionStep {
    pub component_id: String,
    pub direction: [f64; 3],
    pub distance: f64,
}

/// A sub-assembly reference — a nested assembly placed as a component.
#[derive(Debug)]
pub struct SubAssemblyRef {
    /// Component ID used to reference this sub-assembly in the parent.
    pub component_id: String,
    /// Display name.
    pub name: String,
    /// 4x4 homogeneous transform matrix (column-major) for placement in parent.
    pub transform: [f64; 16],
    /// The nested assembly.
    pub assembly: Assembly,
    /// Whether this sub-assembly instance is suppressed.
    pub suppressed: bool,
    /// Whether this sub-assembly instance is hidden.
    pub hidden: bool,
    /// Whether this sub-assembly is grounded (fixed in parent).
    pub grounded: bool,
}

impl SubAssemblyRef {
    pub fn new(component_id: String, name: String, assembly: Assembly) -> Self {
        Self {
            component_id,
            name,
            transform: transform::to_array(&Mat4::identity()),
            assembly,
            suppressed: false,
            hidden: false,
            grounded: false,
        }
    }

    pub fn with_transform(mut self, matrix: Mat4) -> Self {
        self.transform = transform::to_array(&matrix);
        self
    }

    pub fn with_grounded(mut self, grounded: bool) -> Self {
        self.grounded = grounded;
        self
    }

    pub fn transform_matrix(&self) -> Mat4 {
        transform::from_array(&self.transform)
    }
}

/// An assembly containing parts and their positioned instances.
#[derive(Debug)]
pub struct Assembly {
    pub parts: Vec<Part>,
    pub components: Vec<Component>,
    pub sub_assemblies: Vec<SubAssemblyRef>,
    pub mates: Vec<Mate>,
    pub explosion_steps: Vec<ExplosionStep>,
    pub patterns: Vec<pattern::AssemblyPattern>,
    /// Assembly-level features (cuts/holes) applied across components after mate solving.
    pub assembly_features: Vec<AssemblyFeature>,
}

impl Assembly {
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            components: Vec::new(),
            sub_assemblies: Vec::new(),
            mates: Vec::new(),
            explosion_steps: Vec::new(),
            patterns: Vec::new(),
            assembly_features: Vec::new(),
        }
    }

    /// Add a part definition. Returns its index.
    pub fn add_part(&mut self, part: Part) -> usize {
        let idx = self.parts.len();
        self.parts.push(part);
        idx
    }

    /// Add a component instance. Returns its index.
    pub fn add_component(&mut self, component: Component) -> usize {
        let idx = self.components.len();
        self.components.push(component);
        idx
    }

    /// Add a sub-assembly reference. Returns its index.
    pub fn add_sub_assembly(&mut self, sub: SubAssemblyRef) -> usize {
        let idx = self.sub_assemblies.len();
        self.sub_assemblies.push(sub);
        idx
    }

    /// Find a part by ID.
    pub fn find_part(&self, part_id: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.id == part_id)
    }

    /// Find a part by ID (mutable).
    pub fn find_part_mut(&mut self, part_id: &str) -> Option<&mut Part> {
        self.parts.iter_mut().find(|p| p.id == part_id)
    }

    /// Get active (non-suppressed) components.
    pub fn active_components(&self) -> Vec<&Component> {
        self.components.iter().filter(|c| !c.suppressed).collect()
    }

    /// Replace a component's part reference. Keeps transform and mates.
    pub fn replace_component_part(&mut self, comp_id: &str, new_part_id: &str) -> bool {
        if let Some(comp) = self.components.iter_mut().find(|c| c.id == comp_id) {
            comp.part_id = new_part_id.to_string();
            true
        } else {
            false
        }
    }

    /// Get a mate by ID (for editing UI).
    pub fn get_mate(&self, mate_id: &str) -> Option<&Mate> {
        self.mates.iter().find(|m| m.id == mate_id)
    }

    /// Update an existing mate's kind and/or geometry references.
    /// Only fields provided as `Some` are updated; `None` fields are left unchanged.
    pub fn update_mate(
        &mut self,
        mate_id: &str,
        kind: Option<MateKind>,
        geometry_ref_a: Option<GeometryRef>,
        geometry_ref_b: Option<GeometryRef>,
    ) -> KernelResult<()> {
        let mate = self
            .mates
            .iter_mut()
            .find(|m| m.id == mate_id)
            .ok_or_else(|| KernelError::NotFound(format!("Mate '{}' not found", mate_id)))?;
        if let Some(k) = kind {
            mate.kind = k;
        }
        if let Some(ref_a) = geometry_ref_a {
            mate.geometry_ref_a = ref_a;
        }
        if let Some(ref_b) = geometry_ref_b {
            mate.geometry_ref_b = ref_b;
        }
        Ok(())
    }

    /// Remove a mate by ID.
    pub fn remove_mate(&mut self, mate_id: &str) -> KernelResult<()> {
        let idx = self
            .mates
            .iter()
            .position(|m| m.id == mate_id)
            .ok_or_else(|| KernelError::NotFound(format!("Mate '{}' not found", mate_id)))?;
        self.mates.remove(idx);
        Ok(())
    }
}

impl Default for Assembly {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_mate(id: &str) -> Mate {
        Mate {
            id: id.into(),
            kind: MateKind::Coincident,
            component_a: "comp-a".into(),
            component_b: "comp-b".into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(1),
            suppressed: false,
        }
    }

    #[test]
    fn get_mate_returns_existing() {
        let mut asm = Assembly::new();
        asm.mates.push(make_test_mate("mate-1"));
        let mate = asm.get_mate("mate-1");
        assert!(mate.is_some());
        assert_eq!(mate.unwrap().id, "mate-1");
    }

    #[test]
    fn get_mate_returns_none_for_missing() {
        let asm = Assembly::new();
        assert!(asm.get_mate("no-such-mate").is_none());
    }

    #[test]
    fn update_mate_kind() {
        let mut asm = Assembly::new();
        asm.mates.push(make_test_mate("mate-1"));

        asm.update_mate("mate-1", Some(MateKind::Distance { value: 5.0 }), None, None)
            .unwrap();

        let mate = asm.get_mate("mate-1").unwrap();
        assert!(matches!(mate.kind, MateKind::Distance { value } if (value - 5.0).abs() < 1e-12));
    }

    #[test]
    fn update_mate_geometry_refs() {
        let mut asm = Assembly::new();
        asm.mates.push(make_test_mate("mate-1"));

        asm.update_mate(
            "mate-1",
            None,
            Some(GeometryRef::Edge(3)),
            Some(GeometryRef::Vertex(7)),
        )
        .unwrap();

        let mate = asm.get_mate("mate-1").unwrap();
        assert!(matches!(mate.geometry_ref_a, GeometryRef::Edge(3)));
        assert!(matches!(mate.geometry_ref_b, GeometryRef::Vertex(7)));
        // kind should be unchanged
        assert!(matches!(mate.kind, MateKind::Coincident));
    }

    #[test]
    fn update_nonexistent_mate_returns_error() {
        let mut asm = Assembly::new();
        let result = asm.update_mate("no-such-mate", Some(MateKind::Parallel), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn remove_mate_success() {
        let mut asm = Assembly::new();
        asm.mates.push(make_test_mate("mate-1"));
        asm.mates.push(make_test_mate("mate-2"));

        asm.remove_mate("mate-1").unwrap();
        assert_eq!(asm.mates.len(), 1);
        assert!(asm.get_mate("mate-1").is_none());
        assert!(asm.get_mate("mate-2").is_some());
    }

    #[test]
    fn remove_nonexistent_mate_returns_error() {
        let mut asm = Assembly::new();
        let result = asm.remove_mate("no-such-mate");
        assert!(result.is_err());
    }
}
