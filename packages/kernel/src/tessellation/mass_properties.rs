//! Accurate mass properties from tessellated mesh using the divergence theorem.

use super::mesh::TriMesh;

/// Mass properties computed from a closed triangle mesh.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MassProperties {
    /// Volume (cubic units). Exact for closed meshes via divergence theorem.
    pub volume: f64,
    /// Total surface area (square units).
    pub surface_area: f64,
    /// Center of mass (volume-weighted centroid).
    pub center_of_mass: [f64; 3],
    /// Bounding box min corner.
    pub bbox_min: [f64; 3],
    /// Bounding box max corner.
    pub bbox_max: [f64; 3],
}

/// Compute mass properties from a tessellated mesh.
///
/// Volume uses the divergence theorem: sum of v0·(v1×v2)/6 for each triangle.
/// Surface area is the sum of triangle areas (½|e1×e2|).
/// Center of mass is the volume-weighted centroid.
pub fn compute_mass_properties(mesh: &TriMesh) -> MassProperties {
    let mut volume = 0.0f64;
    let mut surface_area = 0.0f64;
    let mut cx = 0.0f64;
    let mut cy = 0.0f64;
    let mut cz = 0.0f64;
    let mut bbox_min = [f64::INFINITY; 3];
    let mut bbox_max = [f64::NEG_INFINITY; 3];

    for tri in mesh.indices.chunks(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let v0 = [mesh.positions[i0*3] as f64, mesh.positions[i0*3+1] as f64, mesh.positions[i0*3+2] as f64];
        let v1 = [mesh.positions[i1*3] as f64, mesh.positions[i1*3+1] as f64, mesh.positions[i1*3+2] as f64];
        let v2 = [mesh.positions[i2*3] as f64, mesh.positions[i2*3+1] as f64, mesh.positions[i2*3+2] as f64];

        // Volume via divergence theorem: V = Σ v0·(v1×v2) / 6
        let cross = [
            v1[1]*v2[2] - v1[2]*v2[1],
            v1[2]*v2[0] - v1[0]*v2[2],
            v1[0]*v2[1] - v1[1]*v2[0],
        ];
        let signed_vol = v0[0]*cross[0] + v0[1]*cross[1] + v0[2]*cross[2];
        volume += signed_vol;

        // Center of mass contribution (weighted by tetrahedron volume)
        cx += signed_vol * (v0[0] + v1[0] + v2[0]);
        cy += signed_vol * (v0[1] + v1[1] + v2[1]);
        cz += signed_vol * (v0[2] + v1[2] + v2[2]);

        // Surface area: ½|e1×e2|
        let e1 = [v1[0]-v0[0], v1[1]-v0[1], v1[2]-v0[2]];
        let e2 = [v2[0]-v0[0], v2[1]-v0[1], v2[2]-v0[2]];
        let tri_cross = [
            e1[1]*e2[2] - e1[2]*e2[1],
            e1[2]*e2[0] - e1[0]*e2[2],
            e1[0]*e2[1] - e1[1]*e2[0],
        ];
        surface_area += (tri_cross[0]*tri_cross[0] + tri_cross[1]*tri_cross[1] + tri_cross[2]*tri_cross[2]).sqrt() / 2.0;

        // Bounding box
        for v in &[v0, v1, v2] {
            for i in 0..3 {
                if v[i] < bbox_min[i] { bbox_min[i] = v[i]; }
                if v[i] > bbox_max[i] { bbox_max[i] = v[i]; }
            }
        }
    }

    volume /= 6.0;

    // Center of mass = weighted centroid / (4 * total_volume)
    let com = if volume.abs() > 1e-20 {
        let denom = 24.0 * volume;
        [cx / denom, cy / denom, cz / denom]
    } else {
        [0.0; 3]
    };

    if bbox_min[0].is_infinite() {
        bbox_min = [0.0; 3];
        bbox_max = [0.0; 3];
    }

    MassProperties {
        volume,
        surface_area,
        center_of_mass: com,
        bbox_min,
        bbox_max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;
    use crate::tessellation::{tessellate_brep, TessellationParams};

    #[test]
    fn mass_props_box_volume() {
        let brep = build_box_brep(10.0, 5.0, 7.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);
        assert!((props.volume - 350.0).abs() < 0.1, "Volume={:.2}, expected 350", props.volume);
    }

    #[test]
    fn mass_props_box_surface_area() {
        let brep = build_box_brep(10.0, 5.0, 7.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);
        let expected = 2.0 * (10.0*5.0 + 10.0*7.0 + 5.0*7.0); // 310
        assert!((props.surface_area - expected).abs() < 0.1, "SA={:.2}, expected {}", props.surface_area, expected);
    }

    #[test]
    fn mass_props_box_center_of_mass() {
        let brep = build_box_brep(10.0, 5.0, 7.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);
        assert!((props.center_of_mass[0] - 5.0).abs() < 0.1, "CoM X={:.2}", props.center_of_mass[0]);
        assert!((props.center_of_mass[1] - 2.5).abs() < 0.1, "CoM Y={:.2}", props.center_of_mass[1]);
        assert!((props.center_of_mass[2] - 3.5).abs() < 0.1, "CoM Z={:.2}", props.center_of_mass[2]);
    }

    #[test]
    fn mass_props_unit_cube() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);
        assert!((props.volume - 1.0).abs() < 0.001);
        assert!((props.surface_area - 6.0).abs() < 0.001);
        assert!((props.center_of_mass[0] - 0.5).abs() < 0.001);
    }
}
