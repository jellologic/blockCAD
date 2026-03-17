//! Assembly-level features — cuts and holes that span multiple components.
//!
//! Assembly features define tool bodies (extruded profiles, cylinders) that are
//! boolean-subtracted from one or more component BReps after mate solving.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::operations::boolean::csg::csg_subtract;
use crate::topology::builders::rebuild_brep_from_faces;
use crate::topology::BRep;

/// The kind of assembly-level feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssemblyFeatureKind {
    /// An extruded cut defined by a closed profile.
    Cut {
        /// Closed polygon defining the cut profile (in world space).
        profile_points: Vec<Pt3>,
        /// Extrusion direction (unit vector).
        direction: Vec3,
        /// Extrusion depth along `direction`.
        depth: f64,
    },
    /// A cylindrical hole.
    Hole {
        /// Center of the hole on the entry surface (world space).
        position: Pt3,
        /// Drill direction (unit vector, points into the material).
        direction: Vec3,
        /// Hole diameter.
        diameter: f64,
        /// Hole depth along `direction`.
        depth: f64,
    },
}

/// An assembly-level feature that cuts across one or more components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyFeature {
    /// Unique identifier for this feature.
    pub id: String,
    /// What kind of cut/hole this feature represents.
    pub kind: AssemblyFeatureKind,
    /// Component IDs to apply the feature to. Empty means all components.
    pub affected_components: Vec<String>,
}

/// Number of segments used to approximate cylindrical holes.
const HOLE_SEGMENTS: usize = 32;

/// Build a polygonal cylinder BRep to use as a tool body for hole features.
fn build_cylinder_tool(
    position: &Pt3,
    direction: &Vec3,
    diameter: f64,
    depth: f64,
) -> KernelResult<BRep> {
    let radius = diameter / 2.0;
    let dir = direction.normalize();

    // Build a local coordinate frame around the direction vector
    let arbitrary = if dir.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let u = dir.cross(&arbitrary).normalize();
    let v = dir.cross(&u).normalize();

    let n = HOLE_SEGMENTS;
    let mut bottom_ring: Vec<Pt3> = Vec::with_capacity(n);
    let mut top_ring: Vec<Pt3> = Vec::with_capacity(n);

    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let offset = u * (radius * cos_a) + v * (radius * sin_a);
        bottom_ring.push(Pt3::new(
            position.x + offset.x,
            position.y + offset.y,
            position.z + offset.z,
        ));
        top_ring.push(Pt3::new(
            position.x + offset.x + dir.x * depth,
            position.y + offset.y + dir.y * depth,
            position.z + offset.z + dir.z * depth,
        ));
    }

    let mut faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();

    // Bottom cap (normal = -direction)
    let bottom_cap: Vec<Pt3> = bottom_ring.iter().rev().cloned().collect();
    faces.push((bottom_cap, -dir));

    // Top cap (normal = +direction)
    faces.push((top_ring.clone(), dir));

    // Side faces (quads)
    for i in 0..n {
        let j = (i + 1) % n;
        let b0 = bottom_ring[i];
        let b1 = bottom_ring[j];
        let t0 = top_ring[i];
        let t1 = top_ring[j];

        // Outward normal for this side quad
        let mid = Vec3::new(
            (b0.x + b1.x) / 2.0 - position.x,
            (b0.y + b1.y) / 2.0 - position.y,
            (b0.z + b1.z) / 2.0 - position.z,
        );
        // Remove the direction component to get radial outward
        let radial = (mid - dir * mid.dot(&dir)).normalize();

        faces.push((vec![b0, b1, t1, t0], radial));
    }

    rebuild_brep_from_faces(&faces)
}

