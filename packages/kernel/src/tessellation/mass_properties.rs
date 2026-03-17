//! Accurate mass properties from tessellated mesh using the divergence theorem.
//!
//! Implements volume integrals converted to surface integrals for computing
//! volume, center of mass, and inertia tensor from closed triangle meshes.
//! Based on "Fast and Accurate Computation of Polyhedral Mass Properties"
//! by Brian Mirtich (1996).

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
    /// Inertia tensor (3x3 symmetric matrix) about the center of mass,
    /// assuming uniform density of 1.0. Multiply by density for actual values.
    pub inertia_tensor: [[f64; 3]; 3],
    /// Principal moments of inertia (eigenvalues of inertia_tensor), sorted ascending.
    pub principal_moments: [f64; 3],
    /// Principal axes (eigenvectors of inertia_tensor), one per row,
    /// corresponding to principal_moments.
    pub principal_axes: [[f64; 3]; 3],
    /// Bounding box min corner.
    pub bbox_min: [f64; 3],
    /// Bounding box max corner.
    pub bbox_max: [f64; 3],
}

/// Compute mass properties from a tessellated mesh assuming uniform density.
///
/// Volume uses the divergence theorem: sum of v0·(v1×v2)/6 for each triangle.
/// Surface area is the sum of triangle areas (1/2 |e1 x e2|).
/// Center of mass is the volume-weighted centroid.
/// Inertia tensor uses volume integrals of x², y², z², xy, xz, yz converted
/// to surface integrals via the divergence theorem.
pub fn compute_mass_properties(mesh: &TriMesh) -> MassProperties {
    let mut volume = 0.0f64;
    let mut surface_area = 0.0f64;
    let mut cx = 0.0f64;
    let mut cy = 0.0f64;
    let mut cz = 0.0f64;
    let mut bbox_min = [f64::INFINITY; 3];
    let mut bbox_max = [f64::NEG_INFINITY; 3];

    // Second-order volume integrals for inertia tensor (about origin)
    let mut xx = 0.0f64;
    let mut yy = 0.0f64;
    let mut zz = 0.0f64;
    let mut xy = 0.0f64;
    let mut xz = 0.0f64;
    let mut yz = 0.0f64;

    for tri in mesh.indices.chunks(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let v0 = [
            mesh.positions[i0 * 3] as f64,
            mesh.positions[i0 * 3 + 1] as f64,
            mesh.positions[i0 * 3 + 2] as f64,
        ];
        let v1 = [
            mesh.positions[i1 * 3] as f64,
            mesh.positions[i1 * 3 + 1] as f64,
            mesh.positions[i1 * 3 + 2] as f64,
        ];
        let v2 = [
            mesh.positions[i2 * 3] as f64,
            mesh.positions[i2 * 3 + 1] as f64,
            mesh.positions[i2 * 3 + 2] as f64,
        ];

        // Volume via divergence theorem: V = Σ v0·(v1×v2) / 6
        let cross = [
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[2],
            v1[0] * v2[1] - v1[1] * v2[0],
        ];
        let signed_vol = v0[0] * cross[0] + v0[1] * cross[1] + v0[2] * cross[2];
        volume += signed_vol;

        // Center of mass contribution (weighted by tetrahedron volume)
        cx += signed_vol * (v0[0] + v1[0] + v2[0]);
        cy += signed_vol * (v0[1] + v1[1] + v2[1]);
        cz += signed_vol * (v0[2] + v1[2] + v2[2]);

        // Second-order volume integrals via divergence theorem.
        // For each signed tetrahedron (origin, v0, v1, v2), the contribution
        // to ∫x² dV, ∫xy dV, etc. can be computed in closed form.
        //
        // For a tetrahedron with one vertex at origin and the other three at
        // a, b, c, the volume integral of x_i * x_j over the tet is:
        //   det * (sum of products) / 60  for diagonal terms
        //   det * (sum of products) / 120 for off-diagonal terms
        //
        // where det = a · (b × c) = signed_vol

        // Diagonal: ∫x² dV contribution
        // = det/60 * (a_x² + b_x² + c_x² + a_x*b_x + a_x*c_x + b_x*c_x)
        let (a, b, c) = (v0, v1, v2);
        xx += signed_vol
            * (a[0] * a[0]
                + b[0] * b[0]
                + c[0] * c[0]
                + a[0] * b[0]
                + a[0] * c[0]
                + b[0] * c[0]);
        yy += signed_vol
            * (a[1] * a[1]
                + b[1] * b[1]
                + c[1] * c[1]
                + a[1] * b[1]
                + a[1] * c[1]
                + b[1] * c[1]);
        zz += signed_vol
            * (a[2] * a[2]
                + b[2] * b[2]
                + c[2] * c[2]
                + a[2] * b[2]
                + a[2] * c[2]
                + b[2] * c[2]);

        // Off-diagonal: ∫xy dV contribution
        // = det/120 * (2*a_x*a_y + 2*b_x*b_y + 2*c_x*c_y
        //             + a_x*b_y + a_y*b_x + a_x*c_y + a_y*c_x
        //             + b_x*c_y + b_y*c_x)
        xy += signed_vol
            * (2.0 * a[0] * a[1]
                + 2.0 * b[0] * b[1]
                + 2.0 * c[0] * c[1]
                + a[0] * b[1]
                + a[1] * b[0]
                + a[0] * c[1]
                + a[1] * c[0]
                + b[0] * c[1]
                + b[1] * c[0]);
        xz += signed_vol
            * (2.0 * a[0] * a[2]
                + 2.0 * b[0] * b[2]
                + 2.0 * c[0] * c[2]
                + a[0] * b[2]
                + a[2] * b[0]
                + a[0] * c[2]
                + a[2] * c[0]
                + b[0] * c[2]
                + b[2] * c[0]);
        yz += signed_vol
            * (2.0 * a[1] * a[2]
                + 2.0 * b[1] * b[2]
                + 2.0 * c[1] * c[2]
                + a[1] * b[2]
                + a[2] * b[1]
                + a[1] * c[2]
                + a[2] * c[1]
                + b[1] * c[2]
                + b[2] * c[1]);

        // Surface area: 1/2 |e1 × e2|
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let tri_cross = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        surface_area += (tri_cross[0] * tri_cross[0]
            + tri_cross[1] * tri_cross[1]
            + tri_cross[2] * tri_cross[2])
            .sqrt()
            / 2.0;

        // Bounding box
        for v in &[v0, v1, v2] {
            for i in 0..3 {
                if v[i] < bbox_min[i] {
                    bbox_min[i] = v[i];
                }
                if v[i] > bbox_max[i] {
                    bbox_max[i] = v[i];
                }
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

    // Finalize second-order integrals
    // Diagonal integrals: divide by 60 (factor of 6 from volume already included
    // in signed_vol, so we divide by 60 total: /6 from volume, /10 from tet integral)
    xx /= 60.0;
    yy /= 60.0;
    zz /= 60.0;
    // Off-diagonal integrals: divide by 120
    xy /= 120.0;
    xz /= 120.0;
    yz /= 120.0;

    // Compute inertia tensor about center of mass using parallel axis theorem.
    // I_about_origin uses the raw volume integrals:
    //   I_xx_origin = density * (∫y² dV + ∫z² dV)
    //   I_xy_origin = -density * ∫xy dV
    // Then shift to center of mass:
    //   I_xx_com = I_xx_origin - m * (com_y² + com_z²)
    //   I_xy_com = I_xy_origin + m * com_x * com_y
    // We use density = 1.0, so mass = volume.
    let m = volume.abs();

    let inertia_tensor = if m > 1e-20 {
        // Inertia about origin
        let ixx_o = yy + zz;
        let iyy_o = xx + zz;
        let izz_o = xx + yy;
        let ixy_o = -xy;
        let ixz_o = -xz;
        let iyz_o = -yz;

        // Parallel axis theorem to shift to center of mass
        let ixx = ixx_o - m * (com[1] * com[1] + com[2] * com[2]);
        let iyy = iyy_o - m * (com[0] * com[0] + com[2] * com[2]);
        let izz = izz_o - m * (com[0] * com[0] + com[1] * com[1]);
        let ixy = ixy_o + m * com[0] * com[1];
        let ixz = ixz_o + m * com[0] * com[2];
        let iyz = iyz_o + m * com[1] * com[2];

        [[ixx, ixy, ixz], [ixy, iyy, iyz], [ixz, iyz, izz]]
    } else {
        [[0.0; 3]; 3]
    };

    // Compute principal moments and axes via eigendecomposition of the
    // 3x3 symmetric inertia tensor.
    let (principal_moments, principal_axes) = symmetric_eigen_3x3(inertia_tensor);

    if bbox_min[0].is_infinite() {
        bbox_min = [0.0; 3];
        bbox_max = [0.0; 3];
    }

    MassProperties {
        volume,
        surface_area,
        center_of_mass: com,
        inertia_tensor,
        principal_moments,
        principal_axes,
        bbox_min,
        bbox_max,
    }
}

/// Compute mass properties with a specified density.
///
/// The inertia tensor and principal moments are scaled by density.
/// Volume, surface area, center of mass, and principal axes are unaffected.
pub fn compute_mass_properties_with_density(mesh: &TriMesh, density: f64) -> MassProperties {
    let mut props = compute_mass_properties(mesh);
    // Scale inertia tensor by density
    for i in 0..3 {
        for j in 0..3 {
            props.inertia_tensor[i][j] *= density;
        }
    }
    for i in 0..3 {
        props.principal_moments[i] *= density;
    }
    props
}

// ── 3x3 Symmetric Eigenvalue Solver ─────────────────────────────────────────

/// Eigendecomposition of a 3x3 real symmetric matrix.
/// Returns (eigenvalues sorted ascending, eigenvectors as rows).
/// Uses the analytical closed-form solution based on Cardano's formula.
fn symmetric_eigen_3x3(m: [[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    let a = m[0][0];
    let b = m[1][1];
    let c = m[2][2];
    let d = m[0][1]; // = m[1][0]
    let e = m[0][2]; // = m[2][0]
    let f = m[1][2]; // = m[2][1]

    // Characteristic equation: λ³ - p*λ² + q*λ - r = 0
    // where p = trace, q = sum of 2x2 minors, r = det
    let p = a + b + c;
    let q = a * b + a * c + b * c - d * d - e * e - f * f;
    let r = a * b * c + 2.0 * d * e * f - a * f * f - b * e * e - c * d * d;

    // Solve using Cardano's method for the depressed cubic
    // Substituting λ = t + p/3 gives t³ + pt' + q' = 0
    let p_over_3 = p / 3.0;
    let pp = (p * p - 3.0 * q) / 9.0;
    let qq = (2.0 * p * p * p - 9.0 * p * q + 27.0 * r) / 54.0;

    let mut eigenvalues;

    let det = qq * qq - pp * pp * pp;
    if det <= 0.0 {
        // Three real roots (the typical case for inertia tensors)
        let phi = if pp.abs() < 1e-30 {
            0.0
        } else {
            (qq / (pp * pp.sqrt())).clamp(-1.0, 1.0).acos() / 3.0
        };
        let two_sqrt_pp = 2.0 * pp.sqrt();

        eigenvalues = [
            -two_sqrt_pp * (phi + 2.0 * std::f64::consts::FRAC_PI_3).cos() + p_over_3,
            -two_sqrt_pp * (phi - 2.0 * std::f64::consts::FRAC_PI_3).cos() + p_over_3,
            -two_sqrt_pp * phi.cos() + p_over_3,
        ];
    } else {
        // Fallback: one real root (shouldn't happen for real symmetric matrices,
        // but handle numerically)
        let sqrt_det = det.sqrt();
        let s = -qq + sqrt_det;
        let t = -qq - sqrt_det;
        let s = s.signum() * s.abs().cbrt();
        let t = t.signum() * t.abs().cbrt();
        let ev = s + t + p_over_3;
        eigenvalues = [ev, ev, ev];
    }

    // Sort ascending
    eigenvalues.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Compute eigenvectors
    let mut axes = [[0.0f64; 3]; 3];
    for (idx, &ev) in eigenvalues.iter().enumerate() {
        axes[idx] = find_eigenvector(m, ev);
    }

    // Ensure orthogonality via Gram-Schmidt
    axes = orthonormalize(axes);

    (eigenvalues, axes)
}

/// Find an eigenvector for a 3x3 symmetric matrix given an eigenvalue.
fn find_eigenvector(m: [[f64; 3]; 3], eigenvalue: f64) -> [f64; 3] {
    // (M - λI) * v = 0
    let a = [
        [m[0][0] - eigenvalue, m[0][1], m[0][2]],
        [m[1][0], m[1][1] - eigenvalue, m[1][2]],
        [m[2][0], m[2][1], m[2][2] - eigenvalue],
    ];

    // Try cross products of rows to find a non-zero vector in the null space
    let row0 = a[0];
    let row1 = a[1];
    let row2 = a[2];

    let candidates = [
        cross(row0, row1),
        cross(row0, row2),
        cross(row1, row2),
    ];

    // Pick the candidate with largest magnitude
    let mut best = [1.0, 0.0, 0.0];
    let mut best_mag = 0.0;
    for c in &candidates {
        let mag = c[0] * c[0] + c[1] * c[1] + c[2] * c[2];
        if mag > best_mag {
            best_mag = mag;
            best = *c;
        }
    }

    // Normalize
    let len = best_mag.sqrt();
    if len > 1e-15 {
        [best[0] / len, best[1] / len, best[2] / len]
    } else {
        // Degenerate case: eigenvalue has multiplicity > 1
        // Return a unit vector not already used
        [1.0, 0.0, 0.0]
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 1e-15 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        v
    }
}

/// Gram-Schmidt orthonormalization for 3 vectors.
fn orthonormalize(mut vecs: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    vecs[0] = normalize(vecs[0]);

    // v1 = v1 - (v1·v0)*v0
    let d10 = dot(vecs[1], vecs[0]);
    for i in 0..3 {
        vecs[1][i] -= d10 * vecs[0][i];
    }
    vecs[1] = normalize(vecs[1]);

    // If v1 is degenerate, pick an orthogonal vector
    if dot(vecs[1], vecs[1]) < 0.5 {
        // Find a vector not parallel to v0
        let trial = if vecs[0][0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let d = dot(trial, vecs[0]);
        vecs[1] = normalize([
            trial[0] - d * vecs[0][0],
            trial[1] - d * vecs[0][1],
            trial[2] - d * vecs[0][2],
        ]);
    }

    // v2 = v0 × v1 (ensures right-handed orthonormal frame)
    vecs[2] = cross(vecs[0], vecs[1]);
    vecs[2] = normalize(vecs[2]);

    vecs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tessellation::mesh::TriMesh;
    use crate::tessellation::{tessellate_brep, TessellationParams};
    use crate::topology::builders::build_box_brep;

    // ── Helper: build a tessellated cylinder mesh ───────────────────────────
    fn build_cylinder_mesh(radius: f64, height: f64, segments: u32) -> TriMesh {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut indices = Vec::new();

        // Vertices: bottom center (0), top center (1),
        // bottom ring (2..2+segments), top ring (2+segments..2+2*segments)
        let n = segments;

        // Bottom center
        positions.extend_from_slice(&[0.0f32, 0.0, 0.0]);
        normals.extend_from_slice(&[0.0f32, -1.0, 0.0]);

        // Top center
        positions.extend_from_slice(&[0.0f32, height as f32, 0.0]);
        normals.extend_from_slice(&[0.0f32, 1.0, 0.0]);

        // Bottom ring
        for i in 0..n {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
            let x = radius * angle.cos();
            let z = radius * angle.sin();
            positions.extend_from_slice(&[x as f32, 0.0, z as f32]);
            normals.extend_from_slice(&[0.0, -1.0, 0.0]);
        }

        // Top ring
        for i in 0..n {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
            let x = radius * angle.cos();
            let z = radius * angle.sin();
            positions.extend_from_slice(&[x as f32, height as f32, z as f32]);
            normals.extend_from_slice(&[0.0, 1.0, 0.0]);
        }

        let bot_center = 0u32;
        let top_center = 1u32;
        let bot_ring_start = 2u32;
        let top_ring_start = 2 + n;

        // Bottom cap (fan from center, winding for outward normal pointing -Y)
        for i in 0..n {
            let next = (i + 1) % n;
            indices.extend_from_slice(&[
                bot_center,
                bot_ring_start + i,
                bot_ring_start + next,
            ]);
        }

        // Top cap (fan from center, winding for outward normal pointing +Y)
        for i in 0..n {
            let next = (i + 1) % n;
            indices.extend_from_slice(&[
                top_center,
                top_ring_start + next,
                top_ring_start + i,
            ]);
        }

        // Side faces (two triangles per quad)
        for i in 0..n {
            let next = (i + 1) % n;
            let b0 = bot_ring_start + i;
            let b1 = bot_ring_start + next;
            let t0 = top_ring_start + i;
            let t1 = top_ring_start + next;
            indices.extend_from_slice(&[b0, t0, b1]);
            indices.extend_from_slice(&[t0, t1, b1]);
        }

        let tri_count = indices.len() / 3;
        TriMesh {
            positions,
            normals,
            uvs: vec![],
            indices,
            face_ids: vec![0; tri_count],
            colors: vec![],
            ..Default::default()
        }
    }

    // ── Existing tests (preserved) ──────────────────────────────────────────

    #[test]
    fn mass_props_box_volume() {
        let brep = build_box_brep(10.0, 5.0, 7.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);
        assert!(
            (props.volume - 350.0).abs() < 0.1,
            "Volume={:.2}, expected 350",
            props.volume
        );
    }

    #[test]
    fn mass_props_box_surface_area() {
        let brep = build_box_brep(10.0, 5.0, 7.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);
        let expected = 2.0 * (10.0 * 5.0 + 10.0 * 7.0 + 5.0 * 7.0); // 310
        assert!(
            (props.surface_area - expected).abs() < 0.1,
            "SA={:.2}, expected {}",
            props.surface_area,
            expected
        );
    }

    #[test]
    fn mass_props_box_center_of_mass() {
        let brep = build_box_brep(10.0, 5.0, 7.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);
        assert!(
            (props.center_of_mass[0] - 5.0).abs() < 0.1,
            "CoM X={:.2}",
            props.center_of_mass[0]
        );
        assert!(
            (props.center_of_mass[1] - 2.5).abs() < 0.1,
            "CoM Y={:.2}",
            props.center_of_mass[1]
        );
        assert!(
            (props.center_of_mass[2] - 3.5).abs() < 0.1,
            "CoM Z={:.2}",
            props.center_of_mass[2]
        );
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

    // ── New inertia tensor tests ────────────────────────────────────────────

    #[test]
    fn inertia_unit_cube() {
        // A unit cube (1x1x1) with uniform density 1.0 has mass = 1.0.
        // Inertia about center of mass:
        //   Ixx = Iyy = Izz = m/12 * (b² + c²) = 1/12 * (1 + 1) = 1/6
        //   Ixy = Ixz = Iyz = 0 (symmetric object)
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);

        let expected_diag = 1.0 / 6.0;
        let tol = 0.005;

        assert!(
            (props.inertia_tensor[0][0] - expected_diag).abs() < tol,
            "Ixx={:.6}, expected {:.6}",
            props.inertia_tensor[0][0],
            expected_diag
        );
        assert!(
            (props.inertia_tensor[1][1] - expected_diag).abs() < tol,
            "Iyy={:.6}, expected {:.6}",
            props.inertia_tensor[1][1],
            expected_diag
        );
        assert!(
            (props.inertia_tensor[2][2] - expected_diag).abs() < tol,
            "Izz={:.6}, expected {:.6}",
            props.inertia_tensor[2][2],
            expected_diag
        );

        // Off-diagonal should be ~0
        assert!(
            props.inertia_tensor[0][1].abs() < tol,
            "Ixy={:.6}",
            props.inertia_tensor[0][1]
        );
        assert!(
            props.inertia_tensor[0][2].abs() < tol,
            "Ixz={:.6}",
            props.inertia_tensor[0][2]
        );
        assert!(
            props.inertia_tensor[1][2].abs() < tol,
            "Iyz={:.6}",
            props.inertia_tensor[1][2]
        );
    }

    #[test]
    fn inertia_rectangular_box() {
        // Box with dimensions a=2, b=3, c=4, density=1
        // Volume = 24, mass = 24
        // Ixx = m/12 * (b² + c²) = 24/12 * (9 + 16) = 50
        // Iyy = m/12 * (a² + c²) = 24/12 * (4 + 16) = 40
        // Izz = m/12 * (a² + b²) = 24/12 * (4 + 9) = 26
        let brep = build_box_brep(2.0, 3.0, 4.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);

        let m = props.volume; // should be ~24
        let tol = 0.5;

        assert!((m - 24.0).abs() < 0.1, "mass={:.2}", m);

        let ixx_expected = m / 12.0 * (9.0 + 16.0);
        let iyy_expected = m / 12.0 * (4.0 + 16.0);
        let izz_expected = m / 12.0 * (4.0 + 9.0);

        assert!(
            (props.inertia_tensor[0][0] - ixx_expected).abs() < tol,
            "Ixx={:.4}, expected {:.4}",
            props.inertia_tensor[0][0],
            ixx_expected
        );
        assert!(
            (props.inertia_tensor[1][1] - iyy_expected).abs() < tol,
            "Iyy={:.4}, expected {:.4}",
            props.inertia_tensor[1][1],
            iyy_expected
        );
        assert!(
            (props.inertia_tensor[2][2] - izz_expected).abs() < tol,
            "Izz={:.4}, expected {:.4}",
            props.inertia_tensor[2][2],
            izz_expected
        );
    }

    #[test]
    fn inertia_cylinder_approx() {
        // Cylinder with radius=1, height=2, axis along Y.
        // Volume = π * r² * h = 2π ≈ 6.2832
        // Inertia about center of mass (density=1, mass=volume):
        //   I_yy (about axis) = m * r² / 2 ≈ π
        //   I_xx = I_zz = m * (3r² + h²) / 12 = m * (3 + 4) / 12 = 7m/12
        let r = 1.0;
        let h = 2.0;
        let mesh = build_cylinder_mesh(r, h, 64);
        let props = compute_mass_properties(&mesh);

        let expected_volume = std::f64::consts::PI * r * r * h;
        let m = props.volume;

        // Volume should be close to analytical (tessellation error)
        assert!(
            (m - expected_volume).abs() / expected_volume < 0.01,
            "Volume={:.4}, expected {:.4}",
            m,
            expected_volume
        );

        let iyy_expected = m * r * r / 2.0;
        let ixx_expected = m * (3.0 * r * r + h * h) / 12.0;

        // Cylinder axis is Y, so Iyy is the axial moment
        let tol_frac = 0.02; // 2% tolerance for tessellation
        assert!(
            (props.inertia_tensor[1][1] - iyy_expected).abs() / iyy_expected < tol_frac,
            "Iyy={:.4}, expected {:.4}",
            props.inertia_tensor[1][1],
            iyy_expected
        );
        assert!(
            (props.inertia_tensor[0][0] - ixx_expected).abs() / ixx_expected < tol_frac,
            "Ixx={:.4}, expected {:.4}",
            props.inertia_tensor[0][0],
            ixx_expected
        );
        assert!(
            (props.inertia_tensor[2][2] - ixx_expected).abs() / ixx_expected < tol_frac,
            "Izz={:.4}, expected {:.4}",
            props.inertia_tensor[2][2],
            ixx_expected
        );
    }

    #[test]
    fn center_of_mass_symmetric_box() {
        // A box centered at (5, 2.5, 3.5) built from (0,0,0) to (10,5,7)
        let brep = build_box_brep(10.0, 5.0, 7.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);

        let tol = 0.05;
        assert!((props.center_of_mass[0] - 5.0).abs() < tol);
        assert!((props.center_of_mass[1] - 2.5).abs() < tol);
        assert!((props.center_of_mass[2] - 3.5).abs() < tol);
    }

    #[test]
    fn center_of_mass_cylinder() {
        // Cylinder from y=0 to y=2, centered at x=0, z=0
        // CoM should be at (0, 1, 0)
        let mesh = build_cylinder_mesh(1.0, 2.0, 48);
        let props = compute_mass_properties(&mesh);

        let tol = 0.02;
        assert!(
            props.center_of_mass[0].abs() < tol,
            "CoM X={:.4}",
            props.center_of_mass[0]
        );
        assert!(
            (props.center_of_mass[1] - 1.0).abs() < tol,
            "CoM Y={:.4}",
            props.center_of_mass[1]
        );
        assert!(
            props.center_of_mass[2].abs() < tol,
            "CoM Z={:.4}",
            props.center_of_mass[2]
        );
    }

    #[test]
    fn principal_axes_symmetric_cube() {
        // For a unit cube, all principal moments are equal (1/6),
        // so any orthonormal frame is valid. Just check that the
        // principal moments are correct and axes are orthonormal.
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);

        let expected = 1.0 / 6.0;
        let tol = 0.005;

        for i in 0..3 {
            assert!(
                (props.principal_moments[i] - expected).abs() < tol,
                "PM[{}]={:.6}, expected {:.6}",
                i,
                props.principal_moments[i],
                expected
            );
        }

        // Check orthonormality of principal axes
        for i in 0..3 {
            let len_sq = dot(props.principal_axes[i], props.principal_axes[i]);
            assert!(
                (len_sq - 1.0).abs() < 1e-6,
                "Axis {} not unit length: {:.6}",
                i,
                len_sq
            );
        }
        for i in 0..3 {
            for j in (i + 1)..3 {
                let d = dot(props.principal_axes[i], props.principal_axes[j]).abs();
                assert!(
                    d < 1e-3,
                    "Axes {} and {} not orthogonal: dot={:.6}",
                    i,
                    j,
                    d
                );
            }
        }
    }

    #[test]
    fn principal_axes_rectangular_box() {
        // For a non-cubic rectangular box, principal axes should align
        // with coordinate axes (since the box is axis-aligned).
        let brep = build_box_brep(2.0, 3.0, 5.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
        let props = compute_mass_properties(&mesh);

        // Principal moments should be sorted ascending.
        // Izz = m/12*(4+9) = 13m/12 (smallest dims)
        // Iyy = m/12*(4+25) = 29m/12
        // Ixx = m/12*(9+25) = 34m/12 (largest)
        assert!(props.principal_moments[0] <= props.principal_moments[1]);
        assert!(props.principal_moments[1] <= props.principal_moments[2]);

        // Each principal axis should be close to a coordinate axis
        // (one component ~1, others ~0)
        for i in 0..3 {
            let ax = props.principal_axes[i];
            let max_component = ax[0].abs().max(ax[1].abs()).max(ax[2].abs());
            assert!(
                max_component > 0.95,
                "Axis {} not aligned: {:?}",
                i,
                ax
            );
        }
    }

    #[test]
    fn density_scaling() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();

        let props1 = compute_mass_properties(&mesh);
        let props2 = compute_mass_properties_with_density(&mesh, 7.8);

        let tol = 1e-6;
        // Inertia should scale by density
        for i in 0..3 {
            for j in 0..3 {
                let expected = props1.inertia_tensor[i][j] * 7.8;
                assert!(
                    (props2.inertia_tensor[i][j] - expected).abs() < tol,
                    "Tensor[{}][{}]={:.6}, expected {:.6}",
                    i,
                    j,
                    props2.inertia_tensor[i][j],
                    expected
                );
            }
        }

        // Volume/CoM unchanged
        assert!((props2.volume - props1.volume).abs() < tol);
        assert!((props2.center_of_mass[0] - props1.center_of_mass[0]).abs() < tol);
    }

    #[test]
    fn eigen_solver_identity() {
        // Identity matrix: eigenvalues = [1,1,1]
        let m = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let (vals, vecs) = symmetric_eigen_3x3(m);
        for i in 0..3 {
            assert!((vals[i] - 1.0).abs() < 1e-10);
        }
        // Vectors should be orthonormal
        for i in 0..3 {
            assert!((dot(vecs[i], vecs[i]) - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn eigen_solver_diagonal() {
        let m = [[3.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 2.0]];
        let (vals, _vecs) = symmetric_eigen_3x3(m);
        // Sorted ascending: 1, 2, 3
        assert!((vals[0] - 1.0).abs() < 1e-10);
        assert!((vals[1] - 2.0).abs() < 1e-10);
        assert!((vals[2] - 3.0).abs() < 1e-10);
    }
}
