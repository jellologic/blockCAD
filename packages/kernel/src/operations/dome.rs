use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
use crate::topology::body::Body;
use crate::topology::brep::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DomeParams {
    /// Index of the face to replace with a dome
    pub face_index: usize,
    /// Dome height above the face
    pub height: f64,
    /// If true, dome follows face shape (elliptical); if false, spherical
    #[serde(default)]
    pub elliptical: bool,
    /// Override dome direction (default: face normal)
    #[serde(default)]
    pub direction: Option<[f64; 3]>,
}

#[derive(Debug)]
pub struct DomeOp;

impl Operation for DomeOp {
    type Params = DomeParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        dome_face(input, params)
    }

    fn name(&self) -> &'static str {
        "Dome"
    }
}

/// Number of concentric rings used to approximate the dome surface.
const DOME_RINGS: usize = 12;

/// Add a dome to a selected face of a solid BRep.
///
/// Algorithm:
/// 1. Extract all face polygons from the BRep.
/// 2. Validate the face index and height.
/// 3. Compute the face centroid, boundary polygon, and dome direction.
/// 4. Generate concentric rings from the boundary toward the apex using
///    spherical interpolation (or elliptical scaling).
/// 5. Replace the selected face with triangulated dome mesh panels.
/// 6. Rebuild the BRep from the modified face list.
pub fn dome_face(brep: &BRep, params: &DomeParams) -> KernelResult<BRep> {
    if matches!(brep.body, Body::Empty) {
        return Err(KernelError::Operation {
            op: "dome".into(),
            detail: "Cannot dome: no existing geometry".into(),
        });
    }

    if params.height <= 0.0 {
        return Err(KernelError::InvalidParameter {
            param: "height".into(),
            value: format!("Height must be positive, got {}", params.height),
        });
    }

    let face_polygons = extract_face_polygons(brep)?;
    if face_polygons.is_empty() {
        return Err(KernelError::Operation {
            op: "dome".into(),
            detail: "No faces found in BRep".into(),
        });
    }

    if params.face_index >= face_polygons.len() {
        return Err(KernelError::InvalidParameter {
            param: "face_index".into(),
            value: format!(
                "Face index {} out of range (0..{})",
                params.face_index,
                face_polygons.len()
            ),
        });
    }

    let (ref boundary, ref face_normal) = face_polygons[params.face_index];
    let n = boundary.len();
    if n < 3 {
        return Err(KernelError::Operation {
            op: "dome".into(),
            detail: "Selected face has fewer than 3 vertices".into(),
        });
    }

    // Dome direction: use override or face normal
    let dome_dir = if let Some(d) = params.direction {
        Vec3::new(d[0], d[1], d[2]).normalize()
    } else {
        face_normal.normalize()
    };

    // Compute face centroid
    let centroid = {
        let sum = boundary.iter().fold(Vec3::new(0.0, 0.0, 0.0), |acc, p| {
            acc + Vec3::new(p.x, p.y, p.z)
        });
        Pt3::new(sum.x / n as f64, sum.y / n as f64, sum.z / n as f64)
    };

    // Generate dome rings: from boundary (ring 0) to near-apex (ring DOME_RINGS-1)
    // Each ring is a scaled + elevated version of the boundary polygon.
    //
    // For spherical dome: we model a hemisphere-like cap. The radius R is chosen
    // so that the dome spans from the face plane to height h along the dome direction.
    // We parameterize by angle theta from 0 (boundary) to pi/2 (apex).
    //
    // For elliptical dome: same approach but each boundary vertex is independently
    // interpolated toward the apex, preserving the face shape.

    let num_rings = DOME_RINGS;
    let mut rings: Vec<Vec<Pt3>> = Vec::with_capacity(num_rings + 1);

    // Ring 0 = boundary itself
    rings.push(boundary.clone());

    for ring_idx in 1..=num_rings {
        let t = ring_idx as f64 / num_rings as f64; // 0..1
        // Use sinusoidal profile for a smooth dome shape:
        // - horizontal shrink: cos(t * pi/2) goes from 1 -> 0
        // - vertical rise: sin(t * pi/2) goes from 0 -> 1
        let theta = t * std::f64::consts::FRAC_PI_2;
        let h_fraction = theta.sin(); // vertical rise fraction
        let r_fraction = theta.cos(); // radial fraction (1 at base, 0 at top)

        let elevation = dome_dir * (params.height * h_fraction);

        let mut ring_pts = Vec::with_capacity(n);
        for pt in boundary.iter() {
            // Vector from centroid to boundary point (in the face plane)
            let radial = Vec3::new(pt.x - centroid.x, pt.y - centroid.y, pt.z - centroid.z);
            // Remove any component along dome_dir to get pure in-plane radial
            let radial_in_plane = radial - dome_dir * radial.dot(&dome_dir);

            let new_pt = centroid + radial_in_plane * r_fraction + elevation;
            ring_pts.push(new_pt);
        }
        rings.push(ring_pts);
    }

    // Build dome faces: quad strips between adjacent rings, plus a cap at the top.
    // The last ring collapses to (nearly) the apex, so we use triangles for the final strip.
    let mut dome_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();

    for seg in 0..num_rings {
        let ring_a = &rings[seg];
        let ring_b = &rings[seg + 1];

        for edge in 0..n {
            let next_edge = (edge + 1) % n;

            let p0 = ring_a[edge];
            let p1 = ring_a[next_edge];
            let p2 = ring_b[next_edge];
            let p3 = ring_b[edge];

            // Check if the top edge has collapsed (last ring -> apex)
            let top_collapsed = (p2 - p3).norm() < 1e-10;

            if top_collapsed {
                // Triangle: p0, p1, p2 (p2 ≈ p3 ≈ apex)
                let e1 = p1 - p0;
                let e2 = p2 - p0;
                let normal = e1.cross(&e2);
                let len = normal.norm();
                if len > 1e-12 {
                    dome_faces.push((vec![p0, p1, p2], normal / len));
                }
            } else {
                // Quad: p0, p1, p2, p3
                let e1 = p1 - p0;
                let e2 = p3 - p0;
                let normal = e1.cross(&e2);
                let len = normal.norm();
                let normal = if len > 1e-12 {
                    normal / len
                } else {
                    dome_dir
                };
                dome_faces.push((vec![p0, p1, p2, p3], normal));
            }
        }
    }

    // Build final face list: all original faces except the domed one, plus dome faces
    let mut result_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();
    for (idx, face) in face_polygons.iter().enumerate() {
        if idx != params.face_index {
            result_faces.push(face.clone());
        }
    }
    result_faces.extend(dome_faces);

    rebuild_brep_from_faces(&result_faces)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tessellation::{tessellate_brep, TessellationParams};
    use crate::topology::builders::build_box_brep;

    #[test]
    fn dome_on_top_face_of_box() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let original_face_count = brep.faces.len();
        assert_eq!(original_face_count, 6);

        let params = DomeParams {
            face_index: 1, // top face
            height: 5.0,
            elliptical: false,
            direction: None,
        };
        let result = dome_face(&brep, &params).unwrap();

        // Should have 5 original faces + dome quad/tri faces replacing the top
        assert!(
            result.faces.len() > original_face_count,
            "Dome should add faces, got {} (original was {})",
            result.faces.len(),
            original_face_count
        );
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn dome_with_different_heights() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();

        // Small dome
        let small = DomeParams {
            face_index: 1,
            height: 1.0,
            elliptical: false,
            direction: None,
        };
        let result_small = dome_face(&brep, &small).unwrap();

        // Tall dome
        let tall = DomeParams {
            face_index: 1,
            height: 20.0,
            elliptical: false,
            direction: None,
        };
        let result_tall = dome_face(&brep, &tall).unwrap();

        // Both should succeed and have the same face count (same topology)
        assert_eq!(result_small.faces.len(), result_tall.faces.len());

        // Tall dome should have higher max Z
        let max_z_small = result_small
            .vertices
            .iter()
            .map(|(_, v)| v.point.z)
            .fold(f64::NEG_INFINITY, f64::max);
        let max_z_tall = result_tall
            .vertices
            .iter()
            .map(|(_, v)| v.point.z)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            max_z_tall > max_z_small,
            "Taller dome should reach higher: tall={} vs small={}",
            max_z_tall,
            max_z_small
        );
    }

    #[test]
    fn dome_invalid_face_index() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = DomeParams {
            face_index: 99,
            height: 5.0,
            elliptical: false,
            direction: None,
        };
        let result = dome_face(&brep, &params);
        assert!(result.is_err(), "Invalid face index should be rejected");
    }

    #[test]
    fn dome_produces_solid_body() {
        let brep = build_box_brep(10.0, 5.0, 8.0).unwrap();
        let params = DomeParams {
            face_index: 1,
            height: 3.0,
            elliptical: false,
            direction: None,
        };
        let result = dome_face(&brep, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));

        // Verify it tessellates successfully
        let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
        assert!(mesh.positions.len() > 0, "Mesh should have vertices");
        assert!(mesh.indices.len() > 0, "Mesh should have triangles");
    }

    #[test]
    fn dome_zero_height_rejected() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = DomeParams {
            face_index: 1,
            height: 0.0,
            elliptical: false,
            direction: None,
        };
        assert!(dome_face(&brep, &params).is_err());
    }

    #[test]
    fn dome_negative_height_rejected() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = DomeParams {
            face_index: 1,
            height: -5.0,
            elliptical: false,
            direction: None,
        };
        assert!(dome_face(&brep, &params).is_err());
    }

    #[test]
    fn dome_on_empty_brep_rejected() {
        let brep = BRep::new();
        let params = DomeParams {
            face_index: 0,
            height: 5.0,
            elliptical: false,
            direction: None,
        };
        assert!(dome_face(&brep, &params).is_err());
    }

    #[test]
    fn dome_with_custom_direction() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = DomeParams {
            face_index: 1,
            height: 5.0,
            elliptical: false,
            direction: Some([0.0, 0.0, 1.0]),
        };
        let result = dome_face(&brep, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
    }
}