/// Build an extruded-profile BRep to use as a tool body for cut features.
fn build_extruded_profile_tool(
    profile_points: &[Pt3],
    direction: &Vec3,
    depth: f64,
) -> KernelResult<BRep> {
    if profile_points.len() < 3 {
        return Err(KernelError::InvalidParameter {
            param: "profile_points".into(),
            value: format!("need >= 3 points, got {}", profile_points.len()),
        });
    }

    let dir = direction.normalize();
    let offset = dir * depth;

    let n = profile_points.len();
    let bottom: Vec<Pt3> = profile_points.to_vec();
    let top: Vec<Pt3> = profile_points
        .iter()
        .map(|p| Pt3::new(p.x + offset.x, p.y + offset.y, p.z + offset.z))
        .collect();

    let mut faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();

    // Bottom cap (normal = -direction): reverse winding
    let bottom_cap: Vec<Pt3> = bottom.iter().rev().cloned().collect();
    faces.push((bottom_cap, -dir));

    // Top cap (normal = +direction)
    faces.push((top.clone(), dir));

    // Side faces
    for i in 0..n {
        let j = (i + 1) % n;
        let b0 = bottom[i];
        let b1 = bottom[j];
        let t0 = top[i];
        let t1 = top[j];

        // Outward normal: cross product of edge vectors
        let edge1 = Vec3::new(b1.x - b0.x, b1.y - b0.y, b1.z - b0.z);
        let edge2 = Vec3::new(t0.x - b0.x, t0.y - b0.y, t0.z - b0.z);
        let normal = edge1.cross(&edge2).normalize();

        faces.push((vec![b0, b1, t1, t0], normal));
    }

    rebuild_brep_from_faces(&faces)
}

