//! Assembly module — multi-part CAD assemblies with component positioning.
//!
//! An Assembly contains Parts (each with its own FeatureTree) and
//! Components (instances of Parts placed at specific transforms).

pub mod evaluator;
pub mod interference;
pub mod bom;
pub mod mass;

use crate::feature_tree::FeatureTree;
use crate::geometry::Mat4;
use crate::geometry::transform;

/// A part definition containing a parametric feature tree.
#[derive(Debug)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub tree: FeatureTree,
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
}

/// A step in an exploded view — translates a component outward.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExplosionStep {
    pub component_id: String,
    pub direction: [f64; 3],
    pub distance: f64,
}

/// An assembly containing parts and their positioned instances.
#[derive(Debug)]
pub struct Assembly {
    pub parts: Vec<Part>,
    pub components: Vec<Component>,
    pub mates: Vec<Mate>,
    pub explosion_steps: Vec<ExplosionStep>,
}

impl Assembly {
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            components: Vec::new(),
            mates: Vec::new(),
            explosion_steps: Vec::new(),
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
}

impl Default for Assembly {
    fn default() -> Self {
        Self::new()
    }
}
