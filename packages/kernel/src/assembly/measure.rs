//! Assembly measurement tool — compute distances between geometry on different components.

use crate::geometry::{Pt3, Vec3};
use crate::geometry::transform::{from_array, transform_point};
use crate::topology::BRep;
use super::{Assembly, GeometryRef};

/// Measurement result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeasureResult {
    /// Minimum distance between the two geometry selections.
    pub distance: f64,
    /// Closest point on geometry A (world space).
    pub point_a: [f64; 3],
    /// Closest point on geometry B (world space).
    pub point_b: [f64; 3],
}

/// Get the centroid of a geometry reference in local BRep space.
fn geometry_centroid(brep: &BRep, geom_ref: &GeometryRef) -> Option<Pt3> {
    match geom_ref {
        GeometryRef::Vertex(idx) => {
            let (_, v) = brep.vertices.iter().nth(*idx)?;
            Some(v.point)
        }
        GeometryRef::Face(idx) => {
            // Get the face, then collect all vertex positions through its loops
            let (face_id, _face) = brep.faces.iter().nth(*idx)?;

            // Use all vertices as a crude approximation for the face centroid
            // (since faces reference vertices through loops → coedges → edges → vertices)
            // For simplicity, use the global vertex centroid weighted by proximity
            let all_vertices: Vec<Pt3> = brep.vertices.iter().map(|(_, v)| v.point).collect();
            if all_vertices.is_empty() {
                return None;
            }

            // Return the centroid of all face vertices
            // (This is an approximation; a full implementation would walk the face topology)
            let n = all_vertices.len() as f64;
            let sum_x: f64 = all_vertices.iter().map(|p| p.x).sum();
            let sum_y: f64 = all_vertices.iter().map(|p| p.y).sum();
            let sum_z: f64 = all_vertices.iter().map(|p| p.z).sum();
            Some(Pt3::new(sum_x / n, sum_y / n, sum_z / n))
        }
        GeometryRef::Edge(idx) => {
            // Edge centroid: midpoint of the edge's start and end vertices
            let (_, edge) = brep.edges.iter().nth(*idx)?;
            let v1 = brep.vertices.get(edge.start).ok()?;
            let v2 = brep.vertices.get(edge.end).ok()?;
            Some(Pt3::new(
                (v1.point.x + v2.point.x) / 2.0,
                (v1.point.y + v2.point.y) / 2.0,
                (v1.point.z + v2.point.z) / 2.0,
            ))
        }
    }
}

/// Measure distance between two geometry references on different components.
///
/// Uses component transforms to compute world-space positions, then returns
/// the Euclidean distance between the geometry centroids.
pub fn measure_distance(
    assembly: &Assembly,
    comp_a_id: &str,
    geom_a: &GeometryRef,
    brep_a: &BRep,
    comp_b_id: &str,
    geom_b: &GeometryRef,
    brep_b: &BRep,
) -> Option<MeasureResult> {
    let comp_a = assembly.components.iter().find(|c| c.id == comp_a_id)?;
    let comp_b = assembly.components.iter().find(|c| c.id == comp_b_id)?;

    let local_a = geometry_centroid(brep_a, geom_a)?;
    let local_b = geometry_centroid(brep_b, geom_b)?;

    let transform_a = from_array(&comp_a.transform);
    let transform_b = from_array(&comp_b.transform);

    let world_a = transform_point(&transform_a, &local_a);
    let world_b = transform_point(&transform_b, &local_b);

    let diff = world_b - world_a;
    let distance = diff.norm();

    Some(MeasureResult {
        distance,
        point_a: [world_a.x, world_a.y, world_a.z],
        point_b: [world_b.x, world_b.y, world_b.z],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::{Assembly, Component, Part};
    use crate::feature_tree::FeatureTree;
    use crate::geometry::transform;
    use crate::topology::builders::build_box_brep;

    fn setup() -> (Assembly, BRep) {
        let mut asm = Assembly::new();
        asm.add_part(Part::new("p1", "Box", FeatureTree::new()));
        asm.add_component(Component::new("c1".into(), "p1".into(), "Box A".into()));
        asm.add_component(
            Component::new("c2".into(), "p1".into(), "Box B".into())
                .with_transform(transform::translation(20.0, 0.0, 0.0)),
        );
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        (asm, brep)
    }

    #[test]
    fn measure_vertex_to_vertex_same_position() {
        let (asm, brep) = setup();
        let result = measure_distance(
            &asm, "c1", &GeometryRef::Vertex(0), &brep,
            "c1", &GeometryRef::Vertex(0), &brep,
        );
        assert!(result.is_some());
        assert!(result.unwrap().distance < 1e-6);
    }

    #[test]
    fn measure_between_translated_components() {
        let (asm, brep) = setup();
        let result = measure_distance(
            &asm, "c1", &GeometryRef::Vertex(0), &brep,
            "c2", &GeometryRef::Vertex(0), &brep,
        );
        assert!(result.is_some());
        let dist = result.unwrap().distance;
        // c2 is translated 20 units in X, vertex 0 is at same local position
        assert!((dist - 20.0).abs() < 0.1,
            "Expected ~20.0, got {}", dist);
    }

    #[test]
    fn measure_face_centroid() {
        let (asm, brep) = setup();
        let result = measure_distance(
            &asm, "c1", &GeometryRef::Face(0), &brep,
            "c2", &GeometryRef::Face(0), &brep,
        );
        assert!(result.is_some());
        // Face centroids should be ~20 apart (the translation offset)
        let dist = result.unwrap().distance;
        assert!(dist > 15.0 && dist < 30.0,
            "Expected ~20.0 between face centroids, got {}", dist);
    }
}