/// Apply an assembly feature to the given component BReps.
///
/// For each affected component, this creates a tool body and performs a boolean
/// subtraction (`csg_subtract`) against the component's BRep.
///
/// If `feature.affected_components` is empty, the feature is applied to all
/// components in the map.
pub fn apply_assembly_feature(
    component_breps: &mut HashMap<String, BRep>,
    feature: &AssemblyFeature,
) -> KernelResult<()> {
    // Build the tool body
    let tool = match &feature.kind {
        AssemblyFeatureKind::Cut {
            profile_points,
            direction,
            depth,
        } => build_extruded_profile_tool(profile_points, direction, *depth)?,
        AssemblyFeatureKind::Hole {
            position,
            direction,
            diameter,
            depth,
        } => build_cylinder_tool(position, direction, *diameter, *depth)?,
    };

    // Determine which components to affect
    let target_ids: Vec<String> = if feature.affected_components.is_empty() {
        component_breps.keys().cloned().collect()
    } else {
        feature.affected_components.clone()
    };

    for comp_id in &target_ids {
        if let Some(brep) = component_breps.get(comp_id) {
            let result = csg_subtract(brep, &tool)?;
            component_breps.insert(comp_id.clone(), result);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;
    use crate::topology::body::Body;

    /// Helper: build a box BRep offset along X.
    fn offset_box(x_offset: f64, w: f64, h: f64, d: f64) -> BRep {
        use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
        let base = build_box_brep(w, h, d).unwrap();
        if x_offset.abs() < 1e-12 {
            return base;
        }
        let polys = extract_face_polygons(&base).unwrap();
        let moved: Vec<(Vec<Pt3>, Vec3)> = polys
            .into_iter()
            .map(|(pts, n)| {
                (
                    pts.into_iter()
                        .map(|p| Pt3::new(p.x + x_offset, p.y, p.z))
                        .collect(),
                    n,
                )
            })
            .collect();
        rebuild_brep_from_faces(&moved).unwrap()
    }

    /// Helper: build a box BRep offset along Z.
    fn stack_box(z_offset: f64, w: f64, h: f64, d: f64) -> BRep {
        use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
        let base = build_box_brep(w, h, d).unwrap();
        if z_offset.abs() < 1e-12 {
            return base;
        }
        let polys = extract_face_polygons(&base).unwrap();
        let moved: Vec<(Vec<Pt3>, Vec3)> = polys
            .into_iter()
            .map(|(pts, n)| {
                (
                    pts.into_iter()
                        .map(|p| Pt3::new(p.x, p.y, p.z + z_offset))
                        .collect(),
                    n,
                )
            })
            .collect();
        rebuild_brep_from_faces(&moved).unwrap()
    }

    #[test]
    fn assembly_cut_through_two_stacked_boxes() {
        // Two 10x10x10 boxes stacked along Z
        let box_a = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let box_b = stack_box(10.0, 10.0, 10.0, 10.0);

        let mut breps = HashMap::new();
        breps.insert("comp_a".to_string(), box_a);
        breps.insert("comp_b".to_string(), box_b);

        // A rectangular cut profile (4x4) centered at (3,3) going through both in +Z
        let feature = AssemblyFeature {
            id: "cut1".into(),
            kind: AssemblyFeatureKind::Cut {
                profile_points: vec![
                    Pt3::new(3.0, 3.0, -1.0),
                    Pt3::new(7.0, 3.0, -1.0),
                    Pt3::new(7.0, 7.0, -1.0),
                    Pt3::new(3.0, 7.0, -1.0),
                ],
                direction: Vec3::new(0.0, 0.0, 1.0),
                depth: 22.0, // through both boxes
            },
            affected_components: vec![], // all
        };

        let original_faces_a = breps["comp_a"].faces.len();
        let original_faces_b = breps["comp_b"].faces.len();

        apply_assembly_feature(&mut breps, &feature).unwrap();

        // Both boxes should now have more faces due to the cut
        assert!(
            breps["comp_a"].faces.len() > original_faces_a,
            "Box A should have more faces after cut: {} vs {}",
            breps["comp_a"].faces.len(),
            original_faces_a
        );
        assert!(
            breps["comp_b"].faces.len() > original_faces_b,
            "Box B should have more faces after cut: {} vs {}",
            breps["comp_b"].faces.len(),
            original_faces_b
        );

        // Both should still be solid bodies
        assert!(matches!(breps["comp_a"].body, Body::Solid(_)));
        assert!(matches!(breps["comp_b"].body, Body::Solid(_)));
    }

    #[test]
    fn assembly_hole_through_three_components() {
        // Three 10x10x5 boxes stacked along Z
        let box_a = build_box_brep(10.0, 10.0, 5.0).unwrap();
        let box_b = stack_box(5.0, 10.0, 10.0, 5.0);
        let box_c = stack_box(10.0, 10.0, 10.0, 5.0);

        let mut breps = HashMap::new();
        breps.insert("comp_a".to_string(), box_a);
        breps.insert("comp_b".to_string(), box_b);
        breps.insert("comp_c".to_string(), box_c);

        // Drill a hole at center (5,5) going through all three in +Z
        let feature = AssemblyFeature {
            id: "hole1".into(),
            kind: AssemblyFeatureKind::Hole {
                position: Pt3::new(5.0, 5.0, -1.0),
                direction: Vec3::new(0.0, 0.0, 1.0),
                diameter: 3.0,
                depth: 17.0, // through all three boxes
            },
            affected_components: vec![], // all
        };

        apply_assembly_feature(&mut breps, &feature).unwrap();

        // All three components should have more faces than the original 6
        for (id, brep) in &breps {
            assert!(
                brep.faces.len() > 6,
                "Component {} should have more than 6 faces after hole, got {}",
                id,
                brep.faces.len()
            );
            assert!(
                matches!(brep.body, Body::Solid(_)),
                "Component {} should still be solid",
                id
            );
        }
    }

    #[test]
    fn feature_only_affects_specified_components() {
        let box_a = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let box_b = stack_box(10.0, 10.0, 10.0, 10.0);
        let box_c = stack_box(20.0, 10.0, 10.0, 10.0);

        let mut breps = HashMap::new();
        breps.insert("comp_a".to_string(), box_a);
        breps.insert("comp_b".to_string(), box_b);
        breps.insert("comp_c".to_string(), box_c);

        let original_c_faces = breps["comp_c"].faces.len();

        // Cut that only targets comp_a and comp_b
        let feature = AssemblyFeature {
            id: "cut_selective".into(),
            kind: AssemblyFeatureKind::Cut {
                profile_points: vec![
                    Pt3::new(3.0, 3.0, -1.0),
                    Pt3::new(7.0, 3.0, -1.0),
                    Pt3::new(7.0, 7.0, -1.0),
                    Pt3::new(3.0, 7.0, -1.0),
                ],
                direction: Vec3::new(0.0, 0.0, 1.0),
                depth: 22.0,
            },
            affected_components: vec!["comp_a".into(), "comp_b".into()],
        };

        apply_assembly_feature(&mut breps, &feature).unwrap();

        // comp_a and comp_b should be modified
        assert!(breps["comp_a"].faces.len() > 6, "comp_a should be cut");
        assert!(breps["comp_b"].faces.len() > 6, "comp_b should be cut");

        // comp_c should be unchanged
        assert_eq!(
            breps["comp_c"].faces.len(),
            original_c_faces,
            "comp_c should NOT be affected"
        );
    }
}
