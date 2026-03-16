//! Mass properties computation from assembly component BReps.

use crate::geometry::{Pt3, Vec3};
use crate::topology::BRep;

/// Mass properties of an assembly.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MassProperties {
    /// Total volume of all components (cubic units).
    pub total_volume: f64,
    /// Bounding box min corner.
    pub bbox_min: [f64; 3],
    /// Bounding box max corner.
    pub bbox_max: [f64; 3],
    /// Center of gravity (volume-weighted centroid).
    pub center_of_gravity: [f64; 3],
    /// Number of components included.
    pub component_count: usize,
}

/// Compute mass properties from positioned component BReps.
///
/// Volume is approximated from bounding box volume (conservative upper bound).
/// Center of gravity is the volume-weighted centroid of component bounding boxes.
pub fn compute_mass_properties(components: &[(String, BRep)]) -> MassProperties {
    if components.is_empty() {
        return MassProperties {
            total_volume: 0.0,
            bbox_min: [0.0; 3],
            bbox_max: [0.0; 3],
            center_of_gravity: [0.0; 3],
            component_count: 0,
        };
    }

    let mut total_volume = 0.0;
    let mut weighted_center = [0.0f64; 3];
    let mut global_min = [f64::INFINITY; 3];
    let mut global_max = [f64::NEG_INFINITY; 3];

    for (_, brep) in components {
        let (comp_min, comp_max) = compute_brep_bbox(brep);

        // Update global bounding box
        for i in 0..3 {
            if comp_min[i] < global_min[i] { global_min[i] = comp_min[i]; }
            if comp_max[i] > global_max[i] { global_max[i] = comp_max[i]; }
        }

        // Component volume (bounding box approximation)
        let vol = (comp_max[0] - comp_min[0]) * (comp_max[1] - comp_min[1]) * (comp_max[2] - comp_min[2]);
        if vol > 0.0 {
            total_volume += vol;
            for i in 0..3 {
                weighted_center[i] += ((comp_min[i] + comp_max[i]) / 2.0) * vol;
            }
        }
    }

    let cog = if total_volume > 0.0 {
        [
            weighted_center[0] / total_volume,
            weighted_center[1] / total_volume,
            weighted_center[2] / total_volume,
        ]
    } else {
        [0.0; 3]
    };

    // Handle case where no valid geometry was found
    if global_min[0].is_infinite() {
        return MassProperties {
            total_volume: 0.0,
            bbox_min: [0.0; 3],
            bbox_max: [0.0; 3],
            center_of_gravity: [0.0; 3],
            component_count: components.len(),
        };
    }

    MassProperties {
        total_volume,
        bbox_min: global_min,
        bbox_max: global_max,
        center_of_gravity: cog,
        component_count: components.len(),
    }
}

fn compute_brep_bbox(brep: &BRep) -> ([f64; 3], [f64; 3]) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];

    for (_, vertex) in brep.vertices.iter() {
        let coords = [vertex.point.x, vertex.point.y, vertex.point.z];
        for i in 0..3 {
            if coords[i] < min[i] { min[i] = coords[i]; }
            if coords[i] > max[i] { max[i] = coords[i]; }
        }
    }

    if min[0].is_infinite() {
        return ([0.0; 3], [0.0; 3]);
    }

    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;

    #[test]
    fn empty_assembly_zero_properties() {
        let props = compute_mass_properties(&[]);
        assert_eq!(props.total_volume, 0.0);
        assert_eq!(props.component_count, 0);
    }

    #[test]
    fn single_box_volume() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let props = compute_mass_properties(&[("comp1".into(), brep)]);
        // Box volume = 10 × 5 × 3 = 150
        assert!((props.total_volume - 150.0).abs() < 0.1,
            "Volume should be ~150, got {}", props.total_volume);
        assert_eq!(props.component_count, 1);
    }

    #[test]
    fn single_box_center_of_gravity() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let props = compute_mass_properties(&[("comp1".into(), brep)]);
        // COG should be at center: (5, 2.5, 1.5)
        assert!((props.center_of_gravity[0] - 5.0).abs() < 0.1);
        assert!((props.center_of_gravity[1] - 2.5).abs() < 0.1);
        assert!((props.center_of_gravity[2] - 1.5).abs() < 0.1);
    }

    #[test]
    fn two_boxes_combined_volume() {
        let brep1 = build_box_brep(10.0, 10.0, 10.0).unwrap(); // 1000
        let brep2 = build_box_brep(5.0, 5.0, 5.0).unwrap();    // 125
        let props = compute_mass_properties(&[
            ("comp1".into(), brep1),
            ("comp2".into(), brep2),
        ]);
        assert!((props.total_volume - 1125.0).abs() < 0.1);
        assert_eq!(props.component_count, 2);
    }

    #[test]
    fn bounding_box_encompasses_all() {
        let brep1 = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let brep2 = build_box_brep(5.0, 5.0, 5.0).unwrap();
        let props = compute_mass_properties(&[
            ("comp1".into(), brep1),
            ("comp2".into(), brep2),
        ]);
        // Global bbox should encompass both (both start at origin)
        assert!((props.bbox_min[0]).abs() < 0.01);
        assert!((props.bbox_max[0] - 10.0).abs() < 0.01);
    }
}
