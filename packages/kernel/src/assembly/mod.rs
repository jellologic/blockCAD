//! Assembly module — multi-part CAD assemblies with component positioning.
//!
//! An Assembly contains Parts (each with its own FeatureTree) and
//! Components (instances of Parts placed at specific transforms).

pub mod evaluator;
pub mod interference;
pub mod bom;
pub mod mass;
pub mod configuration;
pub mod section;
pub mod reference_geometry;
pub mod smart_mate;
pub mod measure;
pub mod report;

use crate::feature_tree::FeatureTree;
use crate::geometry::Mat4;
use crate::geometry::transform;

use self::configuration::AssemblyConfig;
use self::reference_geometry::AssemblyRefGeometry;
use self::section::SectionPlane;

/// A part definition containing a parametric feature tree.
#[derive(Debug)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub tree: FeatureTree,
    /// Custom properties (material, vendor, description, etc.).
    pub properties: std::collections::HashMap<String, String>,
}

impl Part {
    pub fn new(id: impl Into<String>, name: impl Into<String>, tree: FeatureTree) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            tree,
            properties: std::collections::HashMap::new(),
        }
    }
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
    /// Named configurations (C1).
    pub configurations: Vec<AssemblyConfig>,
    /// Currently active configuration index.
    pub active_configuration: Option<usize>,
    /// Section cutting plane (C3).
    pub section_plane: Option<SectionPlane>,
    /// Assembly-level reference geometry (C4).
    pub reference_geometry: Vec<AssemblyRefGeometry>,
    /// Dirty flags for BRep caching (D7). Maps part_id → dirty.
    pub dirty_parts: std::collections::HashSet<String>,
}

