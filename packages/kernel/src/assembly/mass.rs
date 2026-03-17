//! Accurate mass properties computation from assembly component BReps.
//!
//! Uses divergence-theorem-based volume integration (via tessellation::mass_properties)
//! for each component, then combines them with the parallel axis theorem for the
//! assembly-level inertia tensor.

use crate::tessellation::{tessellate_brep, TessellationParams};
use crate::tessellation::mass_properties as mesh_mass;
use crate::topology::BRep;

/// Mass properties of an individual component in the assembly.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentMassProperties {
    pub component_id: String,
    pub volume: f64,
    pub mass: f64,
    pub center_of_mass: [f64; 3],
}

/// Mass properties of the entire assembly.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssemblyMassProperties {
    /// Total volume of all components (cubic units).
    pub total_volume: f64,
    /// Total mass (sum of component volumes * density).
    pub total_mass: f64,
    /// Center of gravity (mass-weighted centroid).
    pub center_of_gravity: [f64; 3],
    /// Assembly-level inertia tensor about the center of gravity.
    /// Assumes uniform density per component. Multiply by density for actual values.
    pub inertia_tensor: [[f64; 3]; 3],
    /// Number of components included.
    pub component_count: usize,
    /// Bounding box min corner.
    pub bbox_min: [f64; 3],
    /// Bounding box max corner.
    pub bbox_max: [f64; 3],
    /// Per-component mass properties breakdown.
    pub per_component: Vec<ComponentMassProperties>,
}

/// Compute accurate mass properties from positioned component BReps.
///
/// Each component is tessellated and its volume, center of mass, and inertia tensor
/// are computed via the divergence theorem. The assembly inertia tensor is obtained
/// by combining per-component tensors using the parallel axis theorem.
///
/// `density` is a uniform density applied to all components (default 1.0).
pub fn compute_assembly_mass_properties(
    components: &[(String, BRep)],
    density: f64,
) -> AssemblyMassProperties {
    if components.is_empty() {
        return AssemblyMassProperties {
            total_volume: 0.0,
            total_mass: 0.0,
            bbox_min: [0.0; 3],
            bbox_max: [0.0; 3],
            center_of_gravity: [0.0; 3],
            inertia_tensor: [[0.0; 3]; 3],
            component_count: 0,
            per_component: Vec::new(),
        };
    }

    let tess_params = TessellationParams::default();

    // Per-component data: (volume, mass, center_of_mass, inertia_tensor_about_com, bbox_min, bbox_max)
    struct CompData {
        id: String,
        volume: f64,
        mass: f64,
        com: [f64; 3],
        /// Inertia tensor about the component's own center of mass, scaled by density
        inertia: [[f64; 3]; 3],
    }

    let mut comp_data: Vec<CompData> = Vec::with_capacity(components.len());
    let mut global_min = [f64::INFINITY; 3];
    let mut global_max = [f64::NEG_INFINITY; 3];

    for (id, brep) in components {
        let mesh = match tessellate_brep(brep, &tess_params) {
            Ok(m) => m,
            Err(_) => continue, // skip components that fail to tessellate
        };

        let props = mesh_mass::compute_mass_properties(&mesh);
        let vol = props.volume.abs();
        let mass = vol * density;

        // Scale inertia tensor by density (mesh_mass assumes density=1)
        let mut inertia = props.inertia_tensor;
        for i in 0..3 {
            for j in 0..3 {
                inertia[i][j] *= density;
            }
        }

        // Update global bbox
        for i in 0..3 {
            if props.bbox_min[i] < global_min[i] {
                global_min[i] = props.bbox_min[i];
            }
            if props.bbox_max[i] > global_max[i] {
                global_max[i] = props.bbox_max[i];
            }
        }

        comp_data.push(CompData {
            id: id.clone(),
            volume: vol,
            mass,
            com: props.center_of_mass,
            inertia,
        });
    }

    if comp_data.is_empty() {
        return AssemblyMassProperties {
            total_volume: 0.0,
            total_mass: 0.0,
            bbox_min: [0.0; 3],
            bbox_max: [0.0; 3],
            center_of_gravity: [0.0; 3],
            inertia_tensor: [[0.0; 3]; 3],
            component_count: components.len(),
            per_component: Vec::new(),
        };
    }

    // Compute total mass and mass-weighted centroid
    let total_mass: f64 = comp_data.iter().map(|c| c.mass).sum();
    let total_volume: f64 = comp_data.iter().map(|c| c.volume).sum();

    let cog = if total_mass > 1e-20 {
        let mut cg = [0.0; 3];
        for c in &comp_data {
            for i in 0..3 {
                cg[i] += c.mass * c.com[i];
            }
        }
        for i in 0..3 {
            cg[i] /= total_mass;
        }
        cg
    } else {
        [0.0; 3]
    };

    // Combine inertia tensors using parallel axis theorem:
    // I_assembly = sum_i [ I_i(about component_i COM) + m_i * ((d_i . d_i) * Identity - d_i outer d_i) ]
    // where d_i = component_i COM - assembly COG
    let mut assembly_inertia = [[0.0f64; 3]; 3];

    for c in &comp_data {
        // Distance from component COM to assembly COG
        let d = [
            c.com[0] - cog[0],
            c.com[1] - cog[1],
            c.com[2] - cog[2],
        ];
        let d_sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];

        for i in 0..3 {
            for j in 0..3 {
                // Component's own inertia (about its COM)
                assembly_inertia[i][j] += c.inertia[i][j];
                // Parallel axis theorem: m * (d^2 * delta_ij - d_i * d_j)
                let kronecker = if i == j { 1.0 } else { 0.0 };
                assembly_inertia[i][j] += c.mass * (d_sq * kronecker - d[i] * d[j]);
            }
        }
    }

    // Handle degenerate bbox
    if global_min[0].is_infinite() {
        global_min = [0.0; 3];
        global_max = [0.0; 3];
    }

    let per_component: Vec<ComponentMassProperties> = comp_data
        .iter()
        .map(|c| ComponentMassProperties {
            component_id: c.id.clone(),
            volume: c.volume,
            mass: c.mass,
            center_of_mass: c.com,
        })
        .collect();

    AssemblyMassProperties {
        total_volume,
        total_mass,
        center_of_gravity: cog,
        inertia_tensor: assembly_inertia,
        component_count: comp_data.len(),
        bbox_min: global_min,
        bbox_max: global_max,
        per_component,
    }
}