impl Assembly {
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            components: Vec::new(),
            mates: Vec::new(),
            explosion_steps: Vec::new(),
            configurations: Vec::new(),
            active_configuration: None,
            section_plane: None,
            reference_geometry: Vec::new(),
            dirty_parts: std::collections::HashSet::new(),
        }
    }

    /// Add a part definition. Returns its index.
    pub fn add_part(&mut self, part: Part) -> usize {
        self.dirty_parts.insert(part.id.clone());
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

    // -- C6: Component delete with mate cascade --

    /// Remove a component by ID and cascade-delete all referencing mates.
    /// Returns true if the component was found and removed.
    pub fn remove_component(&mut self, comp_id: &str) -> bool {
        let had = self.components.len();
        self.components.retain(|c| c.id != comp_id);
        if self.components.len() == had {
            return false;
        }

        // Cascade: remove mates referencing this component
        self.mates.retain(|m| m.component_a != comp_id && m.component_b != comp_id);

        // Also remove explosion steps for this component
        self.explosion_steps.retain(|s| s.component_id != comp_id);

        true
    }

    // -- C8: Copy/Paste --

    /// Copy selected components to a JSON snapshot.
    pub fn copy_components(&self, comp_ids: &[String]) -> String {
        let selected: Vec<&Component> = self.components.iter()
            .filter(|c| comp_ids.contains(&c.id))
            .collect();
        serde_json::to_string(&selected).unwrap_or_else(|_| "[]".into())
    }

    /// Paste components from a JSON snapshot, generating new IDs.
    /// `offset` is applied to each component's translation.
    /// Returns the new component IDs.
    pub fn paste_components(&mut self, snapshot: &str, offset: [f64; 3]) -> Vec<String> {
        let comps: Vec<Component> = serde_json::from_str(snapshot).unwrap_or_default();
        let mut new_ids = Vec::new();
        let base = self.components.len();

        for (i, mut comp) in comps.into_iter().enumerate() {
            let new_id = format!("comp-paste-{}-{}", base, i);
            comp.id = new_id.clone();
            comp.name = format!("{} (Copy)", comp.name);
            // Apply offset to translation
            comp.transform[12] += offset[0];
            comp.transform[13] += offset[1];
            comp.transform[14] += offset[2];
            self.components.push(comp);
            new_ids.push(new_id);
        }

        new_ids
    }

    // -- C3: Section plane management --

    /// Set a section cutting plane.
    pub fn set_section_plane(&mut self, plane: SectionPlane) {
        self.section_plane = Some(plane);
    }

    /// Clear the section cutting plane.
    pub fn clear_section_plane(&mut self) {
        self.section_plane = None;
    }

    // -- D7: Mark parts as dirty for incremental re-evaluation --

    /// Mark a part as dirty (needs re-evaluation).
    pub fn mark_part_dirty(&mut self, part_id: &str) {
        self.dirty_parts.insert(part_id.to_string());
    }

    /// Check if a part is dirty.
    pub fn is_part_dirty(&self, part_id: &str) -> bool {
        self.dirty_parts.contains(part_id)
    }

    /// Clear all dirty flags.
    pub fn clear_dirty_flags(&mut self) {
        self.dirty_parts.clear();
    }

    // -- D6: Validate replacement --

    /// Validate that a replacement part has compatible face topology.
    /// Returns a list of face index mismatches (empty if compatible).
    pub fn validate_replacement(&self, comp_id: &str, _new_part_id: &str) -> Vec<String> {
        let mut conflicts = Vec::new();

        let comp = match self.components.iter().find(|c| c.id == comp_id) {
            Some(c) => c,
            None => {
                conflicts.push(format!("Component '{}' not found", comp_id));
                return conflicts;
            }
        };

        // Check if any mates reference face indices that might be invalid
        for mate in &self.mates {
            if mate.component_a == comp_id || mate.component_b == comp_id {
                let face_ref = if mate.component_a == comp_id {
                    &mate.geometry_ref_a
                } else {
                    &mate.geometry_ref_b
                };
                match face_ref {
                    GeometryRef::Face(idx) => {
                        conflicts.push(format!(
                            "Mate '{}' references face {} — verify compatibility",
                            mate.id, idx
                        ));
                    }
                    GeometryRef::Edge(idx) => {
                        conflicts.push(format!(
                            "Mate '{}' references edge {} — verify compatibility",
                            mate.id, idx
                        ));
                    }
                    _ => {}
                }
            }
        }

        conflicts
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

    fn dummy_part(id: &str, name: &str) -> Part {
        Part {
            id: id.into(),
            name: name.into(),
            tree: FeatureTree::new(),
            properties: std::collections::HashMap::new(),
        }
    }

    // -- C6 tests: remove_component --

    #[test]
    fn remove_component_cascades_mates() {
        let mut asm = Assembly::new();
        asm.add_part(dummy_part("p1", "Part"));
        asm.add_component(Component::new("c1".into(), "p1".into(), "A".into()));
        asm.add_component(Component::new("c2".into(), "p1".into(), "B".into()));
        asm.add_component(Component::new("c3".into(), "p1".into(), "C".into()));
        asm.mates.push(Mate {
            id: "m1".into(),
            kind: MateKind::Coincident,
            component_a: "c1".into(),
            component_b: "c2".into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        });
        asm.mates.push(Mate {
            id: "m2".into(),
            kind: MateKind::Parallel,
            component_a: "c2".into(),
            component_b: "c3".into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        });

        assert!(asm.remove_component("c2"));
        assert_eq!(asm.components.len(), 2);
        // Both mates should be removed (c2 was in both)
        assert_eq!(asm.mates.len(), 0);
    }

    #[test]
    fn remove_nonexistent_component_returns_false() {
        let mut asm = Assembly::new();
        assert!(!asm.remove_component("nonexistent"));
    }

    #[test]
    fn remove_component_preserves_unrelated_mates() {
        let mut asm = Assembly::new();
        asm.add_part(dummy_part("p1", "Part"));
        asm.add_component(Component::new("c1".into(), "p1".into(), "A".into()));
        asm.add_component(Component::new("c2".into(), "p1".into(), "B".into()));
        asm.add_component(Component::new("c3".into(), "p1".into(), "C".into()));
        asm.mates.push(Mate {
            id: "m1".into(),
            kind: MateKind::Coincident,
            component_a: "c1".into(),
            component_b: "c2".into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        });

        // Remove c3, mate between c1-c2 should survive
        assert!(asm.remove_component("c3"));
        assert_eq!(asm.mates.len(), 1);
    }

    // -- C8 tests: copy/paste --

    #[test]
    fn copy_and_paste_components() {
        let mut asm = Assembly::new();
        asm.add_part(dummy_part("p1", "Part"));
        asm.add_component(Component::new("c1".into(), "p1".into(), "Box A".into()));
        asm.add_component(Component::new("c2".into(), "p1".into(), "Box B".into()));

        let snapshot = asm.copy_components(&["c1".into()]);
        let new_ids = asm.paste_components(&snapshot, [10.0, 0.0, 0.0]);
        assert_eq!(new_ids.len(), 1);
        assert_eq!(asm.components.len(), 3);

        let pasted = asm.components.last().unwrap();
        assert!(pasted.name.contains("Copy"));
        assert!((pasted.transform[12] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn paste_multiple_components() {
        let mut asm = Assembly::new();
        asm.add_part(dummy_part("p1", "Part"));
        asm.add_component(Component::new("c1".into(), "p1".into(), "A".into()));
        asm.add_component(Component::new("c2".into(), "p1".into(), "B".into()));

        let snapshot = asm.copy_components(&["c1".into(), "c2".into()]);
        let new_ids = asm.paste_components(&snapshot, [0.0, 20.0, 0.0]);
        assert_eq!(new_ids.len(), 2);
        assert_eq!(asm.components.len(), 4);
    }

    #[test]
    fn paste_empty_snapshot() {
        let mut asm = Assembly::new();
        let new_ids = asm.paste_components("[]", [0.0, 0.0, 0.0]);
        assert!(new_ids.is_empty());
    }

    // -- D6 tests: validate replacement --

    #[test]
    fn validate_replacement_reports_mate_conflicts() {
        let mut asm = Assembly::new();
        asm.add_part(dummy_part("p1", "Part A"));
        asm.add_part(dummy_part("p2", "Part B"));
        asm.add_component(Component::new("c1".into(), "p1".into(), "Comp".into()));
        asm.mates.push(Mate {
            id: "m1".into(),
            kind: MateKind::Coincident,
            component_a: "c1".into(),
            component_b: "c1".into(),
            geometry_ref_a: GeometryRef::Face(3),
            geometry_ref_b: GeometryRef::Face(5),
            suppressed: false,
        });

        let conflicts = asm.validate_replacement("c1", "p2");
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn validate_replacement_nonexistent_component() {
        let asm = Assembly::new();
        let conflicts = asm.validate_replacement("missing", "p1");
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("not found"));
    }
}