/// Convenience wrapper with default density of 1.0.
/// This is the backwards-compatible entry point used by WASM bindings.
pub fn compute_mass_properties(components: &[(String, BRep)]) -> AssemblyMassProperties {
    compute_assembly_mass_properties(components, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;

    #[test]
    fn empty_assembly_zero_properties() {
        let props = compute_mass_properties(&[]);
        assert_eq!(props.total_volume, 0.0);
        assert_eq!(props.total_mass, 0.0);
        assert_eq!(props.component_count, 0);
        assert!(props.per_component.is_empty());
    }

    #[test]
    fn single_box_volume_accurate() {
        // 10 * 5 * 7 = 350
        let brep = build_box_brep(10.0, 5.0, 7.0).unwrap();
        let props = compute_mass_properties(&[("comp1".into(), brep)]);
        assert!(
            (props.total_volume - 350.0).abs() < 0.1,
            "Volume should be ~350, got {}",
            props.total_volume
        );
        assert!(
            (props.total_mass - 350.0).abs() < 0.1,
            "Mass should be ~350 (density=1), got {}",
            props.total_mass
        );
        assert_eq!(props.component_count, 1);
        assert_eq!(props.per_component.len(), 1);
        assert_eq!(props.per_component[0].component_id, "comp1");
    }

    #[test]
    fn single_box_center_of_gravity() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        let props = compute_mass_properties(&[("comp1".into(), brep)]);
        // COG should be at center: (5, 2.5, 1.5)
        assert!(
            (props.center_of_gravity[0] - 5.0).abs() < 0.1,
            "COG X = {}",
            props.center_of_gravity[0]
        );
        assert!(
            (props.center_of_gravity[1] - 2.5).abs() < 0.1,
            "COG Y = {}",
            props.center_of_gravity[1]
        );
        assert!(
            (props.center_of_gravity[2] - 1.5).abs() < 0.1,
            "COG Z = {}",
            props.center_of_gravity[2]
        );
    }

    #[test]
    fn two_boxes_combined_volume() {
        let brep1 = build_box_brep(10.0, 10.0, 10.0).unwrap(); // 1000
        let brep2 = build_box_brep(5.0, 5.0, 5.0).unwrap(); // 125
        let props = compute_mass_properties(&[
            ("comp1".into(), brep1),
            ("comp2".into(), brep2),
        ]);
        assert!(
            (props.total_volume - 1125.0).abs() < 0.5,
            "Volume = {}",
            props.total_volume
        );
        assert_eq!(props.component_count, 2);
        assert_eq!(props.per_component.len(), 2);
    }

    #[test]
    fn assembly_cog_midpoint_of_equal_masses() {
        // Two identical boxes: one at origin, one translated along X by 20.
        // COG should be at (15, 2.5, 3.5) — midpoint of their centers.
        // Box is 10x5x7, so center of first = (5, 2.5, 3.5), center of second = (25, 2.5, 3.5)
        let brep1 = build_box_brep(10.0, 5.0, 7.0).unwrap();

        // Build a second box translated +20 in X by rebuilding vertices
        let brep2 = {
            use crate::topology::builders::build_box_brep;
            use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
            use crate::geometry::{Pt3, Vec3};
            use crate::geometry::transform::{translation, transform_point};
            let base = build_box_brep(10.0, 5.0, 7.0).unwrap();
            let xform = translation(20.0, 0.0, 0.0);
            let faces = extract_face_polygons(&base).unwrap();
            let moved: Vec<(Vec<Pt3>, Vec3)> = faces
                .iter()
                .map(|(pts, n)| {
                    let new_pts: Vec<Pt3> = pts.iter().map(|p| transform_point(&xform, p)).collect();
                    (new_pts, *n)
                })
                .collect();
            rebuild_brep_from_faces(&moved).unwrap()
        };

        let props = compute_mass_properties(&[
            ("comp1".into(), brep1),
            ("comp2".into(), brep2),
        ]);

        let tol = 0.2;
        assert!(
            (props.center_of_gravity[0] - 15.0).abs() < tol,
            "COG X = {}, expected 15",
            props.center_of_gravity[0]
        );
        assert!(
            (props.center_of_gravity[1] - 2.5).abs() < tol,
            "COG Y = {}, expected 2.5",
            props.center_of_gravity[1]
        );
        assert!(
            (props.center_of_gravity[2] - 3.5).abs() < tol,
            "COG Z = {}, expected 3.5",
            props.center_of_gravity[2]
        );
    }

    #[test]
    fn single_box_inertia_tensor() {
        // Box 10x5x7, volume=350, mass=350 (density=1)
        // Inertia about COM:
        //   Ixx = m/12 * (b^2 + c^2) = 350/12 * (25 + 49) = 350/12 * 74 = 2158.33
        //   Iyy = m/12 * (a^2 + c^2) = 350/12 * (100 + 49) = 350/12 * 149 = 4345.83
        //   Izz = m/12 * (a^2 + b^2) = 350/12 * (100 + 25) = 350/12 * 125 = 3645.83
        let brep = build_box_brep(10.0, 5.0, 7.0).unwrap();
        let props = compute_mass_properties(&[("comp1".into(), brep)]);

        let m = props.total_mass;
        let tol = 5.0; // allow some tessellation tolerance

        let ixx_expected = m / 12.0 * (5.0_f64.powi(2) + 7.0_f64.powi(2));
        let iyy_expected = m / 12.0 * (10.0_f64.powi(2) + 7.0_f64.powi(2));
        let izz_expected = m / 12.0 * (10.0_f64.powi(2) + 5.0_f64.powi(2));

        assert!(
            (props.inertia_tensor[0][0] - ixx_expected).abs() < tol,
            "Ixx={:.2}, expected {:.2}",
            props.inertia_tensor[0][0],
            ixx_expected
        );
        assert!(
            (props.inertia_tensor[1][1] - iyy_expected).abs() < tol,
            "Iyy={:.2}, expected {:.2}",
            props.inertia_tensor[1][1],
            iyy_expected
        );
        assert!(
            (props.inertia_tensor[2][2] - izz_expected).abs() < tol,
            "Izz={:.2}, expected {:.2}",
            props.inertia_tensor[2][2],
            izz_expected
        );

        // Off-diagonal should be ~0 for axis-aligned box
        assert!(
            props.inertia_tensor[0][1].abs() < tol,
            "Ixy={:.2}",
            props.inertia_tensor[0][1]
        );
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

    #[test]
    fn density_scales_mass_and_inertia() {
        let brep1 = build_box_brep(10.0, 5.0, 7.0).unwrap();
        let props1 = compute_assembly_mass_properties(
            &[("comp1".into(), brep1)],
            1.0,
        );

        let brep2 = build_box_brep(10.0, 5.0, 7.0).unwrap();
        let props2 = compute_assembly_mass_properties(
            &[("comp1".into(), brep2)],
            7.8,
        );

        // Volume unchanged
        assert!(
            (props2.total_volume - props1.total_volume).abs() < 0.01,
            "Volumes should match"
        );

        // Mass scales by density
        assert!(
            (props2.total_mass - props1.total_mass * 7.8).abs() < 0.5,
            "Mass should scale by density: {} vs {}",
            props2.total_mass,
            props1.total_mass * 7.8
        );

        // Inertia scales by density
        let tol = 1.0;
        for i in 0..3 {
            for j in 0..3 {
                let expected = props1.inertia_tensor[i][j] * 7.8;
                assert!(
                    (props2.inertia_tensor[i][j] - expected).abs() < tol,
                    "Inertia[{}][{}]={:.2}, expected {:.2}",
                    i, j,
                    props2.inertia_tensor[i][j],
                    expected
                );
            }
        }
    }
}
